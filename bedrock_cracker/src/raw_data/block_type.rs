use std::fmt;
use std::fmt::Formatter;

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
