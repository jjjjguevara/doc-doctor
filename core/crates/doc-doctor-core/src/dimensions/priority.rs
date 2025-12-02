//! Priority Dimensions
//!
//! Attention and value assessments.

use serde::{Deserialize, Serialize};

/// Priority dimensions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PriorityDimensions {
    /// How urgently this needs attention
    pub attention_priority: f64,

    /// Value of keeping/maintaining this document
    pub retention_value: f64,

    /// Estimated effort to improve to next level
    pub effort_to_improve: f64,
}

impl PriorityDimensions {
    /// Placeholder: calculate with default values
    pub fn calculate_placeholder() -> Self {
        Self::default()
    }
}
