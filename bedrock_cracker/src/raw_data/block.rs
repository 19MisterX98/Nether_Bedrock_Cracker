use std::fmt;
use std::fmt::Formatter;
use crate::raw_data::block_type::BlockType;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Block {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub block_type: BlockType,
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