use std::cmp::min;
use std::fmt::Formatter;
use std::{borrow::Borrow, fmt, panic, thread};

use java_random::{Random, JAVA_LCG};
use next_long_reverser::get_next_long;

const MASK48: u64 = 0xFFFF_FFFF_FFFF;
const ROOF_HASH: u64 = 343340730;
const FLOOR_HASH: u64 = 2042456806;

const CHUNK_SIZE: u64 = (1 << 12) * (1 << 25); // interrupts every 2^25 seeds


#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum BlockType {
    BEDROCK,
    OTHER,
}

impl BlockType {
    pub const ALL: [BlockType; 2] = [BlockType::BEDROCK, BlockType::OTHER];
}

impl fmt::Display for BlockType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BlockType::BEDROCK => "Bedrock",
                BlockType::OTHER => "Other",
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Block {
    x: i32,
    y: i32,
    z: i32,
    block_type: BlockType,
}

impl Block {
    pub const fn new(x: i32, y: i32, z: i32, block_type: BlockType) -> Block {
        Self {
            x,
            y,
            z,
            block_type,
        }
    }

    fn split_floor_roof(blocks: &[Block]) -> (Vec<BlockFilter>, Vec<BlockFilter>) {
        let mut floor_blocks = vec![];
        let mut roof_blocks = vec![];

        for block in blocks.iter() {
            if block.y < 64 {
                floor_blocks.push(block.into());
            } else {
                roof_blocks.push(block.into());
            }
        }

        (floor_blocks, roof_blocks)
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.x, self.y, self.z, self.block_type
        )
    }
}

impl From<&Block> for BlockFilter {
    fn from(b: &Block) -> Self {
        Self::new(b.x, b.y, b.z, b.block_type)
    }
}

#[derive(Clone, Debug)]
struct BlockFilter {
    pos_hash: u64,
    lower_bound: u64,
    upper_bound: u64,
    possible_range: u64,
}

impl BlockFilter {
    fn new(x: i32, y: i32, z: i32, block_type: BlockType) -> Self {
        let pos_hash = BlockFilter::hashcode(x, y, z) ^ JAVA_LCG.multiplier;
        let (lower_bound, upper_bound) = Self::bounds(y, block_type);

        Self {
            pos_hash,
            lower_bound,
            upper_bound,
            possible_range: MASK48,
        }
    }

    fn create_check(&mut self, lower_bits: u64) -> CheckObject {
        let lower_bits_mask = (1 << lower_bits) - 1;
        self.check_with_bits(lower_bits);
        CheckObject::new(
            self.pos_hash,
            self.lower_bound,
            self.upper_bound,
            lower_bits_mask,
        )
    }

    //Figure out how many seeds an operation filters
    fn discarded_seeds(&self, lower_bits: u64) -> f64 {
        let lower_bits_mask = (1 << lower_bits) - 1;
        let bound = self.bound() + lower_bits_mask * JAVA_LCG.multiplier;
        let success_chance = bound as f64 / self.possible_range as f64;
        let fail_chance = 1.0 - success_chance;
        if fail_chance <= 0.0 {
            return 0.0;
        }
        fail_chance * (1 << lower_bits) as f64
    }

    //the chance for new info decreases
    fn check_with_bits(&mut self, lower_bits: u64) {
        let lower_bits_mask = (1 << lower_bits) - 1;
        let jiggle_room = lower_bits_mask * JAVA_LCG.multiplier;
        let new_range = jiggle_room + self.bound();
        assert!(new_range < self.possible_range);
        self.possible_range = new_range;
    }

    fn bound(&self) -> u64 {
        assert!(self.upper_bound > self.lower_bound);
        self.upper_bound - self.lower_bound
    }

    fn hashcode(x: i32, y: i32, z: i32) -> u64 {
        let mut pos_hash =
            (x.wrapping_mul(3129871)) as i64 ^ ((z as i64).wrapping_mul(116129781)) ^ y as i64;
        pos_hash = pos_hash
            .wrapping_mul(pos_hash)
            .wrapping_mul(42317861)
            .wrapping_add(pos_hash.wrapping_mul(11));
        let pos_hash = pos_hash as u64;
        pos_hash >> 16
    }

    fn bounds(mut layer: i32, block_type: BlockType) -> (u64, u64) {
        let mut lower_bound = 0.0;
        let mut upper_bound = 1.0;

        if layer > 5 {
            layer -= 122;
            let bound = (5 - layer) as f64 / 5.0;
            match block_type {
                BlockType::BEDROCK => lower_bound = bound,
                BlockType::OTHER => upper_bound = bound,
            }
        } else {
            let bound = (5 - layer) as f64 / 5.0;
            match block_type {
                BlockType::BEDROCK => upper_bound = bound,
                BlockType::OTHER => lower_bound = bound,
            }
        }

        lower_bound *= MASK48 as f64;
        upper_bound *= MASK48 as f64;

        //println!("lower_bound: {lower_bound}, upper_bound: {upper_bound}");

        (lower_bound as u64, upper_bound as u64)
    }
}

#[derive(Debug, Clone)]
struct CheckObject {
    pos_hash: u64,
    condition: u64,
    offset: u64,
}

impl CheckObject {
    fn new(pos_hash: u64, lower_bound: u64, upper_bound: u64, lower_bit_mask: u64) -> Self {
        let offset = MASK48 - upper_bound;
        let pos_hash = pos_hash & (MASK48 - lower_bit_mask);
        let condition = lower_bound
            .wrapping_add(offset)
            .wrapping_sub(lower_bit_mask * JAVA_LCG.multiplier);
        Self {
            pos_hash,
            condition,
            offset,
        }
    }

    fn check(&self, upper_bits: u64) -> bool {
        ((upper_bits ^ self.pos_hash)
            .wrapping_mul(JAVA_LCG.multiplier)
            .wrapping_add(self.offset)
            & MASK48)
            < self.condition
    }
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
}

impl<S: Sender> CrossComparison<S> {
    fn new(
        blocks: Vec<BlockFilter>,
        sender: S,
        is_floor_primary_filter: bool,
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
                // reverse to world seed & mask48
                reverse_next_long(bedrock_seed)
            })
            .flat_map(|world_seed_truncated| {
                // reverse again and then go forwards to get all bits
                reverse_next_long(world_seed_truncated)
            })
            .map(|seed| next_long(seed))
            .for_each(|world_seed| {
                // send results
                self.sender.send(CrackProgress::Seed(world_seed));
            });
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

#[derive(Debug, Clone)]
struct Layer<S: Sender> {
    checks: Vec<CheckObject>,
    split: u64,
    next_operation: NextOperation<S>,
}

impl<S: Sender> Layer<S> {
    fn new(lower_bits: u64, checks: Vec<CheckObject>) -> Self {
        let split: u64 = 1 << (lower_bits.saturating_sub(1));
        Self {
            checks,
            split,
            next_operation: NextOperation::None,
        }
    }

    fn run_checks(&self, upper_bits: u64) {
        if self.checks.iter().any(|check| check.check(upper_bits)) {
            return;
        }
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

/// this estimate is naive
pub fn estimate_result_amount(blocks: &[Block]) -> u64 {
    let filters: Vec<_> = blocks.iter().map(BlockFilter::from).collect();
    get_filter_power(&filters)
}

fn get_filter_power(filters: &[BlockFilter]) -> u64 {
    let resulting_seeds: f64 = filters
        .iter()
        .map(|block| 1.0 - block.discarded_seeds(0))
        .product::<f64>()
        * (1u64 << 48) as f64;
    resulting_seeds as u64
}

pub trait Sender: Clone + Send {
    fn send(&self, progress: CrackProgress) -> bool;
}

impl Sender for std::sync::mpsc::Sender<CrackProgress> {
    fn send(&self, progress: CrackProgress) -> bool {
        self.send(progress).is_ok()
    }
}

#[cfg(feature = "tokio")]
impl Sender for tokio::sync::mpsc::Sender<CrackProgress> {
    fn send(&self, progress: CrackProgress) -> bool {
        self.blocking_send(progress).is_ok()
    }
}

fn create_filter_tree<S: Sender>(blocks: &[Block], tx: S) -> Layer<S> {

    let (floor_blocks, roof_blocks) = Block::split_floor_roof(blocks);

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
    let final_check = CrossComparison::new(secondary_filter, tx, is_floor_primary_filter);
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

pub fn search_bedrock_pattern<S: Sender + 'static>(blocks: &[Block], thread_count: u64, sender: S) {
    let checks = create_filter_tree(blocks, sender.clone());

    for thread in 0..thread_count {
        let mut start_bits = (thread * (1 << 36)) / thread_count;
        let mut end_bits = ((thread + 1) * (1 << 36)) / thread_count;
        start_bits <<= 12;
        end_bits <<= 12;

        let checks = checks.clone();

        let sender = sender.clone();

        thread::spawn(move || {
            while start_bits < end_bits {
                let chunk_end = min(start_bits + CHUNK_SIZE, end_bits);
                for upper_bits in (start_bits..chunk_end).step_by(1 << 12) {
                    checks.run_checks(upper_bits);
                }
                //dropping the receiver stops the threads
                if !sender.send(CrackProgress::Progress(chunk_end - start_bits)) {
                    return;
                }

                start_bits = chunk_end;
            }
        });
    }
}

#[derive(Clone, Debug)]
pub enum CrackProgress {
    Seed(u64),
    Progress(u64),
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
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
    fn test_hashcode() {
        let block = BlockFilter::new(-98, 4, -469, BlockType::BEDROCK);
        assert_eq!(block.pos_hash, 99261249361405 ^ JAVA_LCG.multiplier)
    }

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
            .map(|block| BlockFilter::from(block).create_check(10))
            .filter(|check| check.check(ROOF_SEED & 0xFFFF_FFFF_FC00))
            .for_each(|check| panic!("roof bedrock failed: {:#?}", check));

        BLOCKS
            .iter()
            .filter(|block| block.y < 5)
            .map(|block| BlockFilter::from(block).create_check(10))
            .filter(|check| check.check(FLOOR_SEED & 0xFFFF_FFFF_FC00))
            .for_each(|check| panic!("floor bedrock failed: {:#?}", check));
    }

    #[test]
    fn test_filter_tree() {
        let (sender, receiver) = mpsc::channel();

        let layers = create_filter_tree(&BLOCKS, sender);

        // the cracker uses roof data as the primary filter if it has equal info from floor and roof
        layers.run_checks(ROOF_SEED & 0xFFFF_FFFF_F000);

        drop(layers);

        if let Ok(CrackProgress::Seed(WORLD_SEED)) = receiver.recv() {
        } else {
            panic!("No seed found")
        }
    }
}
