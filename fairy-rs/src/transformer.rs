//! Full 2-bit complex transformer implementation

use crate::{FairyConfig, attention::ComplexAttention2Bit};
use num_complex::Complex32;
use ndarray::Array2;

/// A complete 2-bit transformer
pub struct FairyTransformer {
    pub config: FairyConfig,
    pub layers: Vec<TransformerLayer>,
}

pub struct TransformerLayer {
    pub attention: ComplexAttention2Bit,
    // TODO: Add FFN, LayerNorm
}

impl FairyTransformer {
    pub fn new(config: FairyConfig) -> Self {
        let mut layers = Vec::new();
        for _ in 0..config.n_layers {
            layers.push(TransformerLayer {
                attention: ComplexAttention2Bit::new(config.hidden_dim, config.n_heads),
            });
        }
        
        Self { config, layers }
    }
}