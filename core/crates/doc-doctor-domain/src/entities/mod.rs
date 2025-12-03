//! Domain Entities
//!
//! J-Editorial L1 intrinsic properties - pure value objects
//! that represent document metadata without external dependencies.

mod audience;
mod document;
mod form;
mod origin;
mod refinement;
mod stub;

pub use audience::Audience;
pub use document::L1Properties;
pub use form::Form;
pub use origin::Origin;
pub use refinement::Refinement;
pub use stub::{Priority, Stub, StubForm, StubOrigin, StubType, VectorFamily};
