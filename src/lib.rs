use java_random::{JAVA_LCG, Random};
use next_long_reverser::get_next_long;

const MASK48: u64 = 0xffff_ffff_ffff;

pub enum BlockType {
    BEDROCK,
    OTHER,
}

pub struct Block {
    pos_hash: u64,
    lower_bound: u64,
    upper_bound: u64,
    valid_range: u64,
    possible_range: u64,
}

impl Block {

    fn create_check(&mut self, lower_bits: u64) -> CheckObject {
        let lower_bits_mask = (1 << lower_bits) - 1;
        self.check_with_bits(lower_bits);
        CheckObject::new(self.pos_hash, self.lower_bound, self.upper_bound, lower_bits_mask)
    }

    /*
    Figure out how many seeds an operation filters
     */
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

    pub fn new(x: i32, y: i32, z: i32, block_type: BlockType) -> Block {

        let pos_hash = Block::hashcode(x,y,z) ^ JAVA_LCG.multiplier;
        let (lower_bound, upper_bound) = Block::bounds(y, block_type);
        let valid_range = upper_bound - lower_bound;
        println!("Block: {x}, {y}, {z}, {pos_hash}");

        Block { pos_hash, lower_bound, upper_bound, valid_range, possible_range: MASK48 }
    }

    fn hashcode(x: i32, y: i32, z: i32) -> u64 {
        let mut pos_hash = (x.wrapping_mul(3129871)) as i64 ^ ((z as i64).wrapping_mul(116129781)) ^ y as i64;
        pos_hash = pos_hash.wrapping_mul(pos_hash).wrapping_mul(42317861).wrapping_add(pos_hash.wrapping_mul(11));
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

        println!("lower_bound: {lower_bound}, upper_bound: {upper_bound}");

        (lower_bound as u64, upper_bound as u64)
    }
}

pub fn create_filter_tree(blocks: &mut Vec<Block>) -> Layer {
    //sort everything by filter power
    //wanted to try functional programming
    let layers: Vec<Layer> = (0..=12)
        .rev()
        .map(|bits| {
            let mut checks = blocks.iter_mut()
                .map(|block| (block.discarded_seeds(bits), block))
                .filter(|(discarded_seeds, _)| *discarded_seeds > 0.0)
                .collect::<Vec<(f64, &mut Block)>>();

            checks.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());

            let res = checks.into_iter()
                .map(|(_ ,a)| a.create_check(bits))
                .collect();

            Layer::new(bits, res)
        })
        .collect();
    layers.into_iter()
        .rev()
        .reduce(|next_layer, mut layer| {
            let prev_layer = Box::new(next_layer);
            layer.next_layer = Some(prev_layer);
            layer
        })
        .expect("For some reason no layers were created")
}

#[derive(Debug)]
pub struct CheckObject {
    pos_hash: u64,
    condition: u64,
    offset: u64,
}

impl CheckObject {

    fn check(&self, upper_bits: u64) -> bool {
        ((upper_bits ^ self.pos_hash).wrapping_mul(JAVA_LCG.multiplier).wrapping_add(self.offset) & MASK48) < self.condition
    }

    pub fn new(pos_hash: u64, lower_bound: u64, upper_bound: u64, lower_bit_mask: u64) -> CheckObject {
        let offset = MASK48 - upper_bound;
        let pos_hash = pos_hash & (MASK48 - lower_bit_mask);
        let condition = lower_bound.wrapping_add(offset).wrapping_sub(lower_bit_mask * JAVA_LCG.multiplier);
        CheckObject { pos_hash, condition, offset }
    }
}


#[derive(Debug)]
pub struct Layer {
    pub checks: Vec<CheckObject>,
    pub split: u64,
    pub next_layer: Option<Box<Layer>>,
}

impl Layer {
    pub fn new(lower_bits: u64, checks: Vec<CheckObject>) -> Layer {
        let split: u64 = 1 << (lower_bits.saturating_sub(1));
        Layer {checks, split, next_layer: None}
    }

    pub fn run_checks(&self, upper_bits: u64) {
        //println!("run:");
        for check in self.checks.iter() {
            if check.check(upper_bits) {
                //println!("check failed: {upper_bits} {:#?}",check);
                return;
            }
            //println!("success");
        }
        match self.next_layer.as_ref() {
            Some(layer) => {
                layer.run_checks(upper_bits);
                layer.run_checks(upper_bits + self.split);
            },
            None => {
                let chance = upper_bits as f64 / MASK48 as f64;
                println!("Found Seed: {} percent: {}", upper_bits, chance)
            },
        }
    }
}

fn recover_world_seeds(seed: u64, is_floor: bool) -> Vec<u64> {
    let mut hashcode = 343340730; //roof
    if is_floor {
        hashcode = 2042456806; //floor
    }

    get_next_long(seed).into_iter()
        .flat_map(|mut seed| {
            seed ^= JAVA_LCG.multiplier;
            seed ^= hashcode;
            get_next_long(seed).into_iter()
        })
        .map(|mut seed| {
            seed ^= JAVA_LCG.multiplier;
            seed & MASK48
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::BlockType::BEDROCK;
    use super::*;

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
        assert_eq!(block.pos_hash, 99261249361405)
    }

    #[test]
    fn test_seed_conversion() {
        let roof_seed = 1442567685227760047 & MASK48;
        let seed = 5354280283422356689 & MASK48;
        assert!(recover_world_seeds(roof_seed, false).contains(&seed));
    }

    #[test]
    fn test_world_seed_to_roof() {
        let seed = 5354280283422356689 & MASK48;
        let roof_seed = 1442567685227760047 & MASK48;

        let mut rand = Random::with_seed(seed);
        let mut lon = rand.next_long() as u64& MASK48;
        lon = lon ^ 343340730;
        rand.set_seed(lon);
        lon = rand.next_long() as u64 & MASK48;

        assert_eq!(lon, roof_seed);
    }

    #[test]
    fn test_bedrock_matches() {
        let mut seed = (9210758467792927021 ^ JAVA_LCG.multiplier) & MASK48;
        println!("{seed}");
        seed = seed & 0xFFFF_FFFF_F000;
        let mut blocks = Vec::new();
        blocks.push(Block::new(-98, 4, -469, BEDROCK));
        blocks.push(Block::new(-101, 4, -465, BEDROCK));
        blocks.push(Block::new(-101, 4, -463, BEDROCK));
        blocks.push(Block::new(-101, 4, -457, BEDROCK));
        blocks.push(Block::new(-101, 4, -453, BEDROCK));
        blocks.push(Block::new(-100, 4, -456, BEDROCK));
        blocks.push(Block::new(-100, 4, -449, BEDROCK));
        blocks.push(Block::new(-99, 4, -464, BEDROCK));
        blocks.push(Block::new(-99, 4, -459, BEDROCK));
        blocks.push(Block::new(-99, 4, -455, BEDROCK));
        blocks.push(Block::new(-98, 4, -461, BEDROCK));
        blocks.push(Block::new(-98, 4, -460, BEDROCK));
        blocks.push(Block::new(-96, 4, -467, BEDROCK));
        blocks.push(Block::new(-96, 4, -465, BEDROCK));
        blocks.push(Block::new(-96, 4, -464, BEDROCK));
        blocks.push(Block::new(-96, 4, -452, BEDROCK));
        blocks.push(Block::new(-95, 4, -465, BEDROCK));
        blocks.push(Block::new(-95, 4, -458, BEDROCK));
        blocks.push(Block::new(-95, 4, -449, BEDROCK));
        blocks.push(Block::new(-94, 4, -462, BEDROCK));
        blocks.push(Block::new(-94, 4, -459, BEDROCK));
        blocks.push(Block::new(-94, 4, -454, BEDROCK));
        blocks.push(Block::new(-93, 4, -467, BEDROCK));
        blocks.push(Block::new(-93, 4, -465, BEDROCK));
        blocks.push(Block::new(-93, 4, -463, BEDROCK));
        blocks.push(Block::new(-93, 4, -455, BEDROCK));
        blocks.push(Block::new(-92, 4, -468, BEDROCK));
        blocks.push(Block::new(-92, 4, -467, BEDROCK));

        blocks.iter_mut()
            .map(|block| block.create_check(12))
            .filter(|check| check.check(seed))
            .for_each(|check| {
                panic!("check failed: {:#?}", check)
            })
    }
}