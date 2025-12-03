//! Health Command
//!
//! Calculate health score from refinement and stubs.

use anyhow::Result;
use clap::Args;

use doc_doctor_domain::{calculate_health_with_config, calculate_stub_penalty_with_config, Stub};

use crate::config::get_config;
use crate::output::{format_output, HealthOutput, OutputFormat};
use crate::tui::widgets::tables::format_health_output;

#[derive(Args)]
pub struct HealthCommand {
    /// Refinement score (0.0-1.0)
    #[arg(long)]
    pub refinement: f64,

    /// Stubs as JSON array
    #[arg(long)]
    pub stubs: Option<String>,
}

impl HealthCommand {
    pub fn run(&self, format: OutputFormat, verbose: bool) -> Result<()> {
        let config = get_config();

        if verbose {
            eprintln!(
                "Calculating health: refinement={}, stubs={}",
                self.refinement,
                self.stubs.as_deref().unwrap_or("[]")
            );
            eprintln!(
                "Using config: refinement_weight={}, stub_weight={}",
                config.health.refinement_weight, config.health.stub_weight
            );
        }

        // Validate refinement
        if !(0.0..=1.0).contains(&self.refinement) {
            anyhow::bail!("Refinement must be between 0.0 and 1.0");
        }

        // Parse stubs
        let stubs: Vec<Stub> = if let Some(json) = &self.stubs {
            serde_json::from_str(json).map_err(|e| anyhow::anyhow!("Invalid stubs JSON: {}", e))?
        } else {
            Vec::new()
        };

        // Calculate stub penalty using config
        let stub_penalty = calculate_stub_penalty_with_config(&stubs, &config.stub_penalties);

        // Calculate health using config
        let health = calculate_health_with_config(self.refinement, &stubs, config);

        match format {
            OutputFormat::Human => {
                // Use styled output
                print!("{}", format_health_output(health, self.refinement, stubs.len(), stub_penalty.min(1.0)));
            }
            _ => {
                let output = HealthOutput {
                    health,
                    refinement: self.refinement,
                    stub_count: stubs.len(),
                    stub_penalty: stub_penalty.min(1.0),
                };
                println!("{}", format_output(&output, format)?);
            }
        }

        Ok(())
    }
}
