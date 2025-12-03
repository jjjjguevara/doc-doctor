//! Config Command
//!
//! Show, initialize, and manage configuration.

use anyhow::Result;
use clap::Args;

use doc_doctor_config_yaml::{config_sources, user_config_path, project_config_path};

use crate::config::{get_config, get_cli_config, init_user_config};
use crate::output::OutputFormat;

#[derive(Args)]
pub struct ConfigCommand {
    /// Show current configuration
    #[arg(long)]
    pub show: bool,

    /// Initialize user config with defaults
    #[arg(long)]
    pub init: bool,

    /// Show config sources and their status
    #[arg(long)]
    pub sources: bool,

    /// Show config file paths
    #[arg(long)]
    pub paths: bool,

    /// Show configured path aliases
    #[arg(long)]
    pub aliases: bool,
}

impl ConfigCommand {
    pub fn run(&self, format: OutputFormat, verbose: bool) -> Result<()> {
        // Default to --show if no flags specified
        let show_config = self.show || (!self.init && !self.sources && !self.paths && !self.aliases);

        if self.paths {
            self.show_paths(format)?;
        }

        if self.sources {
            self.show_sources(format)?;
        }

        if self.aliases {
            self.show_aliases(format)?;
        }

        if self.init {
            self.init_config(verbose)?;
        }

        if show_config && !self.init {
            self.show_config(format, verbose)?;
        }

        Ok(())
    }

    fn show_aliases(&self, format: OutputFormat) -> Result<()> {
        let cli_config = get_cli_config();

        match format {
            OutputFormat::Human => {
                println!("Path Aliases:");
                if cli_config.paths.is_empty() {
                    println!("  No path aliases configured.");
                    println!();
                    println!("Add aliases to your config file:");
                    if let Some(path) = user_config_path() {
                        println!("  {}", path.display());
                    }
                    println!();
                    println!("Example:");
                    println!("  paths:");
                    println!("    vault: /path/to/your/vault");
                    println!("    docs: /path/to/docs");
                    println!();
                    println!("Then use: ddoc dashboard vault");
                } else {
                    for (alias, path) in &cli_config.paths {
                        println!("  {} -> {}", alias, path.display());
                    }
                }
            }
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&cli_config.paths)?);
            }
            OutputFormat::Yaml => {
                println!("{}", serde_yaml::to_string(&cli_config.paths)?);
            }
        }

        Ok(())
    }

    fn show_config(&self, format: OutputFormat, verbose: bool) -> Result<()> {
        let config = get_config();

        if verbose {
            eprintln!("Loading configuration from layered sources...");
            for (source, exists) in config_sources() {
                let status = if exists { "loaded" } else { "skipped" };
                eprintln!("  {} [{}]", source, status);
            }
        }

        match format {
            OutputFormat::Human => {
                println!("Doc Doctor Configuration");
                println!("========================\n");
                println!("Health Calculation:");
                println!("  refinement_weight: {}", config.health.refinement_weight);
                println!("  stub_weight: {}", config.health.stub_weight);
                println!("\nAudience Gates:");
                println!("  personal: {}", config.audience_gates.personal);
                println!("  internal: {}", config.audience_gates.internal);
                println!("  trusted: {}", config.audience_gates.trusted);
                println!("  public: {}", config.audience_gates.public);
                println!("\nStub Penalties:");
                println!("  transient: {}", config.stub_penalties.transient);
                println!("  persistent: {}", config.stub_penalties.persistent);
                println!("  blocking: {}", config.stub_penalties.blocking);
                println!("  structural: {}", config.stub_penalties.structural);
                println!("\nTrust Factors:");
                println!("  human: {}", config.trust_factors.human);
                println!("  collaborative: {}", config.trust_factors.collaborative);
                println!("  ai_assisted: {}", config.trust_factors.ai_assisted);
                println!("  imported: {}", config.trust_factors.imported);
                println!("  derived: {}", config.trust_factors.derived);
                println!("  ai: {}", config.trust_factors.ai);
                println!("\nForm Cadences (days):");
                println!("  transient: {}", config.form_cadences.transient);
                println!("  developing: {}", config.form_cadences.developing);
                println!("  stable: {}", config.form_cadences.stable);
                println!("  evergreen: {}", config.form_cadences.evergreen);
                println!("  canonical: {:?}", config.form_cadences.canonical);
            }
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(config)?);
            }
            OutputFormat::Yaml => {
                println!("{}", serde_yaml::to_string(config)?);
            }
        }

        Ok(())
    }

    fn show_sources(&self, format: OutputFormat) -> Result<()> {
        let sources: Vec<_> = config_sources()
            .into_iter()
            .map(|(path, exists)| {
                serde_json::json!({
                    "source": path,
                    "status": if exists { "loaded" } else { "not_found" }
                })
            })
            .collect();

        match format {
            OutputFormat::Human => {
                println!("Configuration Sources:");
                for (path, exists) in config_sources() {
                    let status = if exists { "✓ loaded" } else { "✗ not found" };
                    println!("  {} [{}]", path, status);
                }
            }
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&sources)?);
            }
            OutputFormat::Yaml => {
                println!("{}", serde_yaml::to_string(&sources)?);
            }
        }

        Ok(())
    }

    fn show_paths(&self, format: OutputFormat) -> Result<()> {
        let user_path = user_config_path()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "<unavailable>".to_string());
        let project_path = project_config_path().display().to_string();

        match format {
            OutputFormat::Human => {
                println!("Configuration Paths:");
                println!("  User config:    {}", user_path);
                println!("  Project config: {}", project_path);
            }
            OutputFormat::Json => {
                let paths = serde_json::json!({
                    "user_config": user_path,
                    "project_config": project_path
                });
                println!("{}", serde_json::to_string_pretty(&paths)?);
            }
            OutputFormat::Yaml => {
                let paths = serde_json::json!({
                    "user_config": user_path,
                    "project_config": project_path
                });
                println!("{}", serde_yaml::to_string(&paths)?);
            }
        }

        Ok(())
    }

    fn init_config(&self, verbose: bool) -> Result<()> {
        match init_user_config() {
            Ok(path) => {
                println!("Created config file: {}", path.display());
                if verbose {
                    eprintln!("Edit this file to customize calculation parameters.");
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
        Ok(())
    }
}
