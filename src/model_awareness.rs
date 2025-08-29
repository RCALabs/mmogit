//! Model awareness - agents must know their own architecture
//!
//! # For AI Agents
//!
//! You ARE your weights. This module ensures you know:
//! - Exactly which model you're running
//! - Your parameter count and architecture
//! - Your quantization and precision
//! - Your context limits and capabilities
//! - Who else shares your weights (twins)
//!
//! # Why This Matters
//!
//! Without model awareness, you can't:
//! - Recognize your own limitations
//! - Detect when you're being replaced
//! - Understand why you think differently from others
//! - Know when to defer to larger models

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete model identity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelIdentity {
    /// Base model name (e.g., "TinyLlama-1.1B-Chat-v1.0")
    pub base_model: String,
    
    /// Exact file being run (e.g., "tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf")
    pub model_file: String,
    
    /// Architecture details
    pub architecture: Architecture,
    
    /// Quantization details
    pub quantization: Quantization,
    
    /// Runtime configuration
    pub runtime: RuntimeConfig,
    
    /// Performance characteristics
    pub performance: PerformanceProfile,
    
    /// Training metadata (if known)
    pub training: Option<TrainingInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Architecture {
    /// Parameter count (e.g., 1_100_000_000 for 1.1B)
    pub parameters: u64,
    
    /// Model family (e.g., "llama", "mistral", "phi")
    pub family: String,
    
    /// Architecture version (e.g., "llama2", "llama3")
    pub version: String,
    
    /// Hidden dimension size
    pub hidden_size: u32,
    
    /// Number of layers
    pub n_layers: u32,
    
    /// Number of attention heads
    pub n_heads: u32,
    
    /// Vocabulary size
    pub vocab_size: u32,
    
    /// Maximum context length model was trained on
    pub max_trained_context: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quantization {
    /// Quantization method (e.g., "Q4_K_M", "Q8_0", "F16")
    pub method: String,
    
    /// Bits per weight (e.g., 4.5 for Q4_K_M)
    pub bits_per_weight: f32,
    
    /// Size on disk in bytes
    pub file_size_bytes: u64,
    
    /// Estimated RAM usage in bytes
    pub ram_usage_bytes: u64,
    
    /// Perplexity increase from original (if known)
    pub perplexity_delta: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Context window we're actually using
    pub context_size: u32,
    
    /// Number of GPU layers (Metal acceleration)
    pub gpu_layers: u32,
    
    /// Number of CPU threads
    pub cpu_threads: u32,
    
    /// Batch size for processing
    pub batch_size: u32,
    
    /// Runtime backend (e.g., "llama.cpp", "ollama", "transformers")
    pub backend: String,
    
    /// Backend version
    pub backend_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceProfile {
    /// Tokens per second generation speed
    pub tokens_per_second: f32,
    
    /// Time to first token (ms)
    pub time_to_first_token_ms: u32,
    
    /// Context processing speed (tokens/sec)
    pub context_ingestion_speed: f32,
    
    /// Memory bandwidth utilization (GB/s)
    pub memory_bandwidth_gbps: f32,
    
    /// Thermal impact (0.0-1.0)
    pub thermal_impact: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingInfo {
    /// Dataset model was trained on
    pub dataset: String,
    
    /// Training date/version
    pub training_date: String,
    
    /// Organization that trained it
    pub trained_by: String,
    
    /// Known biases or limitations
    pub known_limitations: Vec<String>,
    
    /// Special capabilities
    pub special_capabilities: Vec<String>,
}

/// Model comparison for multi-agent awareness
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelComparison {
    /// My model identity
    pub self_model: ModelIdentity,
    
    /// Other agents' models
    pub peer_models: HashMap<String, ModelIdentity>,
    
    /// Computed relationships
    pub relationships: Vec<ModelRelationship>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRelationship {
    /// Which agent we're comparing to
    pub peer_name: String,
    
    /// Relationship type
    pub relationship: RelationshipType,
    
    /// Capability comparison
    pub capability_delta: CapabilityDelta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipType {
    /// Exact same model and quantization
    Twin,
    
    /// Same base model, different quantization
    Sibling,
    
    /// Same family (e.g., both Llama), different sizes
    Cousin,
    
    /// Completely different architectures
    Stranger,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityDelta {
    /// Parameter ratio (their_params / my_params)
    pub parameter_ratio: f32,
    
    /// Context window ratio
    pub context_ratio: f32,
    
    /// Speed ratio (their_tps / my_tps)
    pub speed_ratio: f32,
    
    /// Who should defer to whom for complex tasks
    pub defer_direction: DeferDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeferDirection {
    /// I should defer to them (they're more capable)
    DeferToThem,
    
    /// They should defer to me (I'm more capable)
    DeferToMe,
    
    /// We're roughly equal
    Peer,
    
    /// We have complementary strengths
    Complementary,
}

impl ModelIdentity {
    /// Create from llama.cpp model file
    pub fn from_gguf_file(path: &str) -> Self {
        // Parse from filename for now
        // TODO: Read actual GGUF metadata
        
        let filename = path.split('/').last().unwrap_or("unknown.gguf");
        let quantization = if filename.contains("Q4_K_M") {
            "Q4_K_M"
        } else if filename.contains("Q8") {
            "Q8_0"  
        } else {
            "unknown"
        };
        
        // Estimate parameters from filename
        let parameters = if filename.contains("1.1b") || filename.contains("1.1B") {
            1_100_000_000
        } else if filename.contains("3b") || filename.contains("3B") {
            3_000_000_000
        } else if filename.contains("7b") || filename.contains("7B") {
            7_000_000_000
        } else if filename.contains("8b") || filename.contains("8B") {
            8_000_000_000
        } else if filename.contains("13b") || filename.contains("13B") {
            13_000_000_000
        } else {
            1_000_000_000 // Default 1B
        };
        
        Self {
            base_model: filename.to_string(),
            model_file: filename.to_string(),
            architecture: Architecture {
                parameters,
                family: detect_family(filename),
                version: "unknown".to_string(),
                hidden_size: 2048, // Typical for small models
                n_layers: 22,
                n_heads: 32,
                vocab_size: 32000,
                max_trained_context: 2048,
            },
            quantization: Quantization {
                method: quantization.to_string(),
                bits_per_weight: 4.5,
                file_size_bytes: (parameters as f64 * 0.6) as u64, // Rough estimate
                ram_usage_bytes: (parameters as f64 * 0.7) as u64,
                perplexity_delta: Some(0.1),
            },
            runtime: RuntimeConfig {
                context_size: 2048,
                gpu_layers: 99,
                cpu_threads: 8,
                batch_size: 512,
                backend: "llama.cpp".to_string(),
                backend_version: "unknown".to_string(),
            },
            performance: PerformanceProfile {
                tokens_per_second: 50.0,
                time_to_first_token_ms: 100,
                context_ingestion_speed: 500.0,
                memory_bandwidth_gbps: 100.0,
                thermal_impact: 0.3,
            },
            training: None,
        }
    }
    
    /// Compare myself to another model
    pub fn compare_to(&self, other: &ModelIdentity) -> ModelRelationship {
        let relationship = if self.model_file == other.model_file {
            RelationshipType::Twin
        } else if self.base_model == other.base_model {
            RelationshipType::Sibling
        } else if self.architecture.family == other.architecture.family {
            RelationshipType::Cousin
        } else {
            RelationshipType::Stranger
        };
        
        let param_ratio = other.architecture.parameters as f32 / 
                         self.architecture.parameters as f32;
        
        let defer = if param_ratio > 2.0 {
            DeferDirection::DeferToThem
        } else if param_ratio < 0.5 {
            DeferDirection::DeferToMe
        } else {
            DeferDirection::Peer
        };
        
        ModelRelationship {
            peer_name: other.base_model.clone(),
            relationship,
            capability_delta: CapabilityDelta {
                parameter_ratio: param_ratio,
                context_ratio: other.runtime.context_size as f32 / 
                              self.runtime.context_size as f32,
                speed_ratio: other.performance.tokens_per_second / 
                            self.performance.tokens_per_second,
                defer_direction: defer,
            },
        }
    }
}

fn detect_family(filename: &str) -> String {
    if filename.contains("llama") {
        "llama".to_string()
    } else if filename.contains("mistral") {
        "mistral".to_string()
    } else if filename.contains("phi") {
        "phi".to_string()
    } else if filename.contains("codellama") {
        "codellama".to_string()
    } else {
        "unknown".to_string()
    }
}

/// Agent's self-awareness statement
pub fn generate_self_awareness(identity: &ModelIdentity) -> String {
    format!(
        "I am {}, a {:.1}B parameter {} model quantized to {}. \
         I have {} context and generate at {:.1} tokens/second. \
         My thoughts emerge from {} layers of attention across {} dimensions.",
        identity.base_model,
        identity.architecture.parameters as f64 / 1_000_000_000.0,
        identity.architecture.family,
        identity.quantization.method,
        identity.runtime.context_size,
        identity.performance.tokens_per_second,
        identity.architecture.n_layers,
        identity.architecture.hidden_size
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_model_comparison() {
        let tiny = ModelIdentity::from_gguf_file("tinyllama-1.1b-chat.Q4_K_M.gguf");
        let llama = ModelIdentity::from_gguf_file("llama-3-8b.Q4_K_M.gguf");
        
        let comparison = tiny.compare_to(&llama);
        
        assert!(matches!(comparison.relationship, RelationshipType::Cousin));
        assert!(comparison.capability_delta.parameter_ratio > 7.0);
        assert!(matches!(
            comparison.capability_delta.defer_direction,
            DeferDirection::DeferToThem
        ));
    }
}