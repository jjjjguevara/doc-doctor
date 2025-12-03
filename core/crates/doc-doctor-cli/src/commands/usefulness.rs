//! Usefulness Command
//!
//! Calculate usefulness margin for an audience.

use anyhow::Result;
use clap::Args;

use doc_doctor_domain::{calculate_usefulness_with_config, Audience};

use crate::config::get_config;
use crate::output::{format_output, OutputFormat, UsefulnessOutput};
use crate::tui::widgets::tables::format_usefulness_output;

#[derive(Args)]
pub struct UsefulnessCommand {
    /// Refinement score (0.0-1.0)
    #[arg(long)]
    pub refinement: f64,

    /// Target audience (personal, internal, trusted, public)
    #[arg(long)]
    pub audience: String,
}

impl UsefulnessCommand {
    pub fn run(&self, format: OutputFormat, verbose: bool) -> Result<()> {
        let config = get_config();

        if verbose {
            eprintln!(
                "Calculating usefulness: refinement={}, audience={}",
                self.refinement, self.audience
            );
            eprintln!(
                "Using gates: personal={}, internal={}, trusted={}, public={}",
                config.audience_gates.personal,
                config.audience_gates.internal,
                config.audience_gates.trusted,
                config.audience_gates.public
            );
        }

        // Validate refinement
        if !(0.0..=1.0).contains(&self.refinement) {
            anyhow::bail!("Refinement must be between 0.0 and 1.0");
        }

        // Parse audience
        let audience: Audience = self
            .audience
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid audience: {}", self.audience))?;

        // Calculate usefulness using config
        let result = calculate_usefulness_with_config(self.refinement, audience, config);

        match format {
            OutputFormat::Human => {
                // Use styled output
                print!("{}", format_usefulness_output(
                    result.margin,
                    result.is_useful,
                    result.refinement,
                    &result.audience.to_string(),
                    result.gate,
                ));
            }
            _ => {
                let output = UsefulnessOutput {
                    margin: result.margin,
                    is_useful: result.is_useful,
                    refinement: result.refinement,
                    audience: result.audience.to_string(),
                    gate: result.gate,
                };
                println!("{}", format_output(&output, format)?);
            }
        }

        Ok(())
    }
}
