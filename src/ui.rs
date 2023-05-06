use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs, Wrap},
    Frame,
};
use unicode_width::UnicodeWidthStr;

use crate::app_state::{App, InputMode, Tab};

pub fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
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
