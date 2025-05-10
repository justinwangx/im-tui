use crate::error::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, Terminal};
use std::io;

/// Type alias for TUI results
pub type TuiResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Run a terminal UI with proper setup and teardown
pub fn run_terminal<F, T>(ui_func: F) -> Result<T>
where
    F: FnOnce(&mut Terminal<CrosstermBackend<io::Stdout>>) -> TuiResult<T>,
{
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the UI function
    let result = match ui_func(&mut terminal) {
        Ok(result) => {
            // Restore terminal
            disable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
            terminal.show_cursor()?;
            Ok(result)
        }
        Err(e) => {
            // Restore terminal on error
            disable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
            terminal.show_cursor()?;
            Err(crate::error::Error::Generic(format!("TUI error: {}", e)))
        }
    };

    result
}

/// Helper to poll for key events with a timeout
pub fn poll_event(timeout_ms: u64) -> io::Result<Option<Event>> {
    if event::poll(std::time::Duration::from_millis(timeout_ms))? {
        let event = event::read()?;
        Ok(Some(event))
    } else {
        Ok(None)
    }
}
