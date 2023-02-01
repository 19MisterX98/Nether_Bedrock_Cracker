use std::{
    borrow::Borrow,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use java_random::{Random, JAVA_LCG};
use next_long_reverser::get_next_long;

use crate::NextOperation::{LAYER, NONE, SEND};

const MASK48: u64 = 0xFFFF_FFFF_FFFF;

pub enum BlockType {
    BEDROCK,
    OTHER,
}

pub struct Block {
    pos_hash: u64,
    lower_bound: u64,
    upper_bound: u64,
    possible_range: u64,
}

impl Block {
    pub fn new(x: i32, y: i32, z: i32, block_type: BlockType) -> Self {
        let pos_hash = Block::hashcode(x, y, z) ^ JAVA_LCG.multiplier;
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
            match block_type {
                BlockType::BEDROCK => lower_bound = (5 - layer) as f64 / 5.0,
                BlockType::OTHER => upper_bound = layer as f64 / 5.0,
            }
        } else {
            match block_type {
                BlockType::BEDROCK => upper_bound = (5 - layer) as f64 / 5.0,
                BlockType::OTHER => lower_bound = layer as f64 / 5.0,
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
enum NextOperation {
    LAYER(Box<Layer>),
    SEND(Sender<u64>),
    NONE,
}

#[derive(Debug, Clone)]
struct Layer {
    checks: Vec<CheckObject>,
    split: u64,
    next_operation: NextOperation,
}

impl Layer {
    fn new(lower_bits: u64, checks: Vec<CheckObject>, tx: &Sender<u64>) -> Self {
        let split: u64 = 1 << (lower_bits.saturating_sub(1));
        let next_operation = if lower_bits == 0 {
            SEND(tx.clone())
        } else {
            NONE
        };
        Self {
            checks,
            split,
            next_operation,
        }
    }

    fn run_checks(&self, upper_bits: u64) {
        for check in &self.checks {
            if check.check(upper_bits) {
                return;
            }
        }
        match self.next_operation.borrow() {
            LAYER(layer) => {
                layer.run_checks(upper_bits);
                layer.run_checks(upper_bits + self.split);
            }
            SEND(tx) => {
                tx.send(upper_bits).unwrap();
            }
            _ => {
                panic!("No operation")
            }
        }
    }
}

fn create_filter_tree(blocks: &mut [Block], tx: &Sender<u64>) -> Layer {
    //sort everything by filter power
    //wanted to try functional programming
    let layers: Vec<Layer> = (0..=12)
        .rev()
        .map(|bits| {
            let mut checks = blocks
                .iter_mut()
                .map(|block| (block.discarded_seeds(bits), block))
                .filter(|(discarded_seeds, _)| *discarded_seeds > 0.0)
                .collect::<Vec<(f64, &mut Block)>>();

            checks.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());

            let res = checks
                .into_iter()
                .map(|(_, a)| a.create_check(bits))
                .collect();

            Layer::new(bits, res, tx)
        })
        .collect();
    layers
        .into_iter()
        .rev()
        .reduce(|next_layer, mut layer| {
            let prev_layer = Box::new(next_layer);
            layer.next_operation = LAYER(prev_layer);
            layer
        })
        .expect("For some reason no layers were created")
}

pub fn world_seeds_from_bedrock_seed(seed: u64, is_floor: bool) -> Vec<i64> {
    let hashcode = if is_floor { 2042456806 } else { 343340730 };

    get_next_long(seed)
        .into_iter()
        .flat_map(|mut seed| {
            seed ^= JAVA_LCG.multiplier;
            seed ^= hashcode;
            get_next_long(seed).into_iter()
        })
        .map(|seed| seed ^ JAVA_LCG.multiplier)
        // Neils get_next_long() masks the output so I go one step further and back
        .flat_map(|seed| get_next_long(seed))
        .map(|seed| Random::with_raw_seed(seed).next_long())
        .collect()
}

pub fn search_bedrock_pattern(blocks: &mut [Block], thread_count: u8) -> Receiver<u64> {
    let (tx, rx) = mpsc::channel();
    let checks = create_filter_tree(blocks, &tx);

    let thread_count = u64::from(thread_count);

    for thread in 0..thread_count {
        let mut start_bits = (thread * (1 << 36)) / thread_count;
        let mut end_bits = ((thread + 1) * (1 << 36)) / thread_count;
        start_bits <<= 12;
        end_bits <<= 12;

        let checks = checks.clone();

        thread::spawn(move || {
            for upper_bits in (start_bits..end_bits).step_by(1 << 12) {
                checks.run_checks(upper_bits);
            }
        });
    }

    rx
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BlockType::BEDROCK;
    use java_random::Random;

    /*
    upper_bits: 81699833426244
    lower_bits: 29905
    seed: 5354280283422356689
    floor: 9210758467792927021
    roof: 1442567685227760047
    */

    #[test]
    fn test_hashcode() {
        let block = Block::new(-98, 4, -469, BEDROCK);
        assert_eq!(block.pos_hash, 99261249361405 ^ JAVA_LCG.multiplier)
    }

    #[test]
    fn test_seed_conversion() {
        let roof_seed = 1442567685227760047 & MASK48;
        let seed = 5354280283422356689;
        assert!(world_seeds_from_bedrock_seed(roof_seed, false).contains(&seed));
    }

    #[test]
    fn test_world_seed_to_roof() {
        let seed = 5354280283422356689 & MASK48;
        let roof_seed = 1442567685227760047 & MASK48;

        let mut rand = Random::with_seed(seed);
        let mut lon = rand.next_long() as u64 & MASK48;
        lon = lon ^ 343340730;
        rand.set_seed(lon);
        lon = rand.next_long() as u64 & MASK48;

        assert_eq!(lon, roof_seed);
    }

    #[test]
    fn test_bedrock_matches() {
        let mut seed = 9210758467792927021 & MASK48;
        seed = seed & 0xFFFF_FFFF_F000;
        let mut blocks = vec![
            Block::new(-98, 4, -469, BEDROCK),
            Block::new(-101, 4, -465, BEDROCK),
            Block::new(-101, 4, -463, BEDROCK),
            Block::new(-101, 4, -457, BEDROCK),
            Block::new(-101, 4, -453, BEDROCK),
            Block::new(-100, 4, -456, BEDROCK),
            Block::new(-100, 4, -449, BEDROCK),
            Block::new(-99, 4, -464, BEDROCK),
            Block::new(-99, 4, -459, BEDROCK),
            Block::new(-99, 4, -455, BEDROCK),
            Block::new(-98, 4, -461, BEDROCK),
            Block::new(-98, 4, -460, BEDROCK),
            Block::new(-96, 4, -467, BEDROCK),
            Block::new(-96, 4, -465, BEDROCK),
            Block::new(-96, 4, -464, BEDROCK),
            Block::new(-96, 4, -452, BEDROCK),
            Block::new(-95, 4, -465, BEDROCK),
            Block::new(-95, 4, -458, BEDROCK),
            Block::new(-95, 4, -449, BEDROCK),
            Block::new(-94, 4, -462, BEDROCK),
            Block::new(-94, 4, -459, BEDROCK),
            Block::new(-94, 4, -454, BEDROCK),
            Block::new(-93, 4, -467, BEDROCK),
            Block::new(-93, 4, -465, BEDROCK),
            Block::new(-93, 4, -463, BEDROCK),
            Block::new(-93, 4, -455, BEDROCK),
            Block::new(-92, 4, -468, BEDROCK),
            Block::new(-92, 4, -467, BEDROCK),
        ];

        blocks
            .iter_mut()
            .map(|block| block.create_check(12))
            .filter(|check| check.check(seed))
            .for_each(|check| panic!("check failed: {:#?}", check))
    }
}
