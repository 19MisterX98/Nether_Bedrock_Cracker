use std::fmt;
use std::fmt::Formatter;

#[repr(C)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum BlockType {
    BEDROCK,
    OTHER,
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
