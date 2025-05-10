use crate::db::MessageDB;
use crate::error::Result;
use crate::sender::Sender;
use crate::tui::common::{run_terminal, TuiResult};
use chrono::{DateTime, Local};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use std::time::{Duration, Instant};

/// The chat view for messaging with a contact
pub struct ChatView {
    messages: Vec<(Option<String>, DateTime<Local>, Option<String>, bool)>,
    input: String,
    scroll: usize,
    contact: String,
    display_name: String,
    should_reset_scroll: bool,
    sender: Sender,
}

impl ChatView {
    /// Create a new chat view for a contact
    pub fn new(contact: String, display_name: String) -> Self {
        Self {
            messages: Vec::new(),
            input: String::new(),
            scroll: 0,
            contact: contact.clone(),
            display_name,
            should_reset_scroll: true,
            sender: Sender::new(contact),
        }
    }

    /// Load messages from the database
    pub fn load_messages(&mut self) -> Result<()> {
        let db = MessageDB::open()?;
        let mut messages = db.get_messages(&self.contact)?;
        // Reverse the messages so oldest are at the top
        messages.reverse();
        self.messages = messages;
        self.should_reset_scroll = true;
        Ok(())
    }

    /// Send a message to the contact
    pub fn send_message(&mut self, text: &str) -> Result<()> {
        self.sender.send_message(text)?;
        // Reload messages to show the sent message
        self.load_messages()?;
        Ok(())
    }

    /// Run the chat view
    pub fn run(&mut self) -> Result<()> {
        run_terminal(|terminal| self.run_ui(terminal))
    }

    /// Handle the UI loop
    fn run_ui(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> TuiResult<()> {
        // Load messages
        self.load_messages()?;

        let tick_rate = Duration::from_millis(200);
        let mut last_tick = Instant::now();

        loop {
            // Reset scroll position if needed
            if self.should_reset_scroll && !self.messages.is_empty() {
                let size = terminal.size()?;
                let visible_messages = self.messages.len().min((size.height - 6) as usize);
                self.scroll = self.messages.len().saturating_sub(visible_messages);
                self.should_reset_scroll = false;
            }

            // Draw UI
            terminal.draw(|f| self.render(f))?;

            // Handle events with timeout
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if let Some(event) = crate::tui::common::poll_event(timeout.as_millis() as u64)? {
                if let Event::Key(key) = event {
                    match key.code {
                        KeyCode::Esc => {
                            return Ok(());
                        }
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            return Ok(());
                        }
                        KeyCode::Char(c) => {
                            self.input.push(c);
                        }
                        KeyCode::Backspace => {
                            self.input.pop();
                        }
                        KeyCode::Enter => {
                            if !self.input.is_empty() {
                                let input = self.input.clone();
                                if let Err(e) = self.send_message(&input) {
                                    eprintln!("Error sending message: {}", e);
                                }
                                self.input.clear();
                            }
                        }
                        KeyCode::Up => {
                            if self.scroll > 0 {
                                self.scroll -= 1;
                            }
                        }
                        KeyCode::Down => {
                            let size = terminal.size()?;
                            let visible_messages =
                                self.messages.len().min((size.height - 6) as usize);
                            let max_scroll = self.messages.len().saturating_sub(visible_messages);
                            if self.scroll < max_scroll {
                                self.scroll += 1;
                            }
                        }
                        _ => {}
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
            }
        }
    }

    /// Render the UI
    fn render(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(0),    // Messages
                Constraint::Length(3), // Input
            ])
            .split(f.size());

        // Title
        let title = Paragraph::new(self.display_name.clone())
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        f.render_widget(title, chunks[0]);

        // Messages
        let messages_area = chunks[1];
        let visible_messages = self.messages.len().min(messages_area.height as usize);
        let start_idx = self.scroll;
        let end_idx = (start_idx + visible_messages).min(self.messages.len());

        let messages_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(1); visible_messages])
            .split(messages_area);

        // Calculate the visible range of messages
        let visible_range = start_idx..end_idx;

        for (i, idx) in visible_range.enumerate() {
            let (text, time, msg_type, is_from_me) = &self.messages[idx];
            let content = if let Some(text) = text {
                text.clone()
            } else if let Some(msg_type) = msg_type {
                format!("[{}]", msg_type)
            } else {
                "<empty message>".to_string()
            };

            let alignment = if *is_from_me {
                Alignment::Right
            } else {
                Alignment::Left
            };

            let style = if *is_from_me {
                Style::default().fg(Color::Blue)
            } else {
                Style::default().fg(Color::Green)
            };

            let message = Paragraph::new(format!("{}: {}", time.format("%H:%M"), content))
                .style(style)
                .alignment(alignment)
                .block(Block::default().borders(Borders::NONE));

            f.render_widget(message, messages_chunks[i]);
        }

        // Input
        let input = Paragraph::new(Text::from(self.input.as_str()))
            .block(Block::default().title("Input").borders(Borders::ALL));
        f.render_widget(input, chunks[2]);
    }
}

/// Convenience function to run the chat TUI
pub fn run_chat_tui(contact: String, display_name: String) -> Result<()> {
    let mut chat = ChatView::new(contact, display_name);
    chat.run()
}
