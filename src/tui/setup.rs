use crate::config::Config;
use crate::error::Result;
use crate::formatter::format_phone_number;
use crate::tui::common::{run_terminal, TuiResult};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

/// Input field enum for the setup view
enum InputField {
    Contact,
    DisplayName,
}

/// The setup view for configuring default contact
pub struct SetupView {
    contact_input: String,
    display_name_input: String,
    active_field: InputField,
    config: Config,
}

impl SetupView {
    /// Create a new setup view
    pub fn new() -> Self {
        Self {
            contact_input: String::new(),
            display_name_input: String::new(),
            active_field: InputField::Contact,
            config: Config::default(),
        }
    }

    /// Get the configuration
    pub fn get_config(&self) -> Config {
        self.config.clone()
    }

    /// Run the setup view
    pub fn run(&mut self) -> Result<Config> {
        run_terminal(|terminal| self.run_ui(terminal))
    }

    /// Handle the UI loop
    fn run_ui(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> TuiResult<Config> {
        loop {
            // Draw UI
            terminal.draw(|f| self.render(f))?;

            // Hide the terminal cursor since we have our own cursor indicator
            terminal.hide_cursor()?;

            // Handle events
            if let Some(event) = crate::tui::common::poll_event(100)? {
                if let Event::Key(key) = event {
                    match key.code {
                        KeyCode::Esc => {
                            return Ok(self.get_config());
                        }
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            return Ok(self.get_config());
                        }
                        KeyCode::Tab => {
                            // Switch between input fields
                            self.active_field = match self.active_field {
                                InputField::Contact => InputField::DisplayName,
                                InputField::DisplayName => InputField::Contact,
                            };
                        }
                        KeyCode::Char(c) => {
                            // Add character to the active input field
                            match self.active_field {
                                InputField::Contact => self.contact_input.push(c),
                                InputField::DisplayName => self.display_name_input.push(c),
                            }
                        }
                        KeyCode::Backspace => {
                            // Remove character from the active input field
                            match self.active_field {
                                InputField::Contact => {
                                    self.contact_input.pop();
                                }
                                InputField::DisplayName => {
                                    self.display_name_input.pop();
                                }
                            }
                        }
                        KeyCode::Enter => {
                            // Save if contact is not empty
                            if !self.contact_input.is_empty() {
                                let formatted_contact = format_phone_number(&self.contact_input);
                                self.config.set_default_contact(formatted_contact);

                                if !self.display_name_input.is_empty() {
                                    self.config
                                        .set_default_display_name(self.display_name_input.clone());
                                }

                                // Return from the setup TUI
                                return Ok(self.get_config());
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /// Render the UI
    fn render(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(1), // Spacer
                Constraint::Length(3), // Contact Input
                Constraint::Length(1), // Spacer
                Constraint::Length(3), // Display Name Input
                Constraint::Length(1), // Spacer
                Constraint::Length(3), // Instructions
                Constraint::Min(0),    // Space
            ])
            .split(f.size());

        // Title
        let title = Paragraph::new("gf")
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // Contact input
        let contact_block_style = if matches!(self.active_field, InputField::Contact) {
            Style::default().fg(Color::Blue)
        } else {
            Style::default().fg(Color::Gray)
        };

        // Add a blinking cursor indicator for the active field
        let contact_text = if matches!(self.active_field, InputField::Contact) {
            format!("{}▎", self.contact_input)
        } else {
            self.contact_input.clone()
        };

        let contact_input = Paragraph::new(contact_text)
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .title("Enter default contact number/email (required)")
                    .title_style(contact_block_style)
                    .borders(Borders::ALL)
                    .border_style(contact_block_style),
            );
        f.render_widget(contact_input, chunks[2]);

        // Display name input
        let name_block_style = if matches!(self.active_field, InputField::DisplayName) {
            Style::default().fg(Color::Blue)
        } else {
            Style::default().fg(Color::Gray)
        };

        // Add a blinking cursor indicator for the active field
        let display_name_text = if matches!(self.active_field, InputField::DisplayName) {
            format!("{}▎", self.display_name_input)
        } else {
            self.display_name_input.clone()
        };

        let display_name_input = Paragraph::new(display_name_text)
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .title("Enter default contact display name (optional)")
                    .title_style(name_block_style)
                    .borders(Borders::ALL)
                    .border_style(name_block_style),
            );
        f.render_widget(display_name_input, chunks[4]);

        // Instructions styled with iMessage blue for emphasis
        let instructions = Paragraph::new(Text::from(vec![Line::from(vec![
            Span::styled(
                "Tab",
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Switch fields | "),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Save | "),
            Span::styled(
                "Esc",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Cancel"),
        ])]))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
        f.render_widget(instructions, chunks[6]);
    }
}

/// Convenience function to run the setup TUI
pub fn run_setup_tui() -> Result<Config> {
    let mut setup = SetupView::new();
    setup.run()
}
