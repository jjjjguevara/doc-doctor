//! Configuration Loading
//!
//! Loads layered configuration for the CLI.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};

use doc_doctor_config_yaml::{
    config_sources, load_layered_config, user_config_path, FileConfigProvider,
};
use doc_doctor_domain::{CalculationConfig, ConfigProvider};
use serde::{Deserialize, Serialize};

use crate::tui::app::Column;

/// Global calculation configuration instance
static CONFIG: OnceLock<CalculationConfig> = OnceLock::new();

/// Global CLI configuration instance (mutable for saving)
static CLI_CONFIG: RwLock<Option<CliConfig>> = RwLock::new(None);

/// Dashboard-specific configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DashboardConfig {
    /// Visible columns in order
    #[serde(default)]
    pub columns: Option<Vec<Column>>,
}

/// CLI-specific configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CliConfig {
    /// Path aliases for quick access
    /// e.g., { "vault": "/path/to/vault", "docs": "/path/to/docs" }
    #[serde(default)]
    pub paths: HashMap<String, PathBuf>,

    /// Folders to ignore when scanning
    /// Can be absolute paths or folder names to match anywhere
    #[serde(default)]
    pub ignore: Vec<String>,

    /// Dashboard configuration
    #[serde(default)]
    pub dashboard: DashboardConfig,

    /// Test directory for running mock tests
    /// Used by the test runner TUI view
    #[serde(default)]
    pub test_dir: Option<PathBuf>,
}

impl CliConfig {
    /// Load CLI config from the same config file
    pub fn load() -> Self {
        let path = match user_config_path() {
            Some(p) => p,
            None => return Self::default(),
        };

        if !path.exists() {
            return Self::default();
        }

        match std::fs::read_to_string(&path) {
            Ok(content) => {
                // Parse the YAML and extract CLI-specific fields
                serde_yaml::from_str(&content).unwrap_or_default()
            }
            Err(_) => Self::default(),
        }
    }

    /// Save CLI config to the config file
    /// Preserves existing fields that aren't managed by CliConfig (like health, audience_gates, etc.)
    pub fn save(&self) -> anyhow::Result<()> {
        let path = user_config_path()
            .ok_or_else(|| anyhow::anyhow!("Could not determine user config directory"))?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Read existing file to preserve other fields
        let mut existing: serde_yaml::Value = if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            serde_yaml::from_str(&content).unwrap_or(serde_yaml::Value::Mapping(Default::default()))
        } else {
            serde_yaml::Value::Mapping(Default::default())
        };

        // Convert self to Value and merge
        let cli_value: serde_yaml::Value = serde_yaml::to_value(self)?;

        if let (Some(existing_map), Some(cli_map)) = (existing.as_mapping_mut(), cli_value.as_mapping()) {
            for (key, value) in cli_map {
                existing_map.insert(key.clone(), value.clone());
            }
        }

        let content = serde_yaml::to_string(&existing)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Resolve a path alias or return the original path
    pub fn resolve_path(&self, path: &str) -> PathBuf {
        // Check if it's an alias
        if let Some(resolved) = self.paths.get(path) {
            resolved.clone()
        } else {
            PathBuf::from(path)
        }
    }
}

/// Get the CLI configuration (cloned)
pub fn get_cli_config() -> CliConfig {
    let guard = CLI_CONFIG.read().unwrap();
    if let Some(config) = guard.as_ref() {
        config.clone()
    } else {
        drop(guard);
        let config = CliConfig::load();
        let mut write_guard = CLI_CONFIG.write().unwrap();
        *write_guard = Some(config.clone());
        config
    }
}

/// Update and save the CLI configuration
pub fn save_cli_config(config: &CliConfig) -> anyhow::Result<()> {
    config.save()?;
    let mut guard = CLI_CONFIG.write().unwrap();
    *guard = Some(config.clone());
    Ok(())
}

/// Get configured dashboard columns, or default
pub fn get_dashboard_columns() -> Vec<Column> {
    get_cli_config()
        .dashboard
        .columns
        .unwrap_or_else(Column::default_columns)
}

/// Save dashboard columns to config
pub fn save_dashboard_columns(columns: &[Column]) -> anyhow::Result<()> {
    let mut config = get_cli_config();
    config.dashboard.columns = Some(columns.to_vec());
    save_cli_config(&config)
}

/// Get ignore patterns from config
pub fn get_ignore_patterns() -> Vec<String> {
    get_cli_config().ignore
}

/// Get test directory from config
/// Returns None and logs a warning if not configured
pub fn get_test_dir() -> Option<PathBuf> {
    let config = get_cli_config();
    if config.test_dir.is_none() {
        eprintln!("Warning: No test_dir configured. Add 'test_dir: /path/to/tests' to your config.");
        eprintln!("  Config path: {:?}", user_config_path());
    }
    config.test_dir
}

/// Check if a path should be ignored based on configured patterns
pub fn should_ignore_path(path: &std::path::Path) -> bool {
    let ignore_patterns = get_ignore_patterns();
    if ignore_patterns.is_empty() {
        return false;
    }

    let path_str = path.to_string_lossy();

    for pattern in &ignore_patterns {
        // Check if it's an absolute path match
        if path_str.starts_with(pattern) {
            return true;
        }
        // Check if any path component matches the pattern
        for component in path.components() {
            if let std::path::Component::Normal(name) = component {
                if name.to_string_lossy() == *pattern {
                    return true;
                }
            }
        }
    }
    false
}

/// Resolve a path, checking for aliases first
pub fn resolve_path(path: &str) -> PathBuf {
    get_cli_config().resolve_path(path)
}

/// Load and return the global configuration
///
/// Configuration is loaded once and cached. Subsequent calls return the cached config.
///
/// Loading order (later overrides earlier):
/// 1. Built-in defaults
/// 2. User config: `~/.config/doc-doctor/config.yaml`
/// 3. Project config: `.doc-doctor.yaml` in current directory
pub fn get_config() -> &'static CalculationConfig {
    CONFIG.get_or_init(|| {
        match load_layered_config() {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Warning: Failed to load config: {}", e);
                eprintln!("Using default configuration");
                CalculationConfig::default()
            }
        }
    })
}

/// Print configuration sources for verbose output
pub fn print_config_sources() {
    eprintln!("Configuration sources:");
    for (source, exists) in config_sources() {
        let status = if exists { "loaded" } else { "not found" };
        eprintln!("  {} [{}]", source, status);
    }
}

/// Initialize user configuration file with defaults
///
/// Creates the user config file with default values if it doesn't exist.
///
/// # Returns
/// Ok with the path if created, Err if it already exists or creation failed
pub fn init_user_config() -> anyhow::Result<std::path::PathBuf> {
    let path = user_config_path()
        .ok_or_else(|| anyhow::anyhow!("Could not determine user config directory"))?;

    if path.exists() {
        anyhow::bail!("Config file already exists: {}", path.display());
    }

    let provider = FileConfigProvider::new(&path);
    let config = CalculationConfig::default();
    provider.save(&config).map_err(|e| anyhow::anyhow!("{}", e))?;

    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_config_returns_valid_config() {
        let config = get_config();
        // Should have valid default values
        assert!((config.health.refinement_weight - 0.7).abs() < 0.001
            || config.health.refinement_weight > 0.0);
    }
}
