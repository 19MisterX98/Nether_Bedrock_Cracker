const MASK48: u64 = (1<<48)-1;
const MULT: u64 = 25214903917;

pub enum BlockType {
    BEDROCK,
    OTHER,
}

pub struct Block {
    //store a lambda that checks
    pub pos_hash: u64,
    pub lower_bound: u64,
    pub upper_bound: u64,
    pub valid_range: u64,
    possible_range: u64,
}

impl Block {

    fn to_check(&self, lower_bits: u64) -> CheckObject {
        let lower_bits_mask = (1 << lower_bits) - 1;
        CheckObject::new(self.pos_hash, self.lower_bound, self.upper_bound, lower_bits_mask)
    }

    /*
    Figure out how many seeds an operation filters
     */
    pub fn discarded_seeds(&self, lower_bits: u64) -> f64 {
        let lower_bits_mask = (1 << lower_bits) - 1;
        let bound = self.bound() + lower_bits_mask * MULT;
        let success_chance = bound as f64 / self.possible_range as f64;
        let fail_chance = 1.0 - success_chance;
        if fail_chance <= 0.0 {
            return 0.0;
        }
        fail_chance * (1 << lower_bits) as f64
    }

    //the chance for new info decreases
    pub fn check_with_bits(&mut self, lower_bits: u64, checks: &mut Vec<CheckObject>) {
        let lower_bits_mask = (1 << lower_bits) - 1;
        let jiggle_room = lower_bits_mask * MULT;
        let new_range = jiggle_room + self.bound();
        assert!(new_range < self.possible_range);
        self.possible_range = new_range;
        checks.push(self.to_check(lower_bits));
    }

    fn bound(&self) -> u64 {
        assert!(self.upper_bound > self.lower_bound);
        self.upper_bound - self.lower_bound
    }

    pub fn new(x: i32, y: i32, z: i32, block_type: BlockType) -> Block {

        let pos_hash = Block::hashcode(x,y,z);
        let (lower_bound, upper_bound) = Block::bounds(y, block_type);
        let valid_range = upper_bound - lower_bound;

        Block { pos_hash, lower_bound, upper_bound, valid_range, possible_range: MASK48 }
    }

    fn hashcode(x: i32, y: i32, z: i32) -> u64 {
        let mut pos_hash = (x.wrapping_mul(3129871)) as i64 ^ ((z as i64).wrapping_mul(116129781)) ^ y as i64;
        pos_hash = pos_hash.wrapping_mul(pos_hash).wrapping_mul(42317861).wrapping_add(pos_hash.wrapping_mul(11));
        let pos_hash = pos_hash as u64;
        (pos_hash >> 16)
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

#[derive(Debug)]
pub struct CheckObject {
    pos_hash: u64,
    condition: u64,
    offset: u64,
}

impl CheckObject {

    fn check(&self, upper_bits: u64) -> bool {
        (upper_bits ^ self.pos_hash).wrapping_mul(MULT).wrapping_add(self.offset) & MASK48 > self.condition
    }

    pub fn new(pos_hash: u64, lower_bound: u64, upper_bound: u64, lower_bit_mask: u64) -> CheckObject {
        let offset = MASK48 - upper_bound;
        let condition = lower_bound.wrapping_add(offset).wrapping_sub(lower_bit_mask * MULT);
        CheckObject { pos_hash, condition, offset }
    }
}