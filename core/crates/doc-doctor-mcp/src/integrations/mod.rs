//! External Plugin Integrations
//!
//! Bridges to Obsidian plugins for enhanced functionality:
//! - Git: Version control via Obsidian Git plugin
//! - Smart Connections: RAG/semantic search via embeddings

pub mod git;
pub mod smart_connections;

/// Result of checking integration availability
#[derive(Debug, Clone)]
pub struct IntegrationStatus {
    pub available: bool,
    pub plugin_name: &'static str,
    pub reminder: Option<String>,
}

impl IntegrationStatus {
    pub fn available(plugin: &'static str) -> Self {
        Self {
            available: true,
            plugin_name: plugin,
            reminder: None,
        }
    }

    pub fn unavailable(plugin: &'static str, reminder: impl Into<String>) -> Self {
        Self {
            available: false,
            plugin_name: plugin,
            reminder: Some(reminder.into()),
        }
    }
}
