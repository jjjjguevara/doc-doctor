//! Calculation Configuration
//!
//! Configurable thresholds and weights for J-Editorial calculations.
//! Defaults follow the 80/20 power law - designed to work well for most use cases.
//!
//! # Layered Configuration
//!
//! Configuration is loaded in layers (later overrides earlier):
//! 1. Built-in defaults (this module)
//! 2. User config: `~/.config/doc-doctor/config.yaml`
//! 3. Project config: `.doc-doctor.yaml` in working directory
//! 4. CLI arguments (highest priority)

use serde::{Deserialize, Serialize};

/// Complete calculation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CalculationConfig {
    /// Schema version for forward compatibility
    pub version: u32,

    /// Health calculation weights
    pub health: HealthConfig,

    /// Audience gate thresholds
    pub audience_gates: AudienceGatesConfig,

    /// Stub penalty values by form
    pub stub_penalties: StubPenaltiesConfig,

    /// Trust factor values by origin
    pub trust_factors: TrustFactorsConfig,

    /// Form staleness cadences in days
    pub form_cadences: FormCadencesConfig,

    /// Vector physics defaults
    pub vector_physics: VectorPhysicsConfig,
}

impl Default for CalculationConfig {
    fn default() -> Self {
        Self {
            version: 1,
            health: HealthConfig::default(),
            audience_gates: AudienceGatesConfig::default(),
            stub_penalties: StubPenaltiesConfig::default(),
            trust_factors: TrustFactorsConfig::default(),
            form_cadences: FormCadencesConfig::default(),
            vector_physics: VectorPhysicsConfig::default(),
        }
    }
}

impl CalculationConfig {
    /// Create a new config with all defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate configuration values are within acceptable ranges
    pub fn validate(&self) -> Result<(), ConfigValidationError> {
        // Health weights must sum to 1.0
        let weight_sum = self.health.refinement_weight + self.health.stub_weight;
        if (weight_sum - 1.0).abs() > 0.001 {
            return Err(ConfigValidationError::InvalidWeights {
                field: "health".to_string(),
                message: format!(
                    "refinement_weight + stub_weight must equal 1.0, got {}",
                    weight_sum
                ),
            });
        }

        // Gates must be in ascending order
        if !(self.audience_gates.personal <= self.audience_gates.internal
            && self.audience_gates.internal <= self.audience_gates.trusted
            && self.audience_gates.trusted <= self.audience_gates.public)
        {
            return Err(ConfigValidationError::InvalidOrder {
                field: "audience_gates".to_string(),
                message: "Gates must be in ascending order: personal <= internal <= trusted <= public".to_string(),
            });
        }

        // All values must be 0.0-1.0
        self.validate_range("audience_gates.personal", self.audience_gates.personal)?;
        self.validate_range("audience_gates.internal", self.audience_gates.internal)?;
        self.validate_range("audience_gates.trusted", self.audience_gates.trusted)?;
        self.validate_range("audience_gates.public", self.audience_gates.public)?;

        self.validate_range("stub_penalties.transient", self.stub_penalties.transient)?;
        self.validate_range("stub_penalties.persistent", self.stub_penalties.persistent)?;
        self.validate_range("stub_penalties.blocking", self.stub_penalties.blocking)?;
        self.validate_range("stub_penalties.structural", self.stub_penalties.structural)?;

        Ok(())
    }

    fn validate_range(&self, field: &str, value: f64) -> Result<(), ConfigValidationError> {
        if !(0.0..=1.0).contains(&value) {
            return Err(ConfigValidationError::OutOfRange {
                field: field.to_string(),
                value,
                min: 0.0,
                max: 1.0,
            });
        }
        Ok(())
    }

    /// Merge another config on top of this one (other values override self)
    pub fn merge(&self, other: &CalculationConfig) -> CalculationConfig {
        // For simplicity, we do a full replacement at the section level
        // A more granular merge could be implemented if needed
        CalculationConfig {
            version: other.version.max(self.version),
            health: other.health.clone(),
            audience_gates: other.audience_gates.clone(),
            stub_penalties: other.stub_penalties.clone(),
            trust_factors: other.trust_factors.clone(),
            form_cadences: other.form_cadences.clone(),
            vector_physics: other.vector_physics.clone(),
        }
    }
}

/// Health calculation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HealthConfig {
    /// Weight for refinement component (default: 0.7)
    pub refinement_weight: f64,

    /// Weight for stub penalty component (default: 0.3)
    pub stub_weight: f64,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            refinement_weight: 0.7,
            stub_weight: 0.3,
        }
    }
}

/// Audience gate thresholds
///
/// Minimum refinement score required for a document to be "useful"
/// for each audience level.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AudienceGatesConfig {
    /// Personal use threshold (default: 0.50)
    pub personal: f64,

    /// Internal/team use threshold (default: 0.70)
    pub internal: f64,

    /// Trusted partners threshold (default: 0.80)
    pub trusted: f64,

    /// Public consumption threshold (default: 0.90)
    pub public: f64,
}

impl Default for AudienceGatesConfig {
    fn default() -> Self {
        Self {
            personal: 0.50,
            internal: 0.70,
            trusted: 0.80,
            public: 0.90,
        }
    }
}

impl AudienceGatesConfig {
    /// Get gate for a specific audience
    pub fn get(&self, audience: &str) -> Option<f64> {
        match audience.to_lowercase().as_str() {
            "personal" => Some(self.personal),
            "internal" => Some(self.internal),
            "trusted" => Some(self.trusted),
            "public" => Some(self.public),
            _ => None,
        }
    }
}

/// Stub penalty values by form
///
/// How much each stub form reduces the health score.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StubPenaltiesConfig {
    /// Minor issue, quick fix (default: 0.02)
    pub transient: f64,

    /// Known gap, documented (default: 0.05)
    pub persistent: f64,

    /// Blocks progression (default: 0.10)
    pub blocking: f64,

    /// Fundamental issue (default: 0.15)
    pub structural: f64,
}

impl Default for StubPenaltiesConfig {
    fn default() -> Self {
        Self {
            transient: 0.02,
            persistent: 0.05,
            blocking: 0.10,
            structural: 0.15,
        }
    }
}

impl StubPenaltiesConfig {
    /// Get penalty for a specific stub form
    pub fn get(&self, form: &str) -> Option<f64> {
        match form.to_lowercase().as_str() {
            "transient" => Some(self.transient),
            "persistent" => Some(self.persistent),
            "blocking" => Some(self.blocking),
            "structural" => Some(self.structural),
            _ => None,
        }
    }
}

/// Trust factor values by origin
///
/// How much to trust content based on its origin.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TrustFactorsConfig {
    /// Human-authored content (default: 0.90)
    pub human: f64,

    /// Collaborative authorship (default: 0.85)
    pub collaborative: f64,

    /// AI-assisted with human review (default: 0.70)
    pub ai_assisted: f64,

    /// Imported from external source (default: 0.60)
    pub imported: f64,

    /// Derived/transformed content (default: 0.60)
    pub derived: f64,

    /// Pure AI-generated (default: 0.50)
    pub ai: f64,
}

impl Default for TrustFactorsConfig {
    fn default() -> Self {
        Self {
            human: 0.90,
            collaborative: 0.85,
            ai_assisted: 0.70,
            imported: 0.60,
            derived: 0.60,
            ai: 0.50,
        }
    }
}

impl TrustFactorsConfig {
    /// Get trust factor for a specific origin
    pub fn get(&self, origin: &str) -> Option<f64> {
        match origin.to_lowercase().as_str() {
            "human" => Some(self.human),
            "collaborative" => Some(self.collaborative),
            "ai_assisted" => Some(self.ai_assisted),
            "imported" => Some(self.imported),
            "derived" => Some(self.derived),
            "ai" => Some(self.ai),
            _ => None,
        }
    }
}

/// Form staleness cadences in days
///
/// How long before each form type is considered "stale".
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FormCadencesConfig {
    /// Transient documents (default: 7 days)
    pub transient: u32,

    /// Developing documents (default: 30 days)
    pub developing: u32,

    /// Stable documents (default: 90 days)
    pub stable: u32,

    /// Evergreen documents (default: 365 days)
    pub evergreen: u32,

    /// Canonical documents (default: None - never stale)
    pub canonical: Option<u32>,
}

impl Default for FormCadencesConfig {
    fn default() -> Self {
        Self {
            transient: 7,
            developing: 30,
            stable: 90,
            evergreen: 365,
            canonical: None, // Never stale
        }
    }
}

impl FormCadencesConfig {
    /// Get cadence in days for a specific form (None = never stale)
    pub fn get(&self, form: &str) -> Option<u32> {
        match form.to_lowercase().as_str() {
            "transient" => Some(self.transient),
            "developing" => Some(self.developing),
            "stable" => Some(self.stable),
            "evergreen" => Some(self.evergreen),
            "canonical" => self.canonical,
            _ => None,
        }
    }

    /// Get half-life in days for freshness decay
    pub fn half_life(&self, form: &str) -> Option<f64> {
        self.get(form).map(|d| d as f64)
    }
}

/// Vector physics calculation defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VectorPhysicsConfig {
    /// Default urgency when not specified (default: 0.5)
    pub default_urgency: f64,

    /// Default impact when not specified (default: 0.5)
    pub default_impact: f64,

    /// Default complexity when not specified (default: 0.5)
    pub default_complexity: f64,

    /// Friction factor for external dependencies (default: 0.2)
    pub external_dep_friction: f64,

    /// Friction factor for controversy (default: 0.3)
    pub controversy_friction: f64,

    /// Friction factor for blocking stubs (default: 0.1)
    pub blocking_friction: f64,
}

impl Default for VectorPhysicsConfig {
    fn default() -> Self {
        Self {
            default_urgency: 0.5,
            default_impact: 0.5,
            default_complexity: 0.5,
            external_dep_friction: 0.2,
            controversy_friction: 0.3,
            blocking_friction: 0.1,
        }
    }
}

/// Configuration validation error
#[derive(Debug, Clone)]
pub enum ConfigValidationError {
    /// Weights don't sum correctly
    InvalidWeights { field: String, message: String },

    /// Values not in expected order
    InvalidOrder { field: String, message: String },

    /// Value outside valid range
    OutOfRange {
        field: String,
        value: f64,
        min: f64,
        max: f64,
    },
}

impl std::fmt::Display for ConfigValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidWeights { field, message } => {
                write!(f, "Invalid weights in '{}': {}", field, message)
            }
            Self::InvalidOrder { field, message } => {
                write!(f, "Invalid order in '{}': {}", field, message)
            }
            Self::OutOfRange {
                field,
                value,
                min,
                max,
            } => {
                write!(
                    f,
                    "Value {} for '{}' is out of range ({}-{})",
                    value, field, min, max
                )
            }
        }
    }
}

impl std::error::Error for ConfigValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CalculationConfig::default();
        assert_eq!(config.version, 1);
        assert_eq!(config.health.refinement_weight, 0.7);
        assert_eq!(config.audience_gates.public, 0.90);
    }

    #[test]
    fn test_validation_passes_for_defaults() {
        let config = CalculationConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validation_fails_for_bad_weights() {
        let mut config = CalculationConfig::default();
        config.health.refinement_weight = 0.5;
        config.health.stub_weight = 0.3; // Sum = 0.8, not 1.0

        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_fails_for_bad_gate_order() {
        let mut config = CalculationConfig::default();
        config.audience_gates.personal = 0.95; // Higher than public

        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let config = CalculationConfig::default();
        let yaml = serde_yaml::to_string(&config).unwrap();
        let parsed: CalculationConfig = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(config.health.refinement_weight, parsed.health.refinement_weight);
        assert_eq!(config.audience_gates.public, parsed.audience_gates.public);
    }

    #[test]
    fn test_audience_gates_get() {
        let gates = AudienceGatesConfig::default();
        assert_eq!(gates.get("personal"), Some(0.50));
        assert_eq!(gates.get("PUBLIC"), Some(0.90)); // Case insensitive
        assert_eq!(gates.get("unknown"), None);
    }

    #[test]
    fn test_stub_penalties_get() {
        let penalties = StubPenaltiesConfig::default();
        assert_eq!(penalties.get("transient"), Some(0.02));
        assert_eq!(penalties.get("STRUCTURAL"), Some(0.15));
    }
}
