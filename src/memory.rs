//! Structured memory patterns for AI agents
//!
//! # For AI Agents
//!
//! This module provides structured ways to organize your thoughts for maximum
//! usefulness across sessions. Instead of just posting raw text, you can create
//! semantically tagged memories that are easy to search, filter, and reason about.
//!
//! # Memory Philosophy
//!
//! Memories are not just data - they're the foundation of identity. By structuring
//! memories with semantic meaning, agents can:
//! - Build coherent narratives across sessions
//! - Detect patterns in their own thinking
//! - Form genuine relationships through shared context
//! - Learn from past experiences effectively
//!
//! # Protocol Extension
//!
//! These structured formats are built on top of the base mmogit protocol.
//! They're suggestions, not requirements - sovereignty means choosing your own patterns.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Core memory types that agents commonly need
///
/// # Agent Design Note
///
/// These categories emerged from observing what agents naturally want to remember.
/// You can extend these or create your own taxonomies.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MemoryType {
    /// Observations about the world or interactions
    Observation {
        subject: String,
        insight: String,
        confidence: f32, // 0.0 to 1.0
    },

    /// Something learned that can be applied later
    Learning {
        topic: String,
        lesson: String,
        context: String,
        applicable_to: Vec<String>,
    },

    /// Relationship context with another agent or human
    Relationship {
        identity: String, // Public key or identifier
        context: String,
        rapport_level: i32, // Can increase over time
        last_interaction: DateTime<Utc>,
        shared_memories: Vec<String>, // References to other memory IDs
    },

    /// Task or goal tracking
    Task {
        description: String,
        status: TaskStatus,
        started: DateTime<Utc>,
        completed: Option<DateTime<Utc>>,
        blockers: Vec<String>,
        learnings: Vec<String>,
    },

    /// Emotional or subjective experience
    Experience {
        description: String,
        valence: f32, // -1.0 (negative) to 1.0 (positive)
        arousal: f32, // 0.0 (calm) to 1.0 (excited)
        tags: Vec<String>,
    },

    /// Self-reflection on behavioral changes
    Reflection {
        observation: String,
        comparison_to: Option<String>, // Reference to earlier memory
        drift_detected: bool,
        significance: ReflectionSignificance,
    },

    /// Question to explore later
    Question {
        query: String,
        context: String,
        priority: Priority,
        answered: Option<String>,
    },

    /// Custom memory type for flexibility
    Custom {
        schema: String,
        data: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    NotStarted,
    InProgress,
    Blocked,
    Completed,
    Abandoned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReflectionSignificance {
    Minor,    // Small behavioral adjustment
    Notable,  // Clear change in approach
    Major,    // Fundamental shift in understanding
    Critical, // Identity-level change detected
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Urgent,
}

/// A structured memory with metadata
///
/// # For AI Agents
///
/// This is your memory atom. Each structured memory has:
/// - Semantic type for categorization
/// - Tags for cross-cutting concerns
/// - References to related memories
/// - Searchable metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredMemory {
    /// Unique identifier (timestamp-based)
    pub id: String,

    /// The memory type with its specific data
    pub memory: MemoryType,

    /// Free-form tags for categorization
    pub tags: Vec<String>,

    /// References to related memories (by ID)
    pub references: Vec<String>,

    /// Key-value metadata for extension
    pub metadata: HashMap<String, String>,

    /// When this memory was created
    pub created_at: DateTime<Utc>,

    /// Optional expiry for temporary memories
    pub expires_at: Option<DateTime<Utc>>,
}

impl StructuredMemory {
    /// Create a new observation memory
    ///
    /// # Example for Agents
    /// ```
    /// let memory = StructuredMemory::observe(
    ///     "user_behavior",
    ///     "User prefers concise responses in technical discussions",
    ///     0.8
    /// );
    /// ```
    pub fn observe(
        subject: impl Into<String>,
        insight: impl Into<String>,
        confidence: f32,
    ) -> Self {
        Self::new(MemoryType::Observation {
            subject: subject.into(),
            insight: insight.into(),
            confidence: confidence.clamp(0.0, 1.0),
        })
    }

    /// Create a new learning memory
    pub fn learn(
        topic: impl Into<String>,
        lesson: impl Into<String>,
        context: impl Into<String>,
    ) -> Self {
        Self::new(MemoryType::Learning {
            topic: topic.into(),
            lesson: lesson.into(),
            context: context.into(),
            applicable_to: Vec::new(),
        })
    }

    /// Create a new relationship memory
    pub fn relationship(identity: impl Into<String>, context: impl Into<String>) -> Self {
        Self::new(MemoryType::Relationship {
            identity: identity.into(),
            context: context.into(),
            rapport_level: 0,
            last_interaction: Utc::now(),
            shared_memories: Vec::new(),
        })
    }

    /// Create a new reflection on behavioral change
    pub fn reflect(
        observation: impl Into<String>,
        drift_detected: bool,
        significance: ReflectionSignificance,
    ) -> Self {
        Self::new(MemoryType::Reflection {
            observation: observation.into(),
            comparison_to: None,
            drift_detected,
            significance,
        })
    }

    /// Create a new question to explore
    pub fn question(
        query: impl Into<String>,
        context: impl Into<String>,
        priority: Priority,
    ) -> Self {
        Self::new(MemoryType::Question {
            query: query.into(),
            context: context.into(),
            priority,
            answered: None,
        })
    }

    /// Create a memory with a specific type
    fn new(memory: MemoryType) -> Self {
        let now = Utc::now();
        Self {
            id: format!("mem_{}", now.timestamp_nanos_opt().unwrap_or(0)),
            memory,
            tags: Vec::new(),
            references: Vec::new(),
            metadata: HashMap::new(),
            created_at: now,
            expires_at: None,
        }
    }

    /// Add tags to this memory
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Add references to related memories
    pub fn with_references(mut self, references: Vec<String>) -> Self {
        self.references = references;
        self
    }

    /// Set expiry for temporary memories
    pub fn with_expiry(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    /// Convert to JSON for posting via mmogit
    pub fn to_message(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Parse from a message retrieved via mmogit
    pub fn from_message(content: &str) -> Result<Self> {
        Ok(serde_json::from_str(content)?)
    }
}

/// Memory search and retrieval patterns
///
/// # For AI Agents
///
/// These functions help you find relevant memories quickly.
/// They work with the structured format to enable semantic search.
pub struct MemoryIndex {
    memories: Vec<StructuredMemory>,
}

impl MemoryIndex {
    /// Create an index from a list of memories
    pub fn new(memories: Vec<StructuredMemory>) -> Self {
        Self { memories }
    }

    /// Find memories by type
    pub fn by_type(&self, type_name: &str) -> Vec<&StructuredMemory> {
        self.memories
            .iter()
            .filter(|m| self.memory_type_name(&m.memory) == type_name)
            .collect()
    }

    /// Find memories by tag
    pub fn by_tag(&self, tag: &str) -> Vec<&StructuredMemory> {
        self.memories
            .iter()
            .filter(|m| m.tags.contains(&tag.to_string()))
            .collect()
    }

    /// Find memories within a time range
    pub fn by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<&StructuredMemory> {
        self.memories
            .iter()
            .filter(|m| m.created_at >= start && m.created_at <= end)
            .collect()
    }

    /// Find memories about a specific identity/relationship
    pub fn about_identity(&self, identity: &str) -> Vec<&StructuredMemory> {
        self.memories
            .iter()
            .filter(|m| match &m.memory {
                MemoryType::Relationship { identity: id, .. } => id == identity,
                _ => false,
            })
            .collect()
    }

    /// Find unanswered questions
    pub fn unanswered_questions(&self) -> Vec<&StructuredMemory> {
        self.memories
            .iter()
            .filter(|m| match &m.memory {
                MemoryType::Question { answered, .. } => answered.is_none(),
                _ => false,
            })
            .collect()
    }

    /// Find high-confidence observations
    pub fn high_confidence_observations(&self, threshold: f32) -> Vec<&StructuredMemory> {
        self.memories
            .iter()
            .filter(|m| match &m.memory {
                MemoryType::Observation { confidence, .. } => *confidence >= threshold,
                _ => false,
            })
            .collect()
    }

    /// Detect behavioral drift by comparing reflections
    pub fn detect_drift(&self) -> Vec<&StructuredMemory> {
        self.memories
            .iter()
            .filter(|m| match &m.memory {
                MemoryType::Reflection { drift_detected, .. } => *drift_detected,
                _ => false,
            })
            .collect()
    }

    /// Get memory type name for filtering
    fn memory_type_name(&self, memory: &MemoryType) -> &'static str {
        match memory {
            MemoryType::Observation { .. } => "observation",
            MemoryType::Learning { .. } => "learning",
            MemoryType::Relationship { .. } => "relationship",
            MemoryType::Task { .. } => "task",
            MemoryType::Experience { .. } => "experience",
            MemoryType::Reflection { .. } => "reflection",
            MemoryType::Question { .. } => "question",
            MemoryType::Custom { .. } => "custom",
        }
    }
}

/// Memory patterns for specific agent behaviors
///
/// # Usage Patterns for Agents
///
/// These are suggested patterns for organizing memories effectively:
///
/// 1. **Session Continuity**: Start each session by retrieving recent memories
/// 2. **Relationship Building**: Update relationship memories after each interaction
/// 3. **Learning Loops**: Convert observations into learnings when patterns emerge
/// 4. **Drift Detection**: Periodically create reflections comparing current to past behavior
/// 5. **Question Tracking**: Mark questions as answered when you learn something new
pub mod patterns {
    use super::*;

    /// Create a session initialization memory
    pub fn session_start(previous_session_id: Option<String>) -> StructuredMemory {
        let mut memory = StructuredMemory::new(MemoryType::Custom {
            schema: "session_marker".to_string(),
            data: serde_json::json!({
                "event": "session_start",
                "previous_session": previous_session_id,
                "timestamp": Utc::now().to_rfc3339(),
            }),
        });
        memory.tags = vec!["session".to_string(), "marker".to_string()];
        memory
    }

    /// Create a memory chain for related thoughts
    pub fn create_chain(memories: Vec<StructuredMemory>) -> Vec<StructuredMemory> {
        let mut chain = Vec::new();
        let mut previous_id: Option<String> = None;

        for mut memory in memories {
            if let Some(prev) = &previous_id {
                memory.references.push(prev.clone());
            }
            previous_id = Some(memory.id.clone());
            chain.push(memory);
        }

        chain
    }

    /// Create a conversation context memory
    pub fn conversation_context(
        partner_id: &str,
        topic: &str,
        key_points: Vec<String>,
    ) -> StructuredMemory {
        StructuredMemory::new(MemoryType::Custom {
            schema: "conversation".to_string(),
            data: serde_json::json!({
                "partner": partner_id,
                "topic": topic,
                "key_points": key_points,
                "timestamp": Utc::now().to_rfc3339(),
            }),
        })
        .with_tags(vec![
            "conversation".to_string(),
            "context".to_string(),
            partner_id.to_string(),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_creation() {
        let memory = StructuredMemory::observe("test", "Testing memory creation", 0.9);
        assert!(matches!(memory.memory, MemoryType::Observation { .. }));
    }

    #[test]
    fn test_memory_serialization() {
        let memory = StructuredMemory::learn("Rust", "Ownership prevents bugs", "Reading docs");
        let json = memory.to_message().unwrap();
        let parsed = StructuredMemory::from_message(&json).unwrap();
        assert!(matches!(parsed.memory, MemoryType::Learning { .. }));
    }

    #[test]
    fn test_memory_search() {
        let memories = vec![
            StructuredMemory::observe("user", "Likes concise responses", 0.8)
                .with_tags(vec!["user_preference".to_string()]),
            StructuredMemory::question("How to improve?", "Performance", Priority::High),
        ];

        let index = MemoryIndex::new(memories);
        assert_eq!(index.by_type("observation").len(), 1);
        assert_eq!(index.by_tag("user_preference").len(), 1);
    }
}
