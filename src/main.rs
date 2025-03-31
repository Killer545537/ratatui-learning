mod app;
mod system_data;
mod ui;
mod utils;

use crate::app::App;
use crate::ui::run_app;
use anyhow::{Context, Result};
use ratatui::Terminal;

use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use std::io;

fn main() -> Result<()> {
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?; // Enter a new screen and enable mouse control
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?; // Initialize the terminal

    let app = App::new();
    let res = run_app(&mut terminal, app); // Main app logic

    disable_raw_mode()?;
    execute!(
        // Close the new window
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Handle potential errors
    if let Err(err) = res {
        eprintln!("Error: {err:?}");
    }

    Ok(())
}
