//! Configuration Provider Port
//!
//! Abstraction over configuration loading and saving.

use crate::config::CalculationConfig;

/// Configuration loading error
#[derive(Debug, Clone)]
pub struct ConfigError {
    /// Error message
    pub message: String,
    /// Source path (if applicable)
    pub path: Option<String>,
}

impl ConfigError {
    /// Create a new config error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            path: None,
        }
    }

    /// Add path information
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(path) = &self.path {
            write!(f, "{} ({})", self.message, path)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for ConfigError {}

/// Configuration provider trait
///
/// This is an outbound port for loading and saving calculation configuration.
/// Adapters implement this to provide file-based, environment-based,
/// or other configuration sources.
pub trait ConfigProvider: Send + Sync {
    /// Load configuration
    ///
    /// # Returns
    /// Loaded configuration or error
    fn load(&self) -> Result<CalculationConfig, ConfigError>;

    /// Save configuration
    ///
    /// # Arguments
    /// * `config` - Configuration to save
    ///
    /// # Returns
    /// Ok on success, error on failure
    fn save(&self, config: &CalculationConfig) -> Result<(), ConfigError>;

    /// Check if configuration exists
    fn exists(&self) -> bool;

    /// Get the source description (e.g., file path)
    fn source(&self) -> String;
}

/// Default configuration provider that returns built-in defaults
pub struct DefaultConfigProvider;

impl ConfigProvider for DefaultConfigProvider {
    fn load(&self) -> Result<CalculationConfig, ConfigError> {
        Ok(CalculationConfig::default())
    }

    fn save(&self, _config: &CalculationConfig) -> Result<(), ConfigError> {
        Err(ConfigError::new("Cannot save to default config provider"))
    }

    fn exists(&self) -> bool {
        true // Defaults always exist
    }

    fn source(&self) -> String {
        "built-in defaults".to_string()
    }
}

/// Layered configuration provider
///
/// Loads configuration in layers, with later layers overriding earlier ones.
pub struct LayeredConfigProvider {
    providers: Vec<Box<dyn ConfigProvider>>,
}

impl LayeredConfigProvider {
    /// Create a new layered provider
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    /// Add a provider layer (later layers override earlier)
    pub fn add_layer(mut self, provider: Box<dyn ConfigProvider>) -> Self {
        self.providers.push(provider);
        self
    }

    /// Load merged configuration from all layers
    pub fn load_merged(&self) -> Result<CalculationConfig, ConfigError> {
        let mut config = CalculationConfig::default();

        for provider in &self.providers {
            if provider.exists() {
                let layer_config = provider.load()?;
                config = config.merge(&layer_config);
            }
        }

        Ok(config)
    }

    /// Get sources that were loaded
    pub fn loaded_sources(&self) -> Vec<String> {
        self.providers
            .iter()
            .filter(|p| p.exists())
            .map(|p| p.source())
            .collect()
    }
}

impl Default for LayeredConfigProvider {
    fn default() -> Self {
        Self::new()
    }
}
