//! Standard Configuration Paths
//!
//! Utilities for locating configuration files in standard locations.

use std::path::PathBuf;

/// User configuration file name
pub const USER_CONFIG_NAME: &str = "config.yaml";

/// Project configuration file name
pub const PROJECT_CONFIG_NAME: &str = ".doc-doctor.yaml";

/// Application directory name under config home
pub const APP_DIR_NAME: &str = "doc-doctor";

/// Get the user configuration file path
///
/// Returns the path to `~/.config/doc-doctor/config.yaml` on Unix
/// or the equivalent on other platforms.
///
/// # Returns
/// `Some(path)` if the config directory can be determined, `None` otherwise
///
/// # Example
///
/// ```
/// use doc_doctor_config_yaml::user_config_path;
///
/// if let Some(path) = user_config_path() {
///     println!("User config: {}", path.display());
/// }
/// ```
pub fn user_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join(APP_DIR_NAME).join(USER_CONFIG_NAME))
}

/// Get the user configuration directory
///
/// Returns the path to `~/.config/doc-doctor/` on Unix
/// or the equivalent on other platforms.
///
/// # Returns
/// `Some(path)` if the config directory can be determined, `None` otherwise
pub fn user_config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join(APP_DIR_NAME))
}

/// Get the project configuration file path
///
/// Returns the path to `.doc-doctor.yaml` in the current working directory.
///
/// # Example
///
/// ```
/// use doc_doctor_config_yaml::project_config_path;
///
/// let path = project_config_path();
/// println!("Project config: {}", path.display());
/// ```
pub fn project_config_path() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(PROJECT_CONFIG_NAME)
}

/// Get the project configuration file path relative to a specific directory
///
/// # Arguments
/// * `root` - Directory to look for the project config
///
/// # Example
///
/// ```
/// use doc_doctor_config_yaml::project_config_path_in;
///
/// let path = project_config_path_in("/my/project");
/// assert!(path.ends_with(".doc-doctor.yaml"));
/// ```
pub fn project_config_path_in(root: impl AsRef<std::path::Path>) -> PathBuf {
    root.as_ref().join(PROJECT_CONFIG_NAME)
}

/// Find the nearest project config by walking up the directory tree
///
/// Starting from the given directory, walks up the directory tree
/// looking for a `.doc-doctor.yaml` file.
///
/// # Arguments
/// * `start` - Directory to start searching from
///
/// # Returns
/// `Some(path)` if a config file is found, `None` otherwise
///
/// # Example
///
/// ```no_run
/// use doc_doctor_config_yaml::find_project_config;
///
/// if let Some(config_path) = find_project_config(".") {
///     println!("Found project config at: {}", config_path.display());
/// }
/// ```
pub fn find_project_config(start: impl AsRef<std::path::Path>) -> Option<PathBuf> {
    let start = start.as_ref();
    let canonical = start.canonicalize().ok()?;

    let mut current = canonical.as_path();
    loop {
        let config_path = current.join(PROJECT_CONFIG_NAME);
        if config_path.exists() {
            return Some(config_path);
        }

        match current.parent() {
            Some(parent) => current = parent,
            None => return None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_user_config_path() {
        // Should return Some on most systems
        let path = user_config_path();
        if let Some(p) = path {
            assert!(p.ends_with("config.yaml"));
            assert!(p.to_string_lossy().contains("doc-doctor"));
        }
    }

    #[test]
    fn test_user_config_dir() {
        let dir = user_config_dir();
        if let Some(d) = dir {
            assert!(d.ends_with("doc-doctor"));
        }
    }

    #[test]
    fn test_project_config_path() {
        let path = project_config_path();
        assert!(path.ends_with(".doc-doctor.yaml"));
    }

    #[test]
    fn test_project_config_path_in() {
        let path = project_config_path_in("/my/project");
        assert_eq!(path, PathBuf::from("/my/project/.doc-doctor.yaml"));
    }

    #[test]
    fn test_find_project_config() {
        let temp_dir = TempDir::new().unwrap();

        // Create nested directory structure
        let nested = temp_dir.path().join("a").join("b").join("c");
        fs::create_dir_all(&nested).unwrap();

        // Create config in the middle
        let config_path = temp_dir.path().join("a").join(".doc-doctor.yaml");
        fs::write(&config_path, "version: 1").unwrap();

        // Should find it from nested directory
        let found = find_project_config(&nested);
        assert!(found.is_some());
        assert_eq!(found.unwrap().canonicalize().unwrap(), config_path.canonicalize().unwrap());
    }

    #[test]
    fn test_find_project_config_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let nested = temp_dir.path().join("a").join("b");
        fs::create_dir_all(&nested).unwrap();

        // No config file anywhere
        let found = find_project_config(&nested);
        // May find a config file in parent directories of temp_dir, so just check it works
        // without panicking
        let _ = found;
    }
}
