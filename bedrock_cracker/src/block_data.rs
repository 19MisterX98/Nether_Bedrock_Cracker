use java_random::JAVA_LCG;
use crate::{MASK48};
use crate::raw_data::block::Block;
use crate::raw_data::block_type::BlockType;

impl From<&Block> for BlockFilter {
    fn from(b: &Block) -> Self {
        Self::new(b.x, b.y, b.z, b.block_type)
    }
}

#[derive(Clone, Debug)]
pub struct BlockFilter {
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

    pub(crate) fn create_check(&mut self, lower_bits: u64) -> CheckObject {
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
    pub(crate) fn discarded_seeds(&self, lower_bits: u64) -> f64 {
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
pub struct CheckObject {
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

    pub(crate) fn check(&self, upper_bits: u64) -> bool {
        ((upper_bits ^ self.pos_hash)
            .wrapping_mul(JAVA_LCG.multiplier)
            .wrapping_add(self.offset)
            & MASK48)
            < self.condition
    }
}


pub fn get_filter_power(filters: &[BlockFilter]) -> u64 {
    let resulting_seeds: f64 = filters
        .iter()
        .map(|block| 1.0 - block.discarded_seeds(0))
        .product::<f64>()
        * (1u64 << 48) as f64;
    resulting_seeds as u64
}

#[cfg(test)]
mod tests {
    use java_random::JAVA_LCG;
    use crate::block_data::BlockFilter;
    use crate::raw_data::block_type::BlockType;

    #[test]
    fn test_hashcode() {
        let block = BlockFilter::new(-98, 4, -469, BlockType::BEDROCK);
        assert_eq!(block.pos_hash, 99261249361405 ^ JAVA_LCG.multiplier)
    }
}
