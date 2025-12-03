//! Refinement score type with validation
//!
//! Refinement represents document quality on a 0.0-1.0 scale.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use crate::errors::{DomainError, DomainResult};

/// Refinement score (0.0 to 1.0)
///
/// Represents document quality/completeness:
/// - 0.0: Stub/placeholder
/// - 0.5: Draft with substantial content
/// - 0.7: Review-ready
/// - 0.9: Publication-ready
/// - 1.0: Polished/finalized
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Refinement(f64);

impl Refinement {
    /// Minimum refinement value
    pub const MIN: f64 = 0.0;
    /// Maximum refinement value
    pub const MAX: f64 = 1.0;

    /// Create a new Refinement score with validation
    pub fn new(value: f64) -> DomainResult<Self> {
        if !(Self::MIN..=Self::MAX).contains(&value) {
            return Err(DomainError::RefinementOutOfRange { value });
        }
        Ok(Self(value))
    }

    /// Create a Refinement without validation (clamped to valid range)
    ///
    /// Values outside the valid range are clamped to 0.0-1.0.
    pub fn new_clamped(value: f64) -> Self {
        Self(value.clamp(Self::MIN, Self::MAX))
    }

    /// Get the raw f64 value
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Get a descriptive label for this refinement level
    pub fn label(&self) -> &'static str {
        match self.0 {
            v if v >= 0.90 => "Excellent",
            v if v >= 0.70 => "Good",
            v if v >= 0.50 => "Moderate",
            v if v >= 0.30 => "Weak",
            _ => "Poor",
        }
    }

    /// Check if this refinement meets a threshold
    pub fn meets_threshold(&self, threshold: f64) -> bool {
        self.0 >= threshold
    }
}

impl TryFrom<f64> for Refinement {
    type Error = DomainError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Refinement> for f64 {
    fn from(r: Refinement) -> f64 {
        r.0
    }
}

impl std::fmt::Display for Refinement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

impl Serialize for Refinement {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f64(self.0)
    }
}

impl<'de> Deserialize<'de> for Refinement {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = f64::deserialize(deserializer)?;
        Refinement::new(value).map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_refinement() {
        assert!(Refinement::new(0.0).is_ok());
        assert!(Refinement::new(0.5).is_ok());
        assert!(Refinement::new(1.0).is_ok());
    }

    #[test]
    fn test_invalid_refinement() {
        assert!(Refinement::new(-0.1).is_err());
        assert!(Refinement::new(1.1).is_err());
    }

    #[test]
    fn test_clamped() {
        assert_eq!(Refinement::new_clamped(-0.5).value(), 0.0);
        assert_eq!(Refinement::new_clamped(1.5).value(), 1.0);
        assert_eq!(Refinement::new_clamped(0.5).value(), 0.5);
    }

    #[test]
    fn test_labels() {
        assert_eq!(Refinement::new_clamped(0.95).label(), "Excellent");
        assert_eq!(Refinement::new_clamped(0.75).label(), "Good");
        assert_eq!(Refinement::new_clamped(0.55).label(), "Moderate");
        assert_eq!(Refinement::new_clamped(0.35).label(), "Weak");
        assert_eq!(Refinement::new_clamped(0.15).label(), "Poor");
    }

    #[test]
    fn test_serialization() {
        let r = Refinement::new_clamped(0.75);
        let json = serde_json::to_string(&r).unwrap();
        assert_eq!(json, "0.75");

        let parsed: Refinement = serde_json::from_str("0.80").unwrap();
        assert_eq!(parsed.value(), 0.80);
    }

    #[test]
    fn test_invalid_deserialization() {
        let result: Result<Refinement, _> = serde_json::from_str("1.5");
        assert!(result.is_err());
    }
}
