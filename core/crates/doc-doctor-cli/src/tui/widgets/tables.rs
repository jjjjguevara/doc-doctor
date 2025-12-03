//! Styled Tables
//!
//! Beautiful table output for static CLI commands.

use console::style;
use tabled::{
    settings::{
        object::Rows,
        Alignment, Color, Modify, Padding, Style as TabledStyle,
    },
    Table, Tabled,
};

use crate::tui::theme::console_styles;

/// Table theme for Doc Doctor
pub fn apply_theme<T>(table: &mut Table) {
    table
        .with(TabledStyle::rounded())
        .with(Modify::new(Rows::first()).with(Color::FG_CYAN).with(Alignment::center()))
        .with(Padding::new(1, 1, 0, 0));
}

/// Create a styled key-value table
pub fn key_value_table(pairs: Vec<(&str, String)>) -> String {
    let mut output = String::new();

    for (key, value) in pairs {
        output.push_str(&format!(
            "{}: {}\n",
            style(key).dim(),
            style(&value).white()
        ));
    }

    output
}

/// Health table row
#[derive(Tabled)]
pub struct HealthRow {
    #[tabled(rename = "Metric")]
    pub metric: String,
    #[tabled(rename = "Value")]
    pub value: String,
    #[tabled(rename = "Status")]
    pub status: String,
}

impl HealthRow {
    pub fn new(metric: &str, value: f64, threshold: f64) -> Self {
        let status = if value >= threshold {
            console_styles::success("●").to_string()
        } else if value >= threshold * 0.7 {
            console_styles::warning("●").to_string()
        } else {
            console_styles::error("●").to_string()
        };

        Self {
            metric: metric.to_string(),
            value: format!("{:.2}", value),
            status,
        }
    }
}

/// Stub table row
#[derive(Tabled)]
pub struct StubRow {
    #[tabled(rename = "ID")]
    pub id: String,
    #[tabled(rename = "Type")]
    pub stub_type: String,
    #[tabled(rename = "Form")]
    pub form: String,
    #[tabled(rename = "Description")]
    pub description: String,
    #[tabled(rename = "U")]
    pub urgency: String,
    #[tabled(rename = "I")]
    pub impact: String,
    #[tabled(rename = "C")]
    pub complexity: String,
}

/// Document table row
#[derive(Tabled)]
pub struct DocumentRow {
    #[tabled(rename = "Health")]
    pub health: String,
    #[tabled(rename = "Title")]
    pub title: String,
    #[tabled(rename = "Audience")]
    pub audience: String,
    #[tabled(rename = "Form")]
    pub form: String,
    #[tabled(rename = "Stubs")]
    pub stubs: String,
}

/// Dimension table row
#[derive(Tabled)]
pub struct DimensionRow {
    #[tabled(rename = "Dimension")]
    pub dimension: String,
    #[tabled(rename = "Value")]
    pub value: String,
    #[tabled(rename = "Description")]
    pub description: String,
}

/// Create styled output for parse command
pub fn format_parse_output(
    title: Option<&str>,
    refinement: f64,
    audience: &str,
    form: &str,
    origin: &str,
    stub_count: usize,
) -> String {
    let mut output = String::new();

    // Header
    output.push_str(&format!(
        "\n{}\n",
        console_styles::title("Document Properties")
    ));
    output.push_str(&format!("{}\n\n", "─".repeat(40)));

    // Title
    if let Some(t) = title {
        output.push_str(&format!(
            "  {} {}\n\n",
            console_styles::dim("Title:"),
            console_styles::bold(t)
        ));
    }

    // Core properties
    output.push_str(&format!(
        "  {} {}\n",
        console_styles::dim("Refinement:"),
        format_health_value(refinement)
    ));

    output.push_str(&format!(
        "  {}   {}\n",
        console_styles::dim("Audience:"),
        format_audience(audience)
    ));

    output.push_str(&format!(
        "  {}       {}\n",
        console_styles::dim("Form:"),
        style(form).white()
    ));

    output.push_str(&format!(
        "  {}     {}\n",
        console_styles::dim("Origin:"),
        style(origin).white()
    ));

    let stub_str = stub_count.to_string();
    let stub_styled = if stub_count > 0 {
        console_styles::warning(&stub_str)
    } else {
        console_styles::success(&stub_str)
    };
    output.push_str(&format!(
        "  {}      {}\n",
        console_styles::dim("Stubs:"),
        stub_styled
    ));

    output
}

/// Create styled output for health command
pub fn format_health_output(health: f64, refinement: f64, stub_count: usize, stub_penalty: f64) -> String {
    let mut output = String::new();

    // Header with large health value
    output.push_str(&format!(
        "\n{}  {}\n",
        console_styles::title("Health Score"),
        format_health_large(health)
    ));
    output.push_str(&format!("{}\n\n", "─".repeat(40)));

    // Breakdown
    output.push_str(&format!(
        "  {} {:.2} × 0.70 = {:.2}\n",
        console_styles::dim("Refinement:"),
        refinement,
        refinement * 0.7
    ));

    output.push_str(&format!(
        "  {} {:.2} × 0.30 = {:.2}\n",
        console_styles::dim("Stub Factor:"),
        1.0 - stub_penalty,
        (1.0 - stub_penalty) * 0.3
    ));

    if stub_count > 0 {
        output.push_str(&format!(
            "\n  {} {} stubs with {:.2} total penalty\n",
            console_styles::warning("⚠"),
            stub_count,
            stub_penalty
        ));
    }

    output
}

/// Create styled output for usefulness command
pub fn format_usefulness_output(
    margin: f64,
    is_useful: bool,
    refinement: f64,
    audience: &str,
    gate: f64,
) -> String {
    let mut output = String::new();

    // Header
    let status = if is_useful {
        console_styles::success("✓ USEFUL")
    } else {
        console_styles::error("✗ NOT USEFUL")
    };

    output.push_str(&format!(
        "\n{} for {} audience\n",
        status,
        format_audience(audience)
    ));
    output.push_str(&format!("{}\n\n", "─".repeat(40)));

    // Details
    output.push_str(&format!(
        "  {} {}\n",
        console_styles::dim("Refinement:"),
        format_health_value(refinement)
    ));

    output.push_str(&format!(
        "  {}       {:.2}\n",
        console_styles::dim("Gate:"),
        gate
    ));

    output.push_str(&format!(
        "  {}     {:+.2}\n",
        console_styles::dim("Margin:"),
        margin
    ));

    // Visual bar
    let bar = create_margin_bar(refinement, gate);
    output.push_str(&format!("\n  {}\n", bar));

    output
}

/// Format health value with color
fn format_health_value(value: f64) -> console::StyledObject<String> {
    let text = format!("{:.2}", value);
    if value >= 0.8 {
        style(text).green()
    } else if value >= 0.5 {
        style(text).yellow()
    } else {
        style(text).red()
    }
}

/// Format large health display
fn format_health_large(value: f64) -> String {
    let percentage = (value * 100.0) as u32;
    let styled = if value >= 0.8 {
        style(format!("{}%", percentage)).green().bold()
    } else if value >= 0.5 {
        style(format!("{}%", percentage)).yellow().bold()
    } else {
        style(format!("{}%", percentage)).red().bold()
    };
    styled.to_string()
}

/// Format audience with color
fn format_audience(audience: &str) -> console::StyledObject<&str> {
    match audience.to_lowercase().as_str() {
        "personal" => style(audience).blue(),
        "internal" => style(audience).green(),
        "trusted" => style(audience).yellow(),
        "public" => style(audience).red(),
        _ => style(audience).white(),
    }
}

/// Create visual margin bar
fn create_margin_bar(refinement: f64, gate: f64) -> String {
    let width = 30;
    let gate_pos = (gate * width as f64) as usize;
    let ref_pos = (refinement * width as f64) as usize;

    let mut bar = String::new();
    bar.push_str("  0 ");

    for i in 0..width {
        if i == gate_pos {
            bar.push_str(&style("│").yellow().to_string());
        } else if i < ref_pos {
            if i >= gate_pos {
                bar.push_str(&style("█").green().to_string());
            } else {
                bar.push_str(&style("█").red().to_string());
            }
        } else {
            bar.push_str(&style("░").dim().to_string());
        }
    }

    bar.push_str(" 1.0");
    bar.push_str(&format!("\n      {}gate", " ".repeat(gate_pos)));

    bar
}

/// Create a simple spinner for inline use
pub fn spinner_chars() -> &'static [char] {
    &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏']
}
