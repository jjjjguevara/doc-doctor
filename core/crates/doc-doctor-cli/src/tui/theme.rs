//! Doc Doctor Theme
//!
//! Color palette and styling for the TUI and console output.

use ratatui::style::{Color, Modifier, Style};

/// Doc Doctor color palette
pub mod colors {
    use super::Color;

    // Primary colors
    pub const PRIMARY: Color = Color::Rgb(99, 102, 241);      // Indigo
    pub const PRIMARY_LIGHT: Color = Color::Rgb(165, 180, 252);
    pub const PRIMARY_DARK: Color = Color::Rgb(67, 56, 202);

    // Semantic colors
    pub const SUCCESS: Color = Color::Rgb(34, 197, 94);       // Green
    pub const WARNING: Color = Color::Rgb(234, 179, 8);       // Yellow
    pub const ERROR: Color = Color::Rgb(239, 68, 68);         // Red
    pub const INFO: Color = Color::Rgb(59, 130, 246);         // Blue

    // Health gradient (red -> yellow -> green)
    pub const HEALTH_LOW: Color = Color::Rgb(239, 68, 68);    // Red
    pub const HEALTH_MED: Color = Color::Rgb(234, 179, 8);    // Yellow
    pub const HEALTH_HIGH: Color = Color::Rgb(34, 197, 94);   // Green

    // Audience colors
    pub const AUDIENCE_PERSONAL: Color = Color::Rgb(147, 197, 253);  // Light blue
    pub const AUDIENCE_INTERNAL: Color = Color::Rgb(134, 239, 172);  // Light green
    pub const AUDIENCE_TRUSTED: Color = Color::Rgb(253, 224, 71);    // Light yellow
    pub const AUDIENCE_PUBLIC: Color = Color::Rgb(252, 165, 165);    // Light red

    // Stub form colors
    pub const STUB_TRANSIENT: Color = Color::Rgb(156, 163, 175);     // Gray
    pub const STUB_PERSISTENT: Color = Color::Rgb(251, 191, 36);     // Amber
    pub const STUB_BLOCKING: Color = Color::Rgb(248, 113, 113);      // Red
    pub const STUB_STRUCTURAL: Color = Color::Rgb(167, 139, 250);    // Purple

    // UI colors
    pub const BACKGROUND: Color = Color::Rgb(15, 23, 42);     // Slate 900
    pub const SURFACE: Color = Color::Rgb(30, 41, 59);        // Slate 800
    pub const BORDER: Color = Color::Rgb(71, 85, 105);        // Slate 600
    pub const TEXT: Color = Color::Rgb(248, 250, 252);        // Slate 50
    pub const TEXT_DIM: Color = Color::Rgb(148, 163, 184);    // Slate 400
    pub const TEXT_MUTED: Color = Color::Rgb(100, 116, 139);  // Slate 500

    // Accent colors for variety
    pub const ACCENT_CYAN: Color = Color::Rgb(34, 211, 238);
    pub const ACCENT_PINK: Color = Color::Rgb(236, 72, 153);
    pub const ACCENT_ORANGE: Color = Color::Rgb(251, 146, 60);
}

/// Predefined styles
pub mod styles {
    use super::*;

    pub fn title() -> Style {
        Style::default()
            .fg(colors::PRIMARY_LIGHT)
            .add_modifier(Modifier::BOLD)
    }

    pub fn subtitle() -> Style {
        Style::default()
            .fg(colors::TEXT_DIM)
    }

    pub fn label() -> Style {
        Style::default()
            .fg(colors::TEXT_MUTED)
    }

    pub fn value() -> Style {
        Style::default()
            .fg(colors::TEXT)
    }

    pub fn highlight() -> Style {
        Style::default()
            .fg(colors::PRIMARY)
            .add_modifier(Modifier::BOLD)
    }

    pub fn success() -> Style {
        Style::default()
            .fg(colors::SUCCESS)
    }

    pub fn warning() -> Style {
        Style::default()
            .fg(colors::WARNING)
    }

    pub fn error() -> Style {
        Style::default()
            .fg(colors::ERROR)
    }

    pub fn info() -> Style {
        Style::default()
            .fg(colors::INFO)
    }

    pub fn border() -> Style {
        Style::default()
            .fg(colors::BORDER)
    }

    pub fn border_focused() -> Style {
        Style::default()
            .fg(colors::PRIMARY)
    }

    pub fn selected() -> Style {
        Style::default()
            .bg(colors::SURFACE)
            .add_modifier(Modifier::BOLD)
    }

    pub fn line_number() -> Style {
        Style::default()
            .fg(colors::TEXT_MUTED)
    }
}

/// Get color for health value (0.0 - 1.0)
pub fn health_color(health: f64) -> Color {
    if health >= 0.8 {
        colors::HEALTH_HIGH
    } else if health >= 0.5 {
        colors::HEALTH_MED
    } else {
        colors::HEALTH_LOW
    }
}

/// Get style for health value
pub fn health_style(health: f64) -> Style {
    Style::default().fg(health_color(health))
}

/// Get color for audience level
pub fn audience_color(audience: &str) -> Color {
    match audience.to_lowercase().as_str() {
        "personal" => colors::AUDIENCE_PERSONAL,
        "internal" => colors::AUDIENCE_INTERNAL,
        "trusted" => colors::AUDIENCE_TRUSTED,
        "public" => colors::AUDIENCE_PUBLIC,
        _ => colors::TEXT_DIM,
    }
}

/// Get color for stub form
pub fn stub_form_color(form: &str) -> Color {
    match form.to_lowercase().as_str() {
        "transient" => colors::STUB_TRANSIENT,
        "persistent" => colors::STUB_PERSISTENT,
        "blocking" => colors::STUB_BLOCKING,
        "structural" => colors::STUB_STRUCTURAL,
        _ => colors::TEXT_DIM,
    }
}

/// Get color for document form/lifecycle
pub fn form_color(form: &str) -> Color {
    match form.to_lowercase().as_str() {
        "transient" => colors::TEXT_DIM,
        "developing" => colors::WARNING,
        "stable" => colors::SUCCESS,
        "evergreen" => colors::ACCENT_CYAN,
        "canonical" => colors::PRIMARY,
        _ => colors::TEXT_DIM,
    }
}

/// Get color for document origin
pub fn origin_color(origin: &str) -> Color {
    match origin.to_lowercase().as_str() {
        "human" => colors::SUCCESS,
        "ai" => colors::ACCENT_PINK,
        "ai_assisted" | "ai-assisted" => colors::ACCENT_ORANGE,
        "imported" => colors::INFO,
        "derived" => colors::ACCENT_CYAN,
        "collaborative" => colors::PRIMARY_LIGHT,
        _ => colors::TEXT_DIM,
    }
}

/// Console styling helpers (for non-TUI output)
pub mod console_styles {
    use console::{style, StyledObject};

    pub fn title(text: &str) -> StyledObject<&str> {
        style(text).cyan().bold()
    }

    pub fn success(text: &str) -> StyledObject<&str> {
        style(text).green()
    }

    pub fn warning(text: &str) -> StyledObject<&str> {
        style(text).yellow()
    }

    pub fn error(text: &str) -> StyledObject<&str> {
        style(text).red()
    }

    pub fn info(text: &str) -> StyledObject<&str> {
        style(text).blue()
    }

    pub fn dim(text: &str) -> StyledObject<&str> {
        style(text).dim()
    }

    pub fn bold(text: &str) -> StyledObject<&str> {
        style(text).bold()
    }

    pub fn health(value: f64) -> StyledObject<String> {
        let text = format!("{:.2}", value);
        if value >= 0.8 {
            style(text).green()
        } else if value >= 0.5 {
            style(text).yellow()
        } else {
            style(text).red()
        }
    }

    pub fn useful(is_useful: bool) -> StyledObject<&'static str> {
        if is_useful {
            style("Yes").green()
        } else {
            style("No").red()
        }
    }
}
