//! State Dimensions
//!
//! Current state assessments: health, usefulness, compliance, trust, freshness, coverage.
//! All calculations are pure functions with no side effects.
//!
//! # Configuration
//!
//! All calculations accept an optional `CalculationConfig` parameter. When not provided,
//! built-in defaults are used. Use `dd config show` to see current defaults.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::config::{CalculationConfig, StubPenaltiesConfig};
use crate::entities::{Audience, Form, L1Properties, Origin, Stub, StubForm};

/// Calculate document health score
///
/// Formula: health = refinement_weight × refinement + stub_weight × (1 - stub_penalty)
///
/// # Arguments
/// * `refinement` - Document refinement score (0.0-1.0)
/// * `stubs` - List of document stubs
///
/// # Returns
/// Health score between 0.0 and 1.0
///
/// # Note
/// Uses default configuration. For custom weights, use `calculate_health_with_config`.
pub fn calculate_health(refinement: f64, stubs: &[Stub]) -> f64 {
    calculate_health_with_config(refinement, stubs, &CalculationConfig::default())
}

/// Calculate document health score with custom configuration
///
/// # Arguments
/// * `refinement` - Document refinement score (0.0-1.0)
/// * `stubs` - List of document stubs
/// * `config` - Calculation configuration
pub fn calculate_health_with_config(
    refinement: f64,
    stubs: &[Stub],
    config: &CalculationConfig,
) -> f64 {
    let stub_penalty = calculate_stub_penalty_with_config(stubs, &config.stub_penalties);
    let health = config.health.refinement_weight * refinement
        + config.health.stub_weight * (1.0 - stub_penalty);
    health.clamp(0.0, 1.0)
}

/// Calculate total stub penalty for refinement
///
/// Sum of individual stub penalties, capped at 1.0
pub fn calculate_stub_penalty(stubs: &[Stub]) -> f64 {
    calculate_stub_penalty_with_config(stubs, &StubPenaltiesConfig::default())
}

/// Calculate stub penalty with custom configuration
pub fn calculate_stub_penalty_with_config(stubs: &[Stub], config: &StubPenaltiesConfig) -> f64 {
    if stubs.is_empty() {
        return 0.0;
    }

    let total_penalty: f64 = stubs
        .iter()
        .map(|s| get_stub_penalty(&s.stub_form, config))
        .sum();

    total_penalty.min(1.0)
}

/// Get penalty for a specific stub form from config
fn get_stub_penalty(form: &StubForm, config: &StubPenaltiesConfig) -> f64 {
    match form {
        StubForm::Transient => config.transient,
        StubForm::Persistent => config.persistent,
        StubForm::Blocking => config.blocking,
        StubForm::Structural => config.structural,
    }
}

/// Usefulness assessment result
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Usefulness {
    /// Margin above/below gate (refinement - gate)
    pub margin: f64,

    /// Whether document meets the audience gate
    pub is_useful: bool,

    /// The audience being evaluated against
    pub audience: Audience,

    /// The document's refinement score
    pub refinement: f64,

    /// The audience's gate threshold
    pub gate: f64,
}

/// Calculate usefulness for a given audience
///
/// Formula: margin = refinement - audience_gate
///
/// # Arguments
/// * `refinement` - Document refinement score (0.0-1.0)
/// * `audience` - Target audience
///
/// # Returns
/// Usefulness assessment with margin and boolean result
///
/// # Note
/// Uses default gates. For custom gates, use `calculate_usefulness_with_config`.
pub fn calculate_usefulness(refinement: f64, audience: Audience) -> Usefulness {
    calculate_usefulness_with_config(refinement, audience, &CalculationConfig::default())
}

/// Calculate usefulness with custom configuration
pub fn calculate_usefulness_with_config(
    refinement: f64,
    audience: Audience,
    config: &CalculationConfig,
) -> Usefulness {
    let gate = get_audience_gate(&audience, &config.audience_gates);
    let margin = refinement - gate;

    Usefulness {
        margin,
        is_useful: margin >= 0.0,
        audience,
        refinement,
        gate,
    }
}

/// Get gate threshold for a specific audience from config
fn get_audience_gate(
    audience: &Audience,
    config: &crate::config::AudienceGatesConfig,
) -> f64 {
    match audience {
        Audience::Personal => config.personal,
        Audience::Internal => config.internal,
        Audience::Trusted => config.trusted,
        Audience::Public => config.public,
    }
}

/// Calculate document freshness based on time since last modification
///
/// Formula: freshness = e^(-ln(2) × Δt / τ_form)
///
/// Uses exponential decay with half-life based on document form.
pub fn calculate_freshness(modified: DateTime<Utc>, form: Form, now: DateTime<Utc>) -> f64 {
    calculate_freshness_with_config(modified, form, now, &CalculationConfig::default())
}

/// Calculate freshness with custom configuration
pub fn calculate_freshness_with_config(
    modified: DateTime<Utc>,
    form: Form,
    now: DateTime<Utc>,
    config: &CalculationConfig,
) -> f64 {
    let half_life_days = get_form_cadence(&form, &config.form_cadences);

    // Canonical documents (or None cadence) never go stale
    let half_life = match half_life_days {
        Some(days) => days as f64,
        None => return 1.0,
    };

    if half_life <= 0.0 {
        return 1.0;
    }

    let days_since = (now - modified).num_days() as f64;
    if days_since <= 0.0 {
        return 1.0;
    }

    // Exponential decay: freshness = e^(-ln(2) * Δt / half_life)
    (-0.693 * days_since / half_life).exp()
}

/// Get cadence in days for a form (None = never stale)
fn get_form_cadence(form: &Form, config: &crate::config::FormCadencesConfig) -> Option<u32> {
    match form {
        Form::Transient => Some(config.transient),
        Form::Developing => Some(config.developing),
        Form::Stable => Some(config.stable),
        Form::Evergreen => Some(config.evergreen),
        Form::Canonical => config.canonical,
    }
}

/// Calculate trust level based on origin
pub fn calculate_trust(origin: Origin) -> f64 {
    calculate_trust_with_config(origin, &CalculationConfig::default())
}

/// Calculate trust with custom configuration
pub fn calculate_trust_with_config(origin: Origin, config: &CalculationConfig) -> f64 {
    match origin {
        Origin::Human => config.trust_factors.human,
        Origin::Collaborative => config.trust_factors.collaborative,
        Origin::AiAssisted => config.trust_factors.ai_assisted,
        Origin::Imported => config.trust_factors.imported,
        Origin::Derived => config.trust_factors.derived,
        Origin::Ai => config.trust_factors.ai,
    }
}

/// All state dimensions for a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDimensions {
    /// Health score (0.0-1.0)
    pub health: f64,

    /// Usefulness assessment
    pub usefulness: Usefulness,

    /// Trust level based on origin (0.0-1.0)
    pub trust_level: f64,

    /// Freshness (1.0 = just updated, decays over time)
    pub freshness: f64,

    /// Compliance fit (placeholder, needs external context)
    pub compliance_fit: f64,

    /// Coverage fit (placeholder, needs external context)
    pub coverage_fit: f64,

    /// Whether default config was used (for messaging)
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    #[serde(default)]
    pub using_defaults: bool,
}

impl StateDimensions {
    /// Calculate all state dimensions from L1 properties
    ///
    /// Uses default configuration.
    pub fn calculate(props: &L1Properties) -> Self {
        Self::calculate_at(props, Utc::now())
    }

    /// Calculate all state dimensions at a specific time
    pub fn calculate_at(props: &L1Properties, now: DateTime<Utc>) -> Self {
        Self::calculate_with_config(props, now, &CalculationConfig::default(), true)
    }

    /// Calculate with custom configuration
    pub fn calculate_with_config(
        props: &L1Properties,
        now: DateTime<Utc>,
        config: &CalculationConfig,
        using_defaults: bool,
    ) -> Self {
        let health = calculate_health_with_config(props.refinement.value(), &props.stubs, config);
        let usefulness =
            calculate_usefulness_with_config(props.refinement.value(), props.audience, config);
        let trust_level = calculate_trust_with_config(props.origin, config);

        let freshness = match props.modified {
            Some(modified) => calculate_freshness_with_config(modified, props.form, now, config),
            None => 1.0, // No modification date, assume fresh
        };

        Self {
            health,
            usefulness,
            trust_level,
            freshness,
            compliance_fit: 1.0,
            coverage_fit: 1.0,
            using_defaults,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_health_no_stubs() {
        let health = calculate_health(0.8, &[]);
        // health = 0.7 * 0.8 + 0.3 * 1.0 = 0.56 + 0.3 = 0.86
        assert!((health - 0.86).abs() < 0.01);
    }

    #[test]
    fn test_calculate_health_with_stubs() {
        let stubs = vec![{
            let mut s = Stub::compact("link", "test");
            s.stub_form = StubForm::Transient;
            s
        }];

        let health = calculate_health(0.8, &stubs);
        // stub_penalty = 0.02
        // health = 0.7 * 0.8 + 0.3 * (1 - 0.02) = 0.56 + 0.294 = 0.854
        assert!((health - 0.854).abs() < 0.01);
    }

    #[test]
    fn test_calculate_health_custom_weights() {
        let mut config = CalculationConfig::default();
        config.health.refinement_weight = 0.6;
        config.health.stub_weight = 0.4;

        let health = calculate_health_with_config(0.8, &[], &config);
        // health = 0.6 * 0.8 + 0.4 * 1.0 = 0.48 + 0.4 = 0.88
        assert!((health - 0.88).abs() < 0.01);
    }

    #[test]
    fn test_calculate_health_blocking_stubs() {
        let stubs = vec![
            {
                let mut s = Stub::compact("fix", "critical");
                s.stub_form = StubForm::Blocking;
                s
            },
            {
                let mut s = Stub::compact("fix", "also critical");
                s.stub_form = StubForm::Structural;
                s
            },
        ];

        let health = calculate_health(0.8, &stubs);
        // stub_penalty = 0.10 + 0.15 = 0.25
        // health = 0.7 * 0.8 + 0.3 * (1 - 0.25) = 0.56 + 0.225 = 0.785
        assert!((health - 0.785).abs() < 0.01);
    }

    #[test]
    fn test_stub_penalty_capped() {
        let stubs: Vec<Stub> = (0..20)
            .map(|i| {
                let mut s = Stub::compact("fix", format!("stub {}", i));
                s.stub_form = StubForm::Structural;
                s
            })
            .collect();

        let penalty = calculate_stub_penalty(&stubs);
        assert_eq!(penalty, 1.0);
    }

    #[test]
    fn test_calculate_usefulness() {
        let usefulness = calculate_usefulness(0.75, Audience::Internal);
        assert!((usefulness.gate - 0.70).abs() < 0.001);
        assert!((usefulness.margin - 0.05).abs() < 0.001);
        assert!(usefulness.is_useful);

        let usefulness = calculate_usefulness(0.65, Audience::Internal);
        assert!(!usefulness.is_useful);
        assert!(usefulness.margin < 0.0);
    }

    #[test]
    fn test_calculate_usefulness_custom_gates() {
        let mut config = CalculationConfig::default();
        config.audience_gates.internal = 0.60; // Lower gate

        let usefulness = calculate_usefulness_with_config(0.65, Audience::Internal, &config);
        assert!((usefulness.gate - 0.60).abs() < 0.001);
        assert!(usefulness.is_useful); // Now passes!
    }

    #[test]
    fn test_calculate_usefulness_all_audiences() {
        assert!(calculate_usefulness(0.50, Audience::Personal).is_useful);
        assert!(!calculate_usefulness(0.49, Audience::Personal).is_useful);

        assert!(calculate_usefulness(0.70, Audience::Internal).is_useful);
        assert!(!calculate_usefulness(0.69, Audience::Internal).is_useful);

        assert!(calculate_usefulness(0.80, Audience::Trusted).is_useful);
        assert!(!calculate_usefulness(0.79, Audience::Trusted).is_useful);

        assert!(calculate_usefulness(0.90, Audience::Public).is_useful);
        assert!(!calculate_usefulness(0.89, Audience::Public).is_useful);
    }

    #[test]
    fn test_calculate_freshness() {
        let now = Utc::now();
        let one_week_ago = now - chrono::Duration::days(7);

        let freshness = calculate_freshness(one_week_ago, Form::Transient, now);
        assert!((freshness - 0.5).abs() < 0.01);

        let freshness = calculate_freshness(now, Form::Developing, now);
        assert!((freshness - 1.0).abs() < 0.01);

        let very_old = now - chrono::Duration::days(3650);
        let freshness = calculate_freshness(very_old, Form::Canonical, now);
        assert_eq!(freshness, 1.0);
    }

    #[test]
    fn test_calculate_trust() {
        assert!(calculate_trust(Origin::Human) > calculate_trust(Origin::Ai));
        assert!(calculate_trust(Origin::AiAssisted) > calculate_trust(Origin::Ai));
    }

    #[test]
    fn test_calculate_trust_custom_factors() {
        let mut config = CalculationConfig::default();
        config.trust_factors.ai = 0.8; // Trust AI more

        let trust = calculate_trust_with_config(Origin::Ai, &config);
        assert!((trust - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_state_dimensions() {
        let props = L1Properties::new()
            .refinement(0.8)
            .audience(Audience::Internal);

        let dims = StateDimensions::calculate(&props);
        assert!(dims.health > 0.8);
        assert!(dims.usefulness.is_useful);
        assert!(dims.trust_level > 0.8);
        assert!(dims.using_defaults);
    }

    #[test]
    fn test_state_dimensions_with_config() {
        let props = L1Properties::new()
            .refinement(0.8)
            .audience(Audience::Internal);

        let config = CalculationConfig::default();
        let dims = StateDimensions::calculate_with_config(&props, Utc::now(), &config, false);

        assert!(!dims.using_defaults);
    }
}
