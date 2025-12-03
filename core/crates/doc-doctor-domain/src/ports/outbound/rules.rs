//! Rule Engine Port
//!
//! L3 Operational layer - rule evaluation and application.
//! This is a PORT ONLY - implementation is deferred to future work.

use crate::entities::L1Properties;
use std::fmt;

/// Rule context for evaluation
#[derive(Debug, Clone, Default)]
pub struct RuleContext {
    /// Document properties being evaluated
    pub properties: Option<L1Properties>,
    /// Additional context values
    pub context: std::collections::HashMap<String, String>,
}

impl RuleContext {
    /// Create a new rule context
    pub fn new() -> Self {
        Self::default()
    }

    /// Create context with properties
    pub fn with_properties(properties: L1Properties) -> Self {
        Self {
            properties: Some(properties),
            context: std::collections::HashMap::new(),
        }
    }

    /// Add a context value
    pub fn with_value(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }
}

/// Result of rule evaluation
#[derive(Debug, Clone)]
pub struct RuleResult {
    /// Rule identifier
    pub rule_id: String,
    /// Whether the rule passed
    pub passed: bool,
    /// Message explaining the result
    pub message: String,
    /// Suggested actions
    pub suggestions: Vec<String>,
}

impl RuleResult {
    /// Create a passing result
    pub fn pass(rule_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            rule_id: rule_id.into(),
            passed: true,
            message: message.into(),
            suggestions: Vec::new(),
        }
    }

    /// Create a failing result
    pub fn fail(rule_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            rule_id: rule_id.into(),
            passed: false,
            message: message.into(),
            suggestions: Vec::new(),
        }
    }

    /// Add suggestions
    pub fn with_suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.suggestions = suggestions;
        self
    }
}

/// Action to apply to a document
#[derive(Debug, Clone)]
pub struct Action {
    /// Action type
    pub action_type: ActionType,
    /// Target field
    pub field: String,
    /// New value (if applicable)
    pub value: Option<String>,
}

/// Types of actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    /// Set a field value
    SetValue,
    /// Add to a list
    AddToList,
    /// Remove from a list
    RemoveFromList,
    /// Increment a numeric value
    Increment,
    /// Decrement a numeric value
    Decrement,
}

/// Rule engine error
#[derive(Debug, Clone)]
pub struct RuleError {
    /// Error message
    pub message: String,
    /// Rule that caused the error
    pub rule_id: Option<String>,
}

impl RuleError {
    /// Create a new rule error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            rule_id: None,
        }
    }

    /// Add rule ID
    pub fn with_rule(mut self, rule_id: impl Into<String>) -> Self {
        self.rule_id = Some(rule_id.into());
        self
    }
}

impl fmt::Display for RuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(rule_id) = &self.rule_id {
            write!(f, "[{}] {}", rule_id, self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for RuleError {}

/// Rule engine trait - L3 Operational layer
///
/// **PORT ONLY**: This trait defines the interface for rule evaluation
/// and application. Implementation is deferred to future work.
///
/// Rules can:
/// - Validate document state against policies
/// - Suggest actions based on conditions
/// - Apply automated corrections
pub trait RuleEngine: Send + Sync {
    /// Evaluate rules against a context
    ///
    /// # Arguments
    /// * `context` - Rule evaluation context
    ///
    /// # Returns
    /// List of rule results
    fn evaluate(&self, context: &RuleContext) -> Vec<RuleResult>;

    /// Apply an action to L1 properties
    ///
    /// # Arguments
    /// * `action` - Action to apply
    /// * `target` - Properties to modify
    ///
    /// # Returns
    /// Ok on success, error on failure
    fn apply(&self, action: &Action, target: &mut L1Properties) -> Result<(), RuleError>;

    /// Get available rule IDs
    ///
    /// # Returns
    /// List of rule identifiers
    fn available_rules(&self) -> Vec<String>;
}

/// No-op rule engine (placeholder implementation)
pub struct NoOpRuleEngine;

impl RuleEngine for NoOpRuleEngine {
    fn evaluate(&self, _context: &RuleContext) -> Vec<RuleResult> {
        Vec::new()
    }

    fn apply(&self, _action: &Action, _target: &mut L1Properties) -> Result<(), RuleError> {
        Ok(())
    }

    fn available_rules(&self) -> Vec<String> {
        Vec::new()
    }
}
