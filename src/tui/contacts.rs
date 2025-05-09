use crate::config::Config;
use crate::error::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::io;

pub struct ContactsView {
    config: Config,
    selected_index: usize,
}

impl ContactsView {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            selected_index: 0,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Run the app
        let result = run_app(&mut terminal, self);

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        if let Err(err) = result {
            println!("Error: {:?}", err);
        }

        Ok(())
    }
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    contacts_view: &mut ContactsView,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, contacts_view))?;

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('c')
                        if key.modifiers.contains(KeyModifiers::CONTROL) =>
                    {
                        return Ok(());
                    }
                    KeyCode::Up => {
                        if contacts_view.selected_index > 0 {
                            contacts_view.selected_index -= 1;
                        }
                    }
                    KeyCode::Down => {
                        let contact_count = contacts_view.config.contact_count();
                        if contacts_view.selected_index < contact_count.saturating_sub(1) {
                            contacts_view.selected_index += 1;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

fn ui(f: &mut Frame, contacts_view: &ContactsView) {
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
    let default_contact = if let Some(default) = contacts_view.config.default_contact() {
        match contacts_view.config.default_display_name() {
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
    let contacts: Vec<ListItem> = contacts_view
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
    state.select(Some(contacts_view.selected_index));

    f.render_stateful_widget(contacts_list, content_chunks[1], &mut state);
}
