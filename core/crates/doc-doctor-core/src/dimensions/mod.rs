//! L2 Extrinsic Dimension Calculations
//!
//! These dimensions are calculated on-demand from L1 properties.
//! They are context-dependent interpretations of document quality.

mod state;
mod trajectory;
mod network;
mod priority;

pub use state::{calculate_health, calculate_usefulness, Usefulness, StateDimensions};
pub use trajectory::TrajectoryDimensions;
pub use network::NetworkDimensions;
pub use priority::PriorityDimensions;
