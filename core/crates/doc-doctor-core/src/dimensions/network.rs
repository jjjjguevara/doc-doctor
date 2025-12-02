//! Network Dimensions
//!
//! Graph-based properties: position, propagation risk.

use serde::{Deserialize, Serialize};

/// Network dimensions (require graph context)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkDimensions {
    /// Network position (centrality measure)
    pub network_position: f64,

    /// Risk that changes propagate to dependents
    pub propagation_risk: f64,
}

impl NetworkDimensions {
    /// Placeholder: calculate with default values
    pub fn calculate_placeholder() -> Self {
        Self::default()
    }
}
