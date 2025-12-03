//! Document Form
//!
//! The structural type and lifecycle stage of content.

use serde::{Deserialize, Serialize};

/// Document form - the structural type of content
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Form {
    /// Temporary/scratch content (staleness: 7 days)
    Transient,

    /// Work in progress (staleness: 30 days)
    #[default]
    Developing,

    /// Mature, stable content (staleness: 90 days)
    Stable,

    /// Long-lived reference content (staleness: 365 days)
    Evergreen,

    /// Authoritative, rarely changed (staleness: never)
    Canonical,
}

impl Form {
    /// Get the display name for this form
    pub const fn display_name(&self) -> &'static str {
        match self {
            Form::Transient => "Transient",
            Form::Developing => "Developing",
            Form::Stable => "Stable",
            Form::Evergreen => "Evergreen",
            Form::Canonical => "Canonical",
        }
    }

    /// Get the staleness cadence in days
    ///
    /// This is the half-life for freshness decay.
    pub const fn staleness_cadence_days(&self) -> f64 {
        match self {
            Form::Transient => 7.0,
            Form::Developing => 30.0,
            Form::Stable => 90.0,
            Form::Evergreen => 365.0,
            Form::Canonical => f64::INFINITY,
        }
    }

    /// Get all valid form values
    pub const fn all() -> &'static [Form] {
        &[
            Form::Transient,
            Form::Developing,
            Form::Stable,
            Form::Evergreen,
            Form::Canonical,
        ]
    }
}

impl std::fmt::Display for Form {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Form::Transient => "transient",
            Form::Developing => "developing",
            Form::Stable => "stable",
            Form::Evergreen => "evergreen",
            Form::Canonical => "canonical",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_form_display_name() {
        assert_eq!(Form::Transient.display_name(), "Transient");
        assert_eq!(Form::Developing.display_name(), "Developing");
        assert_eq!(Form::Canonical.display_name(), "Canonical");
    }

    #[test]
    fn test_staleness_cadence() {
        assert_eq!(Form::Transient.staleness_cadence_days(), 7.0);
        assert_eq!(Form::Developing.staleness_cadence_days(), 30.0);
        assert_eq!(Form::Stable.staleness_cadence_days(), 90.0);
        assert_eq!(Form::Evergreen.staleness_cadence_days(), 365.0);
        assert!(Form::Canonical.staleness_cadence_days().is_infinite());
    }

    #[test]
    fn test_serialization() {
        let form = Form::Stable;
        let json = serde_json::to_string(&form).unwrap();
        assert_eq!(json, r#""stable""#);

        let parsed: Form = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, Form::Stable);
    }
}
