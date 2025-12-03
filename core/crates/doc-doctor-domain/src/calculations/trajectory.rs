//! Trajectory Dimensions
//!
//! Vector physics calculations for stub prioritization.
//! These are pure functions that calculate potential energy, friction, and magnitude.

use serde::{Deserialize, Serialize};
use crate::entities::Stub;

/// Context for vector physics calculations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StubContext {
    /// Editorial velocity (resolved per time unit)
    pub editorial_velocity: Option<f64>,

    /// Whether there are external dependencies
    pub has_external_dependencies: bool,

    /// Days since stub was created
    pub age_days: Option<u32>,

    /// Whether there's controversy (multiple perspectives)
    pub has_controversy: bool,
}

impl StubContext {
    /// Create a new default context
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder: set velocity
    pub fn with_velocity(mut self, velocity: f64) -> Self {
        self.editorial_velocity = Some(velocity);
        self
    }

    /// Builder: set external dependencies
    pub fn with_external_deps(mut self, has_deps: bool) -> Self {
        self.has_external_dependencies = has_deps;
        self
    }

    /// Builder: set controversy
    pub fn with_controversy(mut self, has_controversy: bool) -> Self {
        self.has_controversy = has_controversy;
        self
    }

    /// Builder: set age
    pub fn with_age(mut self, days: u32) -> Self {
        self.age_days = Some(days);
        self
    }
}

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
    ///
    /// # Arguments
    /// * `stub` - The stub to analyze
    /// * `context` - Additional context for calculations
    ///
    /// # Returns
    /// Vector physics with PE, friction, velocity, and magnitude
    pub fn calculate(stub: &Stub, context: &StubContext) -> Self {
        let potential_energy = calculate_potential_energy(stub);
        let friction_coefficient = calculate_friction(stub, context);
        let editorial_velocity = context.editorial_velocity.unwrap_or(0.0);
        let magnitude = calculate_magnitude(potential_energy, friction_coefficient);

        Self {
            potential_energy,
            friction_coefficient,
            editorial_velocity,
            magnitude,
        }
    }
}

/// Calculate potential energy: urgency × impact × complexity
///
/// # Arguments
/// * `stub` - The stub to analyze
///
/// # Returns
/// Potential energy value (typically 0.0-1.0)
pub fn calculate_potential_energy(stub: &Stub) -> f64 {
    let urgency = stub.effective_urgency();
    let impact = stub.effective_impact();
    let complexity = stub.effective_complexity();

    urgency * impact * complexity
}

/// Calculate friction coefficient
///
/// Formula: friction = controversy + dependency_count/10 + blocker_weight + external
///
/// # Arguments
/// * `stub` - The stub to analyze
/// * `context` - Additional context
///
/// # Returns
/// Friction coefficient between 0.0 and 1.0
pub fn calculate_friction(stub: &Stub, context: &StubContext) -> f64 {
    // Controversy from participants or context
    let controversy = if stub.participants.len() >= 2 || context.has_controversy {
        0.3
    } else {
        0.0
    };

    // Dependencies add friction
    let dependencies = (stub.dependencies.len() as f64 * 0.1).min(0.5);

    // Blocking stubs have additional friction
    let blocker = if stub.is_blocking() { 0.2 } else { 0.0 };

    // External dependencies add friction
    let external = if context.has_external_dependencies { 0.2 } else { 0.0 };

    (controversy + dependencies + blocker + external).min(1.0)
}

/// Calculate magnitude: sqrt(PE² + friction²)
///
/// This represents the overall "size" of the stub in editorial space.
///
/// # Arguments
/// * `potential_energy` - PE value
/// * `friction` - Friction coefficient
///
/// # Returns
/// Magnitude value
pub fn calculate_magnitude(potential_energy: f64, friction: f64) -> f64 {
    (potential_energy.powi(2) + friction.powi(2)).sqrt()
}

/// Forecast completion time
///
/// Formula: forecast = PE / (velocity × (1 - friction))
///
/// # Arguments
/// * `potential_energy` - PE value
/// * `velocity` - Editorial velocity
/// * `friction` - Friction coefficient
///
/// # Returns
/// Estimated time units, or None if velocity is zero
pub fn forecast_completion(potential_energy: f64, velocity: f64, friction: f64) -> Option<f64> {
    let denominator = velocity * (1.0 - friction);
    if denominator > 0.0 {
        Some(potential_energy / denominator)
    } else {
        None
    }
}

/// Trajectory dimensions (aggregate over multiple stubs)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrajectoryDimensions {
    /// Days since last update
    pub drift: f64,

    /// Health trend (-1.0 to 1.0)
    pub health_trend: f64,

    /// Adoption rate
    pub adoption: f64,

    /// Potential energy (aggregate of stubs)
    pub total_potential_energy: f64,

    /// Friction coefficient (aggregate)
    pub average_friction: f64,

    /// Editorial velocity (stubs resolved per time)
    pub editorial_velocity: f64,

    /// Refinement velocity (change per time)
    pub refinement_velocity: f64,
}

impl TrajectoryDimensions {
    /// Calculate aggregate trajectory dimensions for all stubs
    ///
    /// # Arguments
    /// * `stubs` - List of stubs
    /// * `context` - Shared context for calculations
    ///
    /// # Returns
    /// Aggregate trajectory dimensions
    pub fn calculate(stubs: &[Stub], context: &StubContext) -> Self {
        if stubs.is_empty() {
            return Self::default();
        }

        let physics: Vec<VectorPhysics> = stubs
            .iter()
            .map(|s| VectorPhysics::calculate(s, context))
            .collect();

        let total_pe: f64 = physics.iter().map(|p| p.potential_energy).sum();
        let avg_friction: f64 = physics.iter().map(|p| p.friction_coefficient).sum::<f64>()
            / physics.len() as f64;

        Self {
            drift: 0.0, // Requires historical data
            health_trend: 0.0, // Requires historical data
            adoption: 0.0, // Requires historical data
            total_potential_energy: total_pe,
            average_friction: avg_friction,
            editorial_velocity: context.editorial_velocity.unwrap_or(0.0),
            refinement_velocity: 0.0, // Requires historical data
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::StubForm;

    #[test]
    fn test_potential_energy() {
        let mut stub = Stub::compact("link", "test");
        stub.urgency = Some(0.8);
        stub.impact = Some(0.9);
        stub.complexity = Some(0.5);

        let pe = calculate_potential_energy(&stub);
        assert!((pe - 0.36).abs() < 0.01); // 0.8 * 0.9 * 0.5
    }

    #[test]
    fn test_potential_energy_defaults() {
        let stub = Stub::compact("link", "test");
        // Default priority = Low = 0.25 urgency
        // Default impact = 0.5
        // Default complexity = 0.5
        let pe = calculate_potential_energy(&stub);
        assert!((pe - 0.0625).abs() < 0.01); // 0.25 * 0.5 * 0.5
    }

    #[test]
    fn test_friction_with_participants() {
        let mut stub = Stub::compact("controversy", "test");
        stub.participants = vec!["Alice".into(), "Bob".into()];

        let ctx = StubContext::default();
        let friction = calculate_friction(&stub, &ctx);

        assert!(friction >= 0.3);
    }

    #[test]
    fn test_friction_with_context_controversy() {
        let stub = Stub::compact("link", "test");
        let ctx = StubContext::new().with_controversy(true);

        let friction = calculate_friction(&stub, &ctx);
        assert!(friction >= 0.3);
    }

    #[test]
    fn test_friction_with_blocking() {
        let mut stub = Stub::compact("link", "test");
        stub.stub_form = StubForm::Blocking;

        let ctx = StubContext::default();
        let friction = calculate_friction(&stub, &ctx);

        assert!(friction >= 0.2);
    }

    #[test]
    fn test_friction_with_dependencies() {
        let mut stub = Stub::compact("link", "test");
        stub.dependencies = vec!["dep1".into(), "dep2".into(), "dep3".into()];

        let ctx = StubContext::default();
        let friction = calculate_friction(&stub, &ctx);

        assert!(friction >= 0.3); // 3 deps * 0.1 = 0.3
    }

    #[test]
    fn test_friction_capped() {
        let mut stub = Stub::compact("link", "test");
        stub.participants = vec!["A".into(), "B".into()]; // 0.3
        stub.stub_form = StubForm::Blocking; // 0.2
        stub.dependencies = (0..10).map(|i| format!("dep{}", i)).collect(); // 0.5 (capped)

        let ctx = StubContext::new().with_external_deps(true); // 0.2

        let friction = calculate_friction(&stub, &ctx);
        assert_eq!(friction, 1.0); // Capped at 1.0
    }

    #[test]
    fn test_magnitude() {
        let mag = calculate_magnitude(0.6, 0.8);
        // sqrt(0.36 + 0.64) = sqrt(1.0) = 1.0
        assert!((mag - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_vector_physics() {
        let stub = Stub::compact("link", "test");
        let ctx = StubContext::default();
        let physics = VectorPhysics::calculate(&stub, &ctx);

        // magnitude = sqrt(PE² + FC²)
        let expected_mag = (physics.potential_energy.powi(2) + physics.friction_coefficient.powi(2)).sqrt();
        assert!((physics.magnitude - expected_mag).abs() < 0.001);
    }

    #[test]
    fn test_forecast_completion() {
        // With velocity and no friction
        let forecast = forecast_completion(0.5, 0.1, 0.0);
        assert_eq!(forecast, Some(5.0)); // 0.5 / (0.1 * 1.0)

        // With friction
        let forecast = forecast_completion(0.5, 0.1, 0.5);
        assert_eq!(forecast, Some(10.0)); // 0.5 / (0.1 * 0.5)

        // Zero velocity
        let forecast = forecast_completion(0.5, 0.0, 0.0);
        assert_eq!(forecast, None);

        // 100% friction
        let forecast = forecast_completion(0.5, 0.1, 1.0);
        assert_eq!(forecast, None);
    }

    #[test]
    fn test_trajectory_dimensions() {
        let stubs = vec![
            {
                let mut s = Stub::compact("link", "test1");
                s.urgency = Some(0.8);
                s
            },
            {
                let mut s = Stub::compact("expand", "test2");
                s.urgency = Some(0.6);
                s
            },
        ];

        let ctx = StubContext::new().with_velocity(0.5);
        let dims = TrajectoryDimensions::calculate(&stubs, &ctx);

        assert!(dims.total_potential_energy > 0.0);
        assert!(dims.average_friction >= 0.0);
        assert_eq!(dims.editorial_velocity, 0.5);
    }
}
