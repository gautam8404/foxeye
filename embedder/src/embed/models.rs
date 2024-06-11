use std::fmt::{Display, Formatter, Write};

pub const UAE_LARGE_V1: &str = "WhereIsAI/UAE-Large-V1";
pub const BGE_LARGE_V1_5: &str = "BAAI/bge-large-en-v1.5";

#[derive(Debug, Clone, Default)]
pub enum Model {
    #[default]
    UaeLargeV1,
    BgeLargeV15,
}

impl Display for Model {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Model::UaeLargeV1 => f.write_str(UAE_LARGE_V1),
            Model::BgeLargeV15 => f.write_str(BGE_LARGE_V1_5),
        }
    }
}
