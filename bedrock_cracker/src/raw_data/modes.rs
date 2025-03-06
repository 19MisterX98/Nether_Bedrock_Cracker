use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(C)]
pub enum BedrockGeneration {
    #[default]
    Normal,
    Paper1_18,
}

impl fmt::Display for BedrockGeneration {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BedrockGeneration::Normal => "Vanilla Generation",
                BedrockGeneration::Paper1_18 => "PaperMC < 1.19.2-213",
            }
        )
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(C)]
pub enum OutputMode {
    #[default]
    WorldSeed,
    StructureSeed,
}


impl fmt::Display for OutputMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                OutputMode::WorldSeed => "World Seeds",
                OutputMode::StructureSeed => "Structure Seeds",
            }
        )
    }
}
