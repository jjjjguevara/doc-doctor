//! Terminal Setup and Management
//!
//! Handles terminal initialization, restoration, and the main event loop.

use std::io::{self, Stdout};
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use super::app::{App, Event};

pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// Initialize the terminal for TUI mode
pub fn init() -> Result<Tui> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Restore the terminal to its original state
pub fn restore() -> Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

/// Handle panic by restoring terminal first
pub fn install_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic| {
        let _ = restore();
        original_hook(panic);
    }));
}

/// Poll for events with a timeout
pub fn poll_event(timeout: Duration) -> Result<Option<Event>> {
    if event::poll(timeout)? {
        match event::read()? {
            CrosstermEvent::Key(key) => Ok(Some(Event::Key(key))),
            CrosstermEvent::Mouse(mouse) => Ok(Some(Event::Mouse(mouse))),
            CrosstermEvent::Resize(w, h) => Ok(Some(Event::Resize(w, h))),
            _ => Ok(None),
        }
    } else {
        Ok(Some(Event::Tick))
    }
}

/// Handle common key events, returns true if the event was handled
pub fn handle_common_keys(app: &mut App, key: crossterm::event::KeyEvent) -> bool {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            app.should_quit = true;
            true
        }
        KeyCode::Char('?') => {
            app.mode = super::app::AppMode::Help;
            true
        }
        _ => false,
    }
}
