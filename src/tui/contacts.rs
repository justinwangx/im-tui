use crate::config::Config;
use crate::error::Result;
use crate::tui::common::{run_terminal, TuiResult};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

/// The contacts view for managing contacts
pub struct ContactsView {
    config: Config,
    selected_index: usize,
}

impl ContactsView {
    /// Create a new contacts view
    pub fn new(config: Config) -> Self {
        Self {
            config,
            selected_index: 0,
        }
    }

    /// Run the contacts view
    pub fn run(&mut self) -> Result<()> {
        run_terminal(|terminal| self.run_ui(terminal))
    }

    /// Handle the UI loop
    fn run_ui(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> TuiResult<()> {
        loop {
            // Draw UI
            terminal.draw(|f| self.render(f))?;

            // Handle events
            if let Some(event) = crate::tui::common::poll_event(50)? {
                if let Event::Key(key) = event {
                    match key.code {
                        KeyCode::Esc => {
                            return Ok(());
                        }
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            return Ok(());
                        }
                        KeyCode::Up => {
                            if self.selected_index > 0 {
                                self.selected_index -= 1;
                            }
                        }
                        KeyCode::Down => {
                            let contact_count = self.config.contact_count();
                            if self.selected_index < contact_count.saturating_sub(1) {
                                self.selected_index += 1;
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
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(0),    // Content
            ])
            .split(f.size());

        // Title
        let title = Paragraph::new("Contacts")
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        f.render_widget(title, chunks[0]);

        // Content
        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Default contact section
                Constraint::Min(0),    // Named contacts section
            ])
            .split(chunks[1]);

        // Default contact section
        let default_contact = if let Some(default) = self.config.default_contact() {
            match self.config.default_display_name() {
                Some(display) => format!("{} ({})", display, default),
                None => default.clone(),
            }
        } else {
            "None".to_string()
        };

        let default_section = Paragraph::new(default_contact).block(
            Block::default()
                .title("Default Contact")
                .borders(Borders::ALL),
        );
        f.render_widget(default_section, content_chunks[0]);

        // Named contacts section
        let contacts: Vec<ListItem> = self
            .config
            .list_contacts()
            .into_iter()
            .map(|(name, entry)| {
                let display = match &entry.display_name {
                    Some(display) => format!("{} ({})", display, entry.identifier),
                    None => entry.identifier.clone(),
                };
                ListItem::new(format!("{}: {}", name, display))
            })
            .collect();

        let contacts_list = List::new(contacts)
            .block(
                Block::default()
                    .title("Named Contacts")
                    .borders(Borders::ALL),
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");

        let mut state = ListState::default();
        state.select(Some(self.selected_index));

        f.render_stateful_widget(contacts_list, content_chunks[1], &mut state);
    }
}

/// Convenience function to run the contacts TUI
pub fn run_contacts_tui(config: Config) -> Result<()> {
    let mut contacts_view = ContactsView::new(config);
    contacts_view.run()
}
