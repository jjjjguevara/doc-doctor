//! Domain Errors
//!
//! Pure domain errors that don't depend on infrastructure concerns.

mod validation;

pub use validation::*;

use std::fmt;

/// Domain error type for validation and calculation errors
#[derive(Debug, Clone, PartialEq)]
pub enum DomainError {
    /// Refinement value out of valid range (0.0-1.0)
    RefinementOutOfRange { value: f64 },

    /// Unknown audience value
    UnknownAudience { value: String },

    /// Unknown stub form value
    UnknownStubForm { value: String },

    /// Unknown priority value
    UnknownPriority { value: String },

    /// Invalid calculation input
    InvalidCalculationInput { message: String },
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainError::RefinementOutOfRange { value } => {
                write!(f, "Refinement value {} is out of range (must be 0.0-1.0)", value)
            }
            DomainError::UnknownAudience { value } => {
                write!(f, "Unknown audience '{}'. Valid values: personal, internal, trusted, public", value)
            }
            DomainError::UnknownStubForm { value } => {
                write!(f, "Unknown stub form '{}'. Valid values: transient, persistent, blocking, structural", value)
            }
            DomainError::UnknownPriority { value } => {
                write!(f, "Unknown priority '{}'. Valid values: low, medium, high, critical", value)
            }
            DomainError::InvalidCalculationInput { message } => {
                write!(f, "Invalid calculation input: {}", message)
            }
        }
    }
}

impl std::error::Error for DomainError {}

/// Result type alias for domain operations
pub type DomainResult<T> = Result<T, DomainError>;
