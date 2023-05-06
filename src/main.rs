mod decryption;
mod ui;
mod util;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use decryption::decrypt_key;
use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs, Wrap},
    Frame, Terminal,
};
use util::decode_hex;

use unicode_width::UnicodeWidthStr;

enum Tab {
    Encrypted,
    Decryption,
}

enum InputMode {
    Normal,
    Editing,
}

struct App<'a> {
    pub titles: Vec<&'a str>,
    pub current_tab: Tab,
    input: String,
    input_mode: InputMode,
    encrypted_messages: Vec<Vec<u8>>,
    decrypted_messages: Vec<Vec<u8>>,
    key: Vec<Option<u8>>,
    position: (usize, usize),
}

impl<'a> App<'a> {
    fn new() -> App<'a> {
        App {
            titles: vec!["Encrypted", "Decryption"],
            current_tab: Tab::Encrypted,
            input: String::new(),
            input_mode: InputMode::Normal,
            encrypted_messages: Vec::new(),
            decrypted_messages: Vec::new(),
            key: Vec::new(),
            position: (0, 0),
        }
    }

    pub fn toggle_tab(&mut self) {
        match self.current_tab {
            Tab::Encrypted => self.current_tab = Tab::Decryption,
            Tab::Decryption => self.current_tab = Tab::Encrypted,
        }
        self.position = (0, 0);
    }

    fn get_current_tab_index(&self) -> usize {
        match self.current_tab {
            Tab::Encrypted => 0,
            Tab::Decryption => 1,
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    //if let Err(err) = res {
    //    println!("{:?}", err)
    //}

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Tab => app.toggle_tab(),
                KeyCode::Right => app.toggle_tab(),
                KeyCode::Left => app.toggle_tab(),
                _ => match app.current_tab {
                    Tab::Encrypted => match app.input_mode {
                        InputMode::Normal => match key.code {
                            KeyCode::Char('e') => app.input_mode = InputMode::Editing,
                            _ => {}
                        },
                        InputMode::Editing => match key.code {
                            KeyCode::Right => match app.current_tab {
                                Tab::Encrypted => {
                                    if app.position.1 < app.input.len() {
                                        app.position.1 += 1
                                    }
                                }
                                Tab::Decryption => app.position.1 += 1,
                            },
                            KeyCode::Left => {
                                if app.position.1 > 0 {
                                    app.position.1 -= 1
                                }
                            }
                            KeyCode::Up => {
                                if app.position.0 > 0 {
                                    app.position.0 -= 1
                                }
                            }
                            KeyCode::Down => app.position.0 += 1,
                            KeyCode::Char(c) => {
                                app.input.push(c);
                                app.position.1 += 1;
                            }
                            KeyCode::Backspace => {
                                app.input.pop();
                                if app.position.1 > 0 {
                                    app.position.1 -= 1;
                                }
                            }
                            KeyCode::Enter => {
                                if app.input.len() % 2 == 0 {
                                    if let Ok(msg_bytes) = decode_hex(&app.input) {
                                        app.encrypted_messages.push(msg_bytes);
                                        app.decrypted_messages.push(vec![
                                            0;
                                            app.encrypted_messages
                                                .last()
                                                .unwrap()
                                                .len()
                                        ]);
                                        app.key = decrypt_key(&app.encrypted_messages);
                                        app.input_mode = InputMode::Normal;
                                    }
                                }
                                app.input.clear();
                                app.position = (0, 0);
                            }
                            KeyCode::Esc => {
                                app.input_mode = InputMode::Normal;
                            }
                            _ => {}
                        },
                    },
                    Tab::Decryption => todo!(),
                },
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(size);

    let block = Block::default();
    f.render_widget(block, size);
    let titles = app
        .titles
        .iter()
        .map(|t| Spans::from(vec![Span::styled(*t, Style::default())]))
        .collect();
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Menu"))
        .select(app.get_current_tab_index())
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Black),
        );
    f.render_widget(tabs, chunks[0]);
    match app.current_tab {
        Tab::Encrypted => draw_encrypted_messages_block(f, app, chunks[1]),
        Tab::Decryption => draw_decryption_block(f, app, chunks[1]),
    };
}

fn draw_encrypted_messages_block<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(4),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(area);

    let (msg, style) = match app.input_mode {
        InputMode::Normal => (
            vec![
                Span::raw("Press "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to start editing."),
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
        ),
        InputMode::Editing => (
            vec![
                Span::raw("Press "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to stop editing, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to record the message"),
            ],
            Style::default(),
        ),
    };
    let mut text = Text::from(Spans::from(msg));
    text.patch_style(style);
    let help_message = Paragraph::new(text);
    f.render_widget(help_message, chunks[0]);

    let input = Paragraph::new(app.input.as_ref())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Green),
        })
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::ALL).title("Add"));
    f.render_widget(input, chunks[1]);
    match app.input_mode {
        InputMode::Normal => {}
        InputMode::Editing => f.set_cursor(
            chunks[1].x + 1 + app.position.1 as u16,
            chunks[1].y + 1 + app.position.0 as u16,
        ),
    }

    let messages: Vec<ListItem> = app
        .encrypted_messages
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let mut text = String::new();
            text.push_str(format!("{}:", i).as_str());
            for byte in m {
                text.push_str(format!("{:02X}", byte).as_str());
            }
            ListItem::new(Spans::from(Span::raw(text)))
        })
        .collect();
    let messages =
        List::new(messages).block(Block::default().borders(Borders::ALL).title("Messages"));
    f.render_widget(messages, chunks[2]);
}

fn draw_decryption_block<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Min(1),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(area);

    let (msg, style) = match app.input_mode {
        InputMode::Normal => match app.encrypted_messages.is_empty() {
            false => (
                vec![
                    Span::raw("Press "),
                    Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to exit, "),
                    Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to start editing."),
                ],
                Style::default().add_modifier(Modifier::RAPID_BLINK),
            ),
            true => (
                vec![Span::raw(
                    "Add encrypted messages before attempting to decrypt.",
                )],
                Style::default().add_modifier(Modifier::RAPID_BLINK),
            ),
        },
        InputMode::Editing => (
            vec![
                Span::raw("Press "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to stop editing, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to record the message"),
            ],
            Style::default(),
        ),
    };
    let mut text = Text::from(Spans::from(msg));
    text.patch_style(style);
    let help_message = Paragraph::new(text);
    f.render_widget(help_message, chunks[0]);

    let messages: Vec<ListItem> = app
        .encrypted_messages
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let m_to_string: String = m
                .iter()
                .enumerate()
                .map(|(j, byte)| {
                    if let Some(key_char) = app.key[j] {
                        let c = *byte ^ key_char;
                        if c.is_ascii() {
                            return c as char;
                        }
                    }
                    return '_';
                })
                .collect();
            let content = vec![Spans::from(Span::raw(format!("{}: {}", i, m_to_string)))];
            ListItem::new(content)
        })
        .collect();
    let messages =
        List::new(messages).block(Block::default().borders(Borders::ALL).title("Messages"));
    f.render_widget(messages, chunks[1]);

    let key_string = app
        .key
        .iter()
        .map(|opt| match opt {
            Some(byte) => format!("{:02X}", byte),
            None => "_".to_string(),
        })
        .collect::<Vec<String>>()
        .join("");
    let input = Paragraph::new(key_string.as_ref())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        })
        .block(Block::default().borders(Borders::ALL).title("Key"));
    f.render_widget(input, chunks[2]);

    match app.input_mode {
        InputMode::Normal => {}
        InputMode::Editing => f.set_cursor(
            chunks[1].x + app.input.width() as u16 + 1 + app.position.1 as u16,
            chunks[1].y + 1 + app.position.0 as u16,
        ),
    }
}
