//! YAML Configuration Provider
//!
//! File-based configuration adapter implementing the `ConfigProvider` port.
//!
//! # Layered Configuration
//!
//! Configuration is loaded in layers (later overrides earlier):
//! 1. Built-in defaults
//! 2. User config: `~/.config/doc-doctor/config.yaml`
//! 3. Project config: `.doc-doctor.yaml` in working directory
//! 4. CLI arguments (highest priority)
//!
//! # Usage
//!
//! ```no_run
//! use doc_doctor_config_yaml::{FileConfigProvider, load_layered_config, ConfigProvider};
//!
//! // Load from a specific file
//! let provider = FileConfigProvider::new("/path/to/config.yaml");
//! let config = provider.load().unwrap();
//!
//! // Load with standard layering
//! let config = load_layered_config().unwrap();
//! ```

mod file_provider;
mod paths;

pub use file_provider::FileConfigProvider;
pub use paths::{find_project_config, project_config_path, project_config_path_in, user_config_dir, user_config_path};

use doc_doctor_domain::{
    CalculationConfig, ConfigError, DefaultConfigProvider, LayeredConfigProvider,
};

// Re-export domain types for convenience
pub use doc_doctor_domain::{ConfigProvider, CalculationConfig as Config};

/// Load configuration with standard layering
///
/// Loads config in order (later overrides earlier):
/// 1. Built-in defaults
/// 2. User config (~/.config/doc-doctor/config.yaml)
/// 3. Project config (.doc-doctor.yaml in current directory)
///
/// # Returns
/// Merged configuration from all layers
///
/// # Errors
/// Returns error if any existing config file fails to parse
pub fn load_layered_config() -> Result<CalculationConfig, ConfigError> {
    let mut provider = LayeredConfigProvider::new()
        .add_layer(Box::new(DefaultConfigProvider));

    // Add user config if it exists
    if let Some(user_path) = user_config_path() {
        provider = provider.add_layer(Box::new(FileConfigProvider::new(user_path)));
    }

    // Add project config if it exists
    let project_path = project_config_path();
    provider = provider.add_layer(Box::new(FileConfigProvider::new(project_path)));

    provider.load_merged()
}

/// Load configuration with custom project root
///
/// Like `load_layered_config` but allows specifying a custom project root
/// instead of using the current working directory.
///
/// # Arguments
/// * `project_root` - Directory to look for `.doc-doctor.yaml`
pub fn load_layered_config_with_root(
    project_root: impl AsRef<std::path::Path>,
) -> Result<CalculationConfig, ConfigError> {
    let mut provider = LayeredConfigProvider::new()
        .add_layer(Box::new(DefaultConfigProvider));

    // Add user config if it exists
    if let Some(user_path) = user_config_path() {
        provider = provider.add_layer(Box::new(FileConfigProvider::new(user_path)));
    }

    // Add project config
    let project_path = project_root.as_ref().join(".doc-doctor.yaml");
    provider = provider.add_layer(Box::new(FileConfigProvider::new(project_path)));

    provider.load_merged()
}

/// Get information about loaded config sources
pub fn config_sources() -> Vec<(String, bool)> {
    let mut sources = vec![("built-in defaults".to_string(), true)];

    if let Some(user_path) = user_config_path() {
        let exists = user_path.exists();
        sources.push((user_path.display().to_string(), exists));
    }

    let project_path = project_config_path();
    let exists = project_path.exists();
    sources.push((project_path.display().to_string(), exists));

    sources
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_load_layered_defaults_only() {
        // When no config files exist, should return defaults
        let config = load_layered_config().unwrap();
        assert_eq!(config.health.refinement_weight, 0.7);
        assert_eq!(config.audience_gates.public, 0.90);
    }

    #[test]
    fn test_load_layered_with_project_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".doc-doctor.yaml");

        let mut file = std::fs::File::create(&config_path).unwrap();
        writeln!(
            file,
            r#"
health:
  refinement_weight: 0.8
  stub_weight: 0.2
"#
        )
        .unwrap();

        let config = load_layered_config_with_root(temp_dir.path()).unwrap();
        assert_eq!(config.health.refinement_weight, 0.8);
        assert_eq!(config.health.stub_weight, 0.2);
        // Other values should be defaults
        assert_eq!(config.audience_gates.public, 0.90);
    }

    #[test]
    fn test_config_sources() {
        let sources = config_sources();
        assert!(!sources.is_empty());
        assert_eq!(sources[0].0, "built-in defaults");
        assert!(sources[0].1); // defaults always exist
    }
}
