//! Calculate Dimensions Use Case Port
//!
//! Inbound port for individual dimension calculations.

use crate::calculations::{StubContext, Usefulness, VectorPhysics};
use crate::entities::{Audience, Stub};

/// Calculate dimensions use case trait
///
/// This is an inbound port for fine-grained dimension calculations.
/// Useful when you need individual calculations without full analysis.
pub trait CalculateDimensions {
    /// Calculate health score
    ///
    /// # Arguments
    /// * `refinement` - Refinement score (0.0-1.0)
    /// * `stubs` - List of stubs
    ///
    /// # Returns
    /// Health score (0.0-1.0)
    fn health(&self, refinement: f64, stubs: &[Stub]) -> f64;

    /// Calculate usefulness for an audience
    ///
    /// # Arguments
    /// * `refinement` - Refinement score (0.0-1.0)
    /// * `audience` - Target audience
    ///
    /// # Returns
    /// Usefulness assessment
    fn usefulness(&self, refinement: f64, audience: Audience) -> Usefulness;

    /// Calculate vector physics for a stub
    ///
    /// # Arguments
    /// * `stub` - Stub to analyze
    /// * `context` - Calculation context
    ///
    /// # Returns
    /// Vector physics (PE, friction, magnitude)
    fn vector_physics(&self, stub: &Stub, context: &StubContext) -> VectorPhysics;
}

/// Default implementation that delegates to calculation functions
pub struct DefaultCalculator;

impl CalculateDimensions for DefaultCalculator {
    fn health(&self, refinement: f64, stubs: &[Stub]) -> f64 {
        crate::calculations::calculate_health(refinement, stubs)
    }

    fn usefulness(&self, refinement: f64, audience: Audience) -> Usefulness {
        crate::calculations::calculate_usefulness(refinement, audience)
    }

    fn vector_physics(&self, stub: &Stub, context: &StubContext) -> VectorPhysics {
        VectorPhysics::calculate(stub, context)
    }
}
