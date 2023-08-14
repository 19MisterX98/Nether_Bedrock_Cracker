use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CrackerMode {
    #[default]
    Normal,
    Paper1_18,
}

impl CrackerMode {
    pub const ALL: [CrackerMode; 2] = [CrackerMode::Normal, CrackerMode::Paper1_18];
}

impl fmt::Display for CrackerMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CrackerMode::Normal => "Vanilla Generation",
                CrackerMode::Paper1_18 => "PaperMC < 1.19.2-213",
            }
        )
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OutputMode {
    #[default]
    WorldSeed,
    StructureSeed,
}


impl OutputMode {
    pub const ALL: [OutputMode; 2] = [OutputMode::WorldSeed, OutputMode::StructureSeed];
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
