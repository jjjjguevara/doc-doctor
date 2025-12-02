//! Stub Form - Severity/Permanence Classification
//!
//! Each stub has a form that indicates its expected lifecycle and impact.

use serde::{Deserialize, Serialize};
use std::str::FromStr;
use crate::error::DocDoctorError;

/// Stub form - severity and expected lifecycle
///
/// Forms determine how stubs impact refinement scores and workflows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StubForm {
    /// Minor issue, resolve soon
    /// Refinement penalty: -0.02
    #[default]
    Transient,

    /// Long-term gap, document anyway
    /// Refinement penalty: -0.05
    Persistent,

    /// Must resolve before progression
    /// Refinement penalty: -0.10
    Blocking,

    /// Fundamental architecture issue
    /// Refinement penalty: -0.15
    Structural,
}

impl StubForm {
    /// Get the refinement penalty for this form
    pub const fn refinement_penalty(&self) -> f64 {
        match self {
            StubForm::Transient => -0.02,
            StubForm::Persistent => -0.05,
            StubForm::Blocking => -0.10,
            StubForm::Structural => -0.15,
        }
    }

    /// Get the display name
    pub const fn display_name(&self) -> &'static str {
        match self {
            StubForm::Transient => "Transient",
            StubForm::Persistent => "Persistent",
            StubForm::Blocking => "Blocking",
            StubForm::Structural => "Structural",
        }
    }

    /// Get all valid forms
    pub const fn all() -> &'static [StubForm] {
        &[
            StubForm::Transient,
            StubForm::Persistent,
            StubForm::Blocking,
            StubForm::Structural,
        ]
    }

    /// Get all form names as strings
    pub fn all_names() -> Vec<&'static str> {
        vec!["transient", "persistent", "blocking", "structural"]
    }

    /// Check if this form blocks document progression
    pub const fn is_blocking(&self) -> bool {
        matches!(self, StubForm::Blocking | StubForm::Structural)
    }
}

impl FromStr for StubForm {
    type Err = DocDoctorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "transient" => Ok(StubForm::Transient),
            "persistent" => Ok(StubForm::Persistent),
            "blocking" => Ok(StubForm::Blocking),
            "structural" => Ok(StubForm::Structural),
            _ => Err(DocDoctorError::InvalidStubForm {
                form: s.to_string(),
                position: None,
            }),
        }
    }
}

impl std::fmt::Display for StubForm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            StubForm::Transient => "transient",
            StubForm::Persistent => "persistent",
            StubForm::Blocking => "blocking",
            StubForm::Structural => "structural",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_refinement_penalties() {
        assert_eq!(StubForm::Transient.refinement_penalty(), -0.02);
        assert_eq!(StubForm::Persistent.refinement_penalty(), -0.05);
        assert_eq!(StubForm::Blocking.refinement_penalty(), -0.10);
        assert_eq!(StubForm::Structural.refinement_penalty(), -0.15);
    }

    #[test]
    fn test_is_blocking() {
        assert!(!StubForm::Transient.is_blocking());
        assert!(!StubForm::Persistent.is_blocking());
        assert!(StubForm::Blocking.is_blocking());
        assert!(StubForm::Structural.is_blocking());
    }

    #[test]
    fn test_from_str() {
        assert_eq!(StubForm::from_str("transient").unwrap(), StubForm::Transient);
        assert_eq!(StubForm::from_str("BLOCKING").unwrap(), StubForm::Blocking);
        assert!(StubForm::from_str("unknown").is_err());
    }
}
