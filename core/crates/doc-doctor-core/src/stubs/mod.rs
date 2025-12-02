//! Stub/Vector System
//!
//! Stubs are dynamic demand signals representing gaps in documents.
//! Each stub belongs to one of 5 vector families: Retrieval, Computation,
//! Synthesis, Creation, or Structural.

mod stub_type;
mod stub_form;
mod vector;
mod sync;

pub use stub_type::{Stub, StubType, Priority};
pub use stub_form::StubForm;
pub use vector::{VectorPhysics, StubContext, calculate_stub_penalty};
pub use sync::SyncStatus;
