//! State Dimensions
//!
//! Current state assessments: health, usefulness, compliance, trust, freshness, coverage.

use serde::{Deserialize, Serialize};
use crate::types::{Audience, L1Properties};
use crate::stubs::{Stub, calculate_stub_penalty};

/// Calculate document health score
///
/// Formula: health = 0.7 * refinement + 0.3 * (1 - stub_penalty)
pub fn calculate_health(refinement: f64, stubs: &[Stub]) -> f64 {
    let stub_penalty = calculate_stub_penalty(stubs);
    let health = 0.7 * refinement + 0.3 * (1.0 - stub_penalty);
    health.clamp(0.0, 1.0)
}

/// Usefulness assessment result
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Usefulness {
    /// Margin above/below gate (refinement - gate)
    pub margin: f64,

    /// Whether document meets the audience gate
    pub is_useful: bool,

    /// The audience being evaluated against
    pub audience: Audience,

    /// The document's refinement score
    pub refinement: f64,

    /// The audience's gate threshold
    pub gate: f64,
}

/// Calculate usefulness for a given audience
///
/// Formula: margin = refinement - audience_gate
pub fn calculate_usefulness(refinement: f64, audience: Audience) -> Usefulness {
    let gate = audience.gate();
    let margin = refinement - gate;

    Usefulness {
        margin,
        is_useful: margin >= 0.0,
        audience,
        refinement,
        gate,
    }
}

/// All state dimensions for a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDimensions {
    /// Health score (0.0-1.0)
    pub health: f64,

    /// Usefulness assessment
    pub usefulness: Usefulness,

    /// Compliance fit (placeholder, needs context)
    pub compliance_fit: f64,

    /// Trust level (placeholder, needs context)
    pub trust_level: f64,

    /// Freshness (1.0 = just updated, decays over time)
    pub freshness: f64,

    /// Coverage fit (placeholder, needs context)
    pub coverage_fit: f64,
}

impl StateDimensions {
    /// Calculate all state dimensions from L1 properties
    pub fn calculate(props: &L1Properties) -> Self {
        let health = calculate_health(props.refinement.value(), &props.stubs);
        let usefulness = calculate_usefulness(props.refinement.value(), props.audience);

        Self {
            health,
            usefulness,
            // Placeholder values - these need external context
            compliance_fit: 1.0,
            trust_level: match props.origin {
                // High trust: direct experience or requirements
                crate::types::Origin::Requirement => 0.95,
                crate::types::Origin::Insight => 0.9,
                crate::types::Origin::Dialogue => 0.85,
                // Medium trust: exploratory or derived
                crate::types::Origin::Question => 0.8,
                crate::types::Origin::Curiosity => 0.75,
                crate::types::Origin::Derivative => 0.7,
                // Lower trust: experimental/unverified
                crate::types::Origin::Experimental => 0.6,
            },
            freshness: 1.0, // Would need modified date to calculate
            coverage_fit: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stubs::StubForm;
    use crate::types::Refinement;

    #[test]
    fn test_calculate_health_no_stubs() {
        let health = calculate_health(0.8, &[]);
        // health = 0.7 * 0.8 + 0.3 * 1.0 = 0.56 + 0.3 = 0.86
        assert!((health - 0.86).abs() < 0.01);
    }

    #[test]
    fn test_calculate_health_with_stubs() {
        let stubs = vec![
            {
                let mut s = Stub::compact("link", "test");
                s.stub_form = StubForm::Transient;
                s
            },
        ];

        let health = calculate_health(0.8, &stubs);
        // stub_penalty = 0.02
        // health = 0.7 * 0.8 + 0.3 * (1 - 0.02) = 0.56 + 0.294 = 0.854
        assert!((health - 0.854).abs() < 0.01);
    }

    #[test]
    fn test_calculate_usefulness() {
        let usefulness = calculate_usefulness(0.75, Audience::Internal);
        assert!((usefulness.gate - 0.70).abs() < 0.001);
        assert!((usefulness.margin - 0.05).abs() < 0.001);
        assert!(usefulness.is_useful);

        let usefulness = calculate_usefulness(0.65, Audience::Internal);
        assert!(!usefulness.is_useful);
        assert!(usefulness.margin < 0.0);
    }

    #[test]
    fn test_state_dimensions() {
        let mut props = L1Properties::default();
        props.refinement = Refinement::new_unchecked(0.8);
        props.audience = Audience::Internal;

        let dims = StateDimensions::calculate(&props);
        assert!(dims.health > 0.8);
        assert!(dims.usefulness.is_useful);
    }
}
