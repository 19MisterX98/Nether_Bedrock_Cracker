use std::fmt;
use java_random::{JAVA_LCG, Random};
use next_long_reverser::get_next_long;
use crate::{CrackProgress, FLOOR_HASH, ROOF_HASH};
use crate::block_data::{BlockFilter, CheckObject, get_filter_power};
use crate::raw_data::block::Block;
use crate::raw_data::modes::{CrackerMode, OutputMode};
use crate::raw_data::sender::Sender;

fn split_floor_roof(blocks: &[Block], mode: CrackerMode) -> (Vec<BlockFilter>, Vec<BlockFilter>) {
    let mut floor_blocks = vec![];
    let mut roof_blocks = vec![];

    for block in blocks.iter() {
        let filter = BlockFilter::from(block, mode);
        if block.y < 64 {
            floor_blocks.push(filter);
        } else {
            roof_blocks.push(filter);
        }
    }

    (floor_blocks, roof_blocks)
}

#[derive(Debug, Clone)]
enum NextOperation<S: Sender> {
    Layer(Box<Layer<S>>),
    CrossComparison(CrossComparison<S>),
    None,
}

#[derive(Clone)]
struct CrossComparison<S: Sender> {
    sender: S,
    checks: Vec<CheckObject>,
    //java hashes for minecraft:bedrock_floor and minecraft:bedrock_roof
    primary_hash: u64,
    secondary_hash: u64,
    output: OutputMode,
}

pub fn create_filter_tree<S: Sender>(blocks: &[Block], mode: CrackerMode, output: OutputMode, tx: S) -> Layer<S> {

    let (floor_blocks, roof_blocks) = split_floor_roof(blocks, mode);

    let floor_resulting_seeds = get_filter_power(&floor_blocks);
    let roof_resulting_seeds = get_filter_power(&roof_blocks);

    let is_floor_primary_filter = floor_resulting_seeds < roof_resulting_seeds;

    let (mut primary_filter, secondary_filter) = if is_floor_primary_filter {
        (floor_blocks, roof_blocks)
    } else {
        (roof_blocks, floor_blocks)
    };

    //sort everything by filter power
    //wanted to try functional programming
    let mut layers: Vec<Layer<S>> = (0..=12)
        .rev()
        .map(|bits| {
            let mut checks = primary_filter
                .iter_mut()
                .map(|block| (block.discarded_seeds(bits), block))
                .filter(|(discarded_seeds, _)| *discarded_seeds > 0.0)
                .collect::<Vec<(f64, &mut BlockFilter)>>();

            checks.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());

            let res = checks
                .into_iter()
                .map(|(_, a)| a.create_check(bits))
                .collect();

            Layer::new(bits, res)
        })
        .collect();

    // add checks for the other surface
    let final_check = CrossComparison::new(secondary_filter, tx, is_floor_primary_filter, output);
    layers
        .last_mut()
        .map(|layer| layer.next_operation = NextOperation::CrossComparison(final_check));

    layers
        .into_iter()
        .rev()
        .reduce(|next_layer, mut layer| {
            let prev_layer = Box::new(next_layer);
            layer.next_operation = NextOperation::Layer(prev_layer);
            layer
        })
        .expect("For some reason no layers were created")
}


#[derive(Debug, Clone)]
pub struct Layer<S: Sender> {
    checks: Vec<CheckObject>,
    split: u64,
    next_operation: NextOperation<S>,
}

impl<S: Sender> Layer<S> {
    fn new(lower_bits: u64, mut checks: Vec<CheckObject>) -> Self {
        let split: u64 = 1 << (lower_bits.saturating_sub(1));
        while checks.len() % 8 != 0 {
          checks.push(CheckObject::filler())
        }
        Self {
            checks,
            split,
            next_operation: NextOperation::None,
        }
    }

    #[inline(always)]
    pub fn run_checks(&self, upper_bits: u64) {
        let mut chunks = self.checks.chunks_exact(8);
        while let Some(chunk) = chunks.next() {
            if chunk.iter().any(|check| check.check(upper_bits)) {
                return;
            }
        }

        use std::borrow::Borrow;
        match self.next_operation.borrow() {
            NextOperation::Layer(layer) => {
                layer.run_checks(upper_bits);
                layer.run_checks(upper_bits + self.split);
            }
            NextOperation::CrossComparison(checks) => checks.run(upper_bits),
            _ => {
                panic!("No operation")
            }
        }
    }
}

fn reverse_next_long(seed: u64) -> Vec<u64> {
    get_next_long(seed)
        .into_iter()
        .map(|seed| seed ^ JAVA_LCG.multiplier)
        .collect()
}

fn next_long(seed: u64) -> u64 {
    Random::with_seed(seed).next_long() as u64
}

impl<S: Sender> fmt::Debug for CrossComparison<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CrossComparison")
            .field("primary_hash", &self.primary_hash)
            .field("secondary_hash", &self.secondary_hash)
            .field("checks", &self.checks)
            .finish()
    }
}

impl<S: Sender> CrossComparison<S> {
    fn new(
        blocks: Vec<BlockFilter>,
        sender: S,
        is_floor_primary_filter: bool,
        output: OutputMode,
    ) -> CrossComparison<S> {
        let checks = blocks
            .into_iter()
            .map(|mut block| block.create_check(0))
            .collect();

        let (primary_hash, secondary_hash) = if is_floor_primary_filter {
            (FLOOR_HASH, ROOF_HASH)
        } else {
            (ROOF_HASH, FLOOR_HASH)
        };

        Self {
            sender,
            checks,
            primary_hash,
            secondary_hash,
            output,
        }
    }

    fn check(&self, seed: u64) -> bool {
        for check in self.checks.iter() {
            if check.check(seed) {
                return false;
            }
        }
        true
    }

    fn run(&self, seed: u64) {
        reverse_next_long(seed)
            .into_iter()
            .map(|seed| {
                // get common bedrock seed
                seed ^ self.primary_hash
            })
            .filter(|bedrock_seed| {
                // filter with blocks from the other surface
                let mut secondary_seed = bedrock_seed ^ self.secondary_hash;
                secondary_seed = next_long(secondary_seed);
                self.check(secondary_seed)
            })
            .flat_map(|bedrock_seed| {
                // reverse to world seed & mask48 aka structure seed
                reverse_next_long(bedrock_seed)
            })
            .for_each(|structure_seed| {
                if self.output == OutputMode::WorldSeed {
                    for prev_seed in reverse_next_long(structure_seed) {
                        let world_seed = next_long(prev_seed);
                        self.sender.send(CrackProgress::Seed(world_seed));
                    }
                } else {
                    self.sender.send(CrackProgress::Seed(structure_seed));
                }
            });
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use crate::MASK48;
    use crate::raw_data::block_type::BlockType;
    use super::*;

    const WORLD_SEED: u64 = 765906787396911863;
    const ROOF_SEED: u64 = 191924403737289;
    const FLOOR_SEED: u64 = 18240473916414;

    const BLOCKS: [Block; 24] = [
        Block::new(18, 123, -117, BlockType::OTHER),
        Block::new(18, 123, -118, BlockType::OTHER),
        Block::new(18, 123, -119, BlockType::OTHER),
        Block::new(33, 126, -99, BlockType::OTHER),
        Block::new(35, 126, -99, BlockType::OTHER),
        Block::new(38, 126, -99, BlockType::OTHER),
        Block::new(19, 123, -117, BlockType::BEDROCK),
        Block::new(19, 123, -118, BlockType::BEDROCK),
        Block::new(19, 123, -119, BlockType::BEDROCK),
        Block::new(25, 126, -112, BlockType::BEDROCK),
        Block::new(25, 126, -113, BlockType::BEDROCK),
        Block::new(25, 126, -114, BlockType::BEDROCK),
        Block::new(11, 1, -111, BlockType::OTHER),
        Block::new(11, 1, -110, BlockType::OTHER),
        Block::new(11, 1, -109, BlockType::OTHER),
        Block::new(14, 4, -97, BlockType::OTHER),
        Block::new(14, 4, -96, BlockType::OTHER),
        Block::new(14, 4, -94, BlockType::OTHER),
        Block::new(10, 1, -111, BlockType::BEDROCK),
        Block::new(10, 1, -110, BlockType::BEDROCK),
        Block::new(10, 1, -109, BlockType::BEDROCK),
        Block::new(11, 4, -97, BlockType::BEDROCK),
        Block::new(11, 4, -96, BlockType::BEDROCK),
        Block::new(11, 4, -94, BlockType::BEDROCK),
    ];

    #[test]
    fn test_world_seed_to_roof() {
        let rand = next_long(WORLD_SEED);
        let roof = next_long(rand ^ ROOF_HASH) & MASK48;
        let floor = next_long(rand ^ FLOOR_HASH) & MASK48;

        assert_eq!(ROOF_SEED, roof);
        assert_eq!(FLOOR_SEED, floor);
    }

    #[test]
    fn test_next_long_reverse() {
        for x in 1..10 {
            let y = next_long(x);
            assert!(reverse_next_long(y).contains(&x));
        }
    }

    #[test]
    fn test_checks() {
        BLOCKS
            .iter()
            .filter(|block| block.y > 5)
            .map(|block| BlockFilter::from(block, CrackerMode::Normal).create_check(10))
            .filter(|check| check.check(ROOF_SEED & 0xFFFF_FFFF_FC00))
            .for_each(|check| panic!("roof bedrock failed: {:#?}", check));

        BLOCKS
            .iter()
            .filter(|block| block.y < 5)
            .map(|block| BlockFilter::from(block, CrackerMode::Normal).create_check(10))
            .filter(|check| check.check(FLOOR_SEED & 0xFFFF_FFFF_FC00))
            .for_each(|check| panic!("floor bedrock failed: {:#?}", check));
    }

    #[test]
    fn test_filter_tree() {
        let (sender, receiver) = mpsc::channel();

        let layers = create_filter_tree(&BLOCKS, CrackerMode::Normal, OutputMode::WorldSeed, sender);

        // the cracker uses roof data as the primary filter if it has equal info from floor and roof
        layers.run_checks(ROOF_SEED & 0xFFFF_FFFF_F000);

        drop(layers);

        if let Ok(CrackProgress::Seed(WORLD_SEED)) = receiver.recv() {
        } else {
            panic!("No seed found")
        }
    }
}

