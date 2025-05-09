use crate::db::MessageDB;
use crate::error::Result;
use crate::sender::Sender;
use chrono::{DateTime, Local};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use std::{
    io,
    time::{Duration, Instant},
};

pub struct App {
    messages: Vec<(Option<String>, DateTime<Local>, Option<String>, bool)>,
    input: String,
    scroll: usize,
    contact: String,
    display_name: String,
    should_reset_scroll: bool,
    sender: Sender,
}

impl App {
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

    pub fn load_messages(&mut self) -> Result<()> {
        let db = MessageDB::open()?;
        let mut messages = db.get_messages(&self.contact)?;
        // Reverse the messages so oldest are at the top
        messages.reverse();
        self.messages = messages;
        self.should_reset_scroll = true;
        Ok(())
    }

    pub fn send_message(&mut self, text: &str) -> Result<()> {
        self.sender.send_message(text)?;
        // Reload messages to show the sent message
        self.load_messages()?;
        Ok(())
    }
}

pub fn run_tui(contact: String, display_name: String) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let mut app = App::new(contact, display_name);
    let result = match app.load_messages() {
        Ok(_) => run_app(&mut terminal, app),
        Err(e) => {
            // Restore terminal before returning error
            disable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
            terminal.show_cursor()?;
            return Err(e);
        }
    };

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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    let tick_rate = Duration::from_millis(200);
    let mut last_tick = Instant::now();

    loop {
        // Reset scroll position if needed
        if app.should_reset_scroll && !app.messages.is_empty() {
            let size = terminal.size()?;
            let visible_messages = app.messages.len().min((size.height - 6) as usize);
            app.scroll = app.messages.len().saturating_sub(visible_messages);
            app.should_reset_scroll = false;
        }

        terminal.draw(|f| ui(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => return Ok(()),
                    KeyCode::Char(c) => app.input.push(c),
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Enter => {
                        if !app.input.is_empty() {
                            let input = app.input.clone();
                            if let Err(e) = app.send_message(&input) {
                                eprintln!("Error sending message: {}", e);
                            }
                            app.input.clear();
                        }
                    }
                    KeyCode::Up => {
                        if app.scroll > 0 {
                            app.scroll -= 1;
                        }
                    }
                    KeyCode::Down => {
                        let size = terminal.size()?;
                        let visible_messages = app.messages.len().min((size.height - 6) as usize);
                        let max_scroll = app.messages.len().saturating_sub(visible_messages);
                        if app.scroll < max_scroll {
                            app.scroll += 1;
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

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(0),    // Messages
            Constraint::Length(3), // Input
        ])
        .split(f.size());

    // Title
    let title = Paragraph::new(format!(
        "Chat with {} (Press 'Esc' to quit)",
        app.display_name
    ))
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    // Messages
    let messages_area = chunks[1];
    let visible_messages = app.messages.len().min(messages_area.height as usize);
    let start_idx = app.scroll;
    let end_idx = (start_idx + visible_messages).min(app.messages.len());

    let messages_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(1); visible_messages])
        .split(messages_area);

    // Calculate the visible range of messages
    let visible_range = start_idx..end_idx;

    for (i, idx) in visible_range.enumerate() {
        let (text, time, msg_type, is_from_me) = &app.messages[idx];
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
    let input = Paragraph::new(Text::from(app.input.as_str()))
        .block(Block::default().title("Input").borders(Borders::ALL));
    f.render_widget(input, chunks[2]);
}
