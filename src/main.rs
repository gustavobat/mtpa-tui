mod app_state;
mod decryption;
mod ui;
mod util;

use app_state::{App, InputMode, Tab};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use decryption::decrypt_key;
use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use ui::ui;
use util::decode_hex;

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut app = App::new();
    let _res = run_app(&mut terminal, &mut app);

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
