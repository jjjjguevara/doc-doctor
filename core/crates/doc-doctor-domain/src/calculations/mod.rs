//! Domain Calculations
//!
//! J-Editorial L2 extrinsic dimensions - pure functions that
//! calculate document properties from L1 intrinsic data.

mod state;
mod trajectory;

pub use state::{
    calculate_health,
    calculate_health_with_config,
    calculate_stub_penalty,
    calculate_stub_penalty_with_config,
    calculate_usefulness,
    calculate_usefulness_with_config,
    calculate_freshness,
    calculate_trust,
    StateDimensions,
    Usefulness,
};

pub use trajectory::{
    calculate_potential_energy,
    calculate_friction,
    calculate_magnitude,
    forecast_completion,
    StubContext,
    TrajectoryDimensions,
    VectorPhysics,
};
