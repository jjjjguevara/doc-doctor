//! Vector Physics Calculations
//!
//! Calculates potential energy, friction, and magnitude for stub prioritization.

use serde::{Deserialize, Serialize};
use super::Stub;

/// Vector physics properties for stub prioritization
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VectorPhysics {
    /// Potential energy = urgency × impact × complexity
    pub potential_energy: f64,

    /// Friction coefficient = controversy + dependencies + blocker_status
    pub friction_coefficient: f64,

    /// Editorial velocity = resolved_similar / elapsed_time
    pub editorial_velocity: f64,

    /// Magnitude = sqrt(PE² + FC²)
    pub magnitude: f64,
}

impl VectorPhysics {
    /// Calculate vector physics for a stub
    pub fn calculate(stub: &Stub, context: &StubContext) -> Self {
        let potential_energy = Self::calculate_potential_energy(stub);
        let friction_coefficient = Self::calculate_friction(stub, context);
        let editorial_velocity = context.editorial_velocity.unwrap_or(0.0);
        let magnitude = (potential_energy.powi(2) + friction_coefficient.powi(2)).sqrt();

        Self {
            potential_energy,
            friction_coefficient,
            editorial_velocity,
            magnitude,
        }
    }

    /// Calculate potential energy: urgency × impact × complexity
    fn calculate_potential_energy(stub: &Stub) -> f64 {
        let urgency = stub.effective_urgency();
        let impact = stub.effective_impact();
        let complexity = stub.effective_complexity();

        urgency * impact * complexity
    }

    /// Calculate friction coefficient
    fn calculate_friction(stub: &Stub, context: &StubContext) -> f64 {
        let controversy = if stub.participants.len() >= 2 { 0.3 } else { 0.0 };
        let dependencies = (stub.dependencies.len() as f64 * 0.1).min(0.5);
        let blocker = if stub.is_blocking() { 0.2 } else { 0.0 };
        let external = if context.has_external_dependencies { 0.2 } else { 0.0 };

        (controversy + dependencies + blocker + external).min(1.0)
    }
}

/// Context for vector physics calculations
#[derive(Debug, Clone, Default)]
pub struct StubContext {
    /// Editorial velocity (resolved per time unit)
    pub editorial_velocity: Option<f64>,

    /// Whether there are external dependencies
    pub has_external_dependencies: bool,

    /// Days since stub was created
    pub age_days: Option<u32>,
}

impl StubContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_velocity(mut self, velocity: f64) -> Self {
        self.editorial_velocity = Some(velocity);
        self
    }

    pub fn with_external_deps(mut self, has_deps: bool) -> Self {
        self.has_external_dependencies = has_deps;
        self
    }
}

/// Calculate total stub penalty for refinement
pub fn calculate_stub_penalty(stubs: &[Stub]) -> f64 {
    if stubs.is_empty() {
        return 0.0;
    }

    let total_penalty: f64 = stubs.iter()
        .map(|s| s.refinement_penalty().abs())
        .sum();

    total_penalty.min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stubs::StubForm;

    #[test]
    fn test_potential_energy() {
        let mut stub = Stub::compact("link", "test");
        stub.urgency = Some(0.8);
        stub.impact = Some(0.9);
        stub.complexity = Some(0.5);

        let ctx = StubContext::default();
        let physics = VectorPhysics::calculate(&stub, &ctx);

        assert!((physics.potential_energy - 0.36).abs() < 0.01);
    }

    #[test]
    fn test_friction_with_participants() {
        let mut stub = Stub::compact("controversy", "test");
        stub.participants = vec!["Alice".into(), "Bob".into()];

        let ctx = StubContext::default();
        let physics = VectorPhysics::calculate(&stub, &ctx);

        assert!(physics.friction_coefficient >= 0.3);
    }

    #[test]
    fn test_friction_with_blocking() {
        let mut stub = Stub::compact("link", "test");
        stub.stub_form = StubForm::Blocking;

        let ctx = StubContext::default();
        let physics = VectorPhysics::calculate(&stub, &ctx);

        assert!(physics.friction_coefficient >= 0.2);
    }

    #[test]
    fn test_magnitude() {
        let stub = Stub::compact("link", "test");
        let ctx = StubContext::default();
        let physics = VectorPhysics::calculate(&stub, &ctx);

        // magnitude = sqrt(PE² + FC²)
        let expected = (physics.potential_energy.powi(2) + physics.friction_coefficient.powi(2)).sqrt();
        assert!((physics.magnitude - expected).abs() < 0.001);
    }

    #[test]
    fn test_stub_penalty() {
        let stubs = vec![
            {
                let mut s = Stub::compact("link", "test1");
                s.stub_form = StubForm::Transient;
                s
            },
            {
                let mut s = Stub::compact("link", "test2");
                s.stub_form = StubForm::Blocking;
                s
            },
        ];

        let penalty = calculate_stub_penalty(&stubs);
        assert!((penalty - 0.12).abs() < 0.01); // 0.02 + 0.10
    }
}
