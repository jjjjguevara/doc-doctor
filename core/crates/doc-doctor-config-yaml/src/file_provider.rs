//! File-based Configuration Provider
//!
//! Implements `ConfigProvider` for loading/saving YAML configuration files.

use std::fs;
use std::path::{Path, PathBuf};

use doc_doctor_domain::{CalculationConfig, ConfigError, ConfigProvider};

/// File-based configuration provider
///
/// Loads and saves configuration from/to a YAML file.
///
/// # Example
///
/// ```no_run
/// use doc_doctor_config_yaml::FileConfigProvider;
/// use doc_doctor_domain::ConfigProvider;
///
/// let provider = FileConfigProvider::new("/path/to/config.yaml");
/// if provider.exists() {
///     let config = provider.load().unwrap();
///     println!("Loaded config from: {}", provider.source());
/// }
/// ```
#[derive(Debug, Clone)]
pub struct FileConfigProvider {
    path: PathBuf,
}

impl FileConfigProvider {
    /// Create a new file config provider
    ///
    /// # Arguments
    /// * `path` - Path to the YAML configuration file
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    /// Get the file path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Create parent directories if they don't exist
    fn ensure_parent_dir(&self) -> Result<(), ConfigError> {
        if let Some(parent) = self.path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| {
                    ConfigError::new(format!("Failed to create config directory: {}", e))
                        .with_path(parent.display().to_string())
                })?;
            }
        }
        Ok(())
    }
}

impl ConfigProvider for FileConfigProvider {
    fn load(&self) -> Result<CalculationConfig, ConfigError> {
        if !self.exists() {
            return Err(ConfigError::new("Config file not found").with_path(self.source()));
        }

        let content = fs::read_to_string(&self.path).map_err(|e| {
            ConfigError::new(format!("Failed to read config file: {}", e)).with_path(self.source())
        })?;

        let config: CalculationConfig = serde_yaml::from_str(&content).map_err(|e| {
            ConfigError::new(format!("Failed to parse YAML: {}", e)).with_path(self.source())
        })?;

        // Validate loaded config
        config.validate().map_err(|e| {
            ConfigError::new(format!("Invalid configuration: {}", e)).with_path(self.source())
        })?;

        Ok(config)
    }

    fn save(&self, config: &CalculationConfig) -> Result<(), ConfigError> {
        // Validate before saving
        config.validate().map_err(|e| {
            ConfigError::new(format!("Cannot save invalid configuration: {}", e))
        })?;

        self.ensure_parent_dir()?;

        let yaml = serde_yaml::to_string(config).map_err(|e| {
            ConfigError::new(format!("Failed to serialize config: {}", e))
        })?;

        // Add header comment
        let content = format!(
            "# Doc Doctor Configuration\n# See: https://github.com/jjjjguevara/doc-doctor\n\n{}",
            yaml
        );

        fs::write(&self.path, content).map_err(|e| {
            ConfigError::new(format!("Failed to write config file: {}", e))
                .with_path(self.source())
        })?;

        Ok(())
    }

    fn exists(&self) -> bool {
        self.path.exists() && self.path.is_file()
    }

    fn source(&self) -> String {
        self.path.display().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use doc_doctor_domain::CalculationConfig;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_load_valid_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        let mut file = fs::File::create(&config_path).unwrap();
        writeln!(
            file,
            r#"
version: 1
health:
  refinement_weight: 0.8
  stub_weight: 0.2
audience_gates:
  personal: 0.50
  internal: 0.70
  trusted: 0.80
  public: 0.90
stub_penalties:
  transient: 0.02
  persistent: 0.05
  blocking: 0.10
  structural: 0.15
"#
        )
        .unwrap();

        let provider = FileConfigProvider::new(&config_path);
        assert!(provider.exists());

        let config = provider.load().unwrap();
        assert_eq!(config.health.refinement_weight, 0.8);
        assert_eq!(config.health.stub_weight, 0.2);
    }

    #[test]
    fn test_load_partial_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        // Only override some values
        let mut file = fs::File::create(&config_path).unwrap();
        writeln!(
            file,
            r#"
health:
  refinement_weight: 0.6
  stub_weight: 0.4
"#
        )
        .unwrap();

        let provider = FileConfigProvider::new(&config_path);
        let config = provider.load().unwrap();

        // Overridden values
        assert_eq!(config.health.refinement_weight, 0.6);
        assert_eq!(config.health.stub_weight, 0.4);

        // Default values (from serde default)
        assert_eq!(config.audience_gates.public, 0.90);
    }

    #[test]
    fn test_load_nonexistent_file() {
        let provider = FileConfigProvider::new("/nonexistent/path/config.yaml");
        assert!(!provider.exists());

        let result = provider.load();
        assert!(result.is_err());
    }

    #[test]
    fn test_load_invalid_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        let mut file = fs::File::create(&config_path).unwrap();
        writeln!(file, "invalid: yaml: content: [").unwrap();

        let provider = FileConfigProvider::new(&config_path);
        let result = provider.load();
        assert!(result.is_err());
    }

    #[test]
    fn test_load_invalid_config_values() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        // Invalid: weights don't sum to 1.0
        let mut file = fs::File::create(&config_path).unwrap();
        writeln!(
            file,
            r#"
health:
  refinement_weight: 0.5
  stub_weight: 0.3
"#
        )
        .unwrap();

        let provider = FileConfigProvider::new(&config_path);
        let result = provider.load();
        assert!(result.is_err());
    }

    #[test]
    fn test_save_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("subdir").join("config.yaml");

        let provider = FileConfigProvider::new(&config_path);
        let config = CalculationConfig::default();

        // Should create parent directory and save
        provider.save(&config).unwrap();
        assert!(provider.exists());

        // Should be able to load it back
        let loaded = provider.load().unwrap();
        assert_eq!(loaded.health.refinement_weight, config.health.refinement_weight);
    }

    #[test]
    fn test_save_invalid_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        let provider = FileConfigProvider::new(&config_path);

        // Create invalid config
        let mut config = CalculationConfig::default();
        config.health.refinement_weight = 0.5;
        config.health.stub_weight = 0.3; // Sum != 1.0

        let result = provider.save(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_source() {
        let provider = FileConfigProvider::new("/some/path/config.yaml");
        assert_eq!(provider.source(), "/some/path/config.yaml");
    }
}
