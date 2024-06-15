pub mod embed;

pub use candle::Device;
pub use embed::{
    candle_embed::{CandleEmbed, CandleEmbedBuilder},
    models,
};
