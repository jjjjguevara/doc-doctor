//! Audience classification with associated quality gates
//!
//! Each audience level has an associated refinement gate that documents
//! must meet to be considered "useful" for that audience.

use serde::{Deserialize, Serialize};
use std::str::FromStr;
use crate::errors::{DomainError, DomainResult};

/// Audience classification for documents
///
/// Determines the minimum refinement threshold for document usefulness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Audience {
    /// Personal notes, drafts, scratch space
    /// Gate: 0.50
    #[default]
    Personal,

    /// Internal team documentation
    /// Gate: 0.70
    Internal,

    /// Trusted external parties (partners, contractors)
    /// Gate: 0.80
    Trusted,

    /// Public-facing content
    /// Gate: 0.90
    Public,
}

impl Audience {
    /// Get the refinement gate threshold for this audience
    ///
    /// Documents with refinement >= gate are considered "useful" for this audience.
    pub const fn gate(&self) -> f64 {
        match self {
            Audience::Personal => 0.50,
            Audience::Internal => 0.70,
            Audience::Trusted => 0.80,
            Audience::Public => 0.90,
        }
    }

    /// Get the display name for this audience
    pub const fn display_name(&self) -> &'static str {
        match self {
            Audience::Personal => "Personal",
            Audience::Internal => "Internal",
            Audience::Trusted => "Trusted",
            Audience::Public => "Public",
        }
    }

    /// Get all valid audience values
    pub const fn all() -> &'static [Audience] {
        &[
            Audience::Personal,
            Audience::Internal,
            Audience::Trusted,
            Audience::Public,
        ]
    }

    /// Get all valid audience names as strings
    pub fn all_names() -> Vec<&'static str> {
        vec!["personal", "internal", "trusted", "public"]
    }

    /// Check if a refinement score meets this audience's gate
    pub fn meets_gate(&self, refinement: f64) -> bool {
        refinement >= self.gate()
    }

    /// Calculate the usefulness margin (refinement - gate)
    pub fn usefulness_margin(&self, refinement: f64) -> f64 {
        refinement - self.gate()
    }

    /// Parse audience from string
    pub fn parse(s: &str) -> DomainResult<Self> {
        Self::from_str(s)
    }
}

impl FromStr for Audience {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "personal" => Ok(Audience::Personal),
            "internal" => Ok(Audience::Internal),
            "trusted" => Ok(Audience::Trusted),
            "public" => Ok(Audience::Public),
            _ => Err(DomainError::UnknownAudience {
                value: s.to_string(),
            }),
        }
    }
}

impl std::fmt::Display for Audience {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Audience::Personal => "personal",
            Audience::Internal => "internal",
            Audience::Trusted => "trusted",
            Audience::Public => "public",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audience_gates() {
        assert_eq!(Audience::Personal.gate(), 0.50);
        assert_eq!(Audience::Internal.gate(), 0.70);
        assert_eq!(Audience::Trusted.gate(), 0.80);
        assert_eq!(Audience::Public.gate(), 0.90);
    }

    #[test]
    fn test_meets_gate() {
        assert!(Audience::Personal.meets_gate(0.50));
        assert!(Audience::Personal.meets_gate(0.75));
        assert!(!Audience::Personal.meets_gate(0.49));

        assert!(Audience::Public.meets_gate(0.90));
        assert!(Audience::Public.meets_gate(1.0));
        assert!(!Audience::Public.meets_gate(0.89));
    }

    #[test]
    fn test_usefulness_margin() {
        assert!((Audience::Internal.usefulness_margin(0.80) - 0.10).abs() < 0.001);
        assert!((Audience::Internal.usefulness_margin(0.70) - 0.0).abs() < 0.001);
        assert!((Audience::Internal.usefulness_margin(0.60) - (-0.10)).abs() < 0.001);
    }

    #[test]
    fn test_from_str() {
        assert_eq!(Audience::from_str("personal").unwrap(), Audience::Personal);
        assert_eq!(Audience::from_str("INTERNAL").unwrap(), Audience::Internal);
        assert_eq!(Audience::from_str("Trusted").unwrap(), Audience::Trusted);
        assert!(Audience::from_str("unknown").is_err());
    }

    #[test]
    fn test_serialization() {
        let audience = Audience::Internal;
        let json = serde_json::to_string(&audience).unwrap();
        assert_eq!(json, r#""internal""#);

        let parsed: Audience = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, Audience::Internal);
    }
}
