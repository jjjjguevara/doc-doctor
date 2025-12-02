//! L1 Intrinsic Property Types
//!
//! These types represent the core metadata stored in document frontmatter.
//! They are context-independent and portable across systems.

mod l1_intrinsic;
mod audience;
mod refinement;
mod validation;

pub use l1_intrinsic::L1Properties;
pub use audience::Audience;
pub use refinement::Refinement;
pub use validation::*;

// Re-export origin and form enums
pub use l1_intrinsic::{Origin, Form};
