//! Trajectory Dimensions
//!
//! Change patterns over time: drift, trends, velocity.

use serde::{Deserialize, Serialize};

/// Trajectory dimensions (require historical data)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrajectoryDimensions {
    /// Days since last update
    pub drift: f64,

    /// Health trend (-1.0 to 1.0)
    pub health_trend: f64,

    /// Adoption rate
    pub adoption: f64,

    /// Potential energy (aggregate of stubs)
    pub potential_energy: f64,

    /// Friction coefficient (aggregate)
    pub friction_coefficient: f64,

    /// Editorial velocity (stubs resolved per time)
    pub editorial_velocity: f64,

    /// Refinement velocity (change per time)
    pub refinement_velocity: f64,
}

impl TrajectoryDimensions {
    /// Placeholder: calculate with default/zero values
    pub fn calculate_placeholder() -> Self {
        Self::default()
    }
}
