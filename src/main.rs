use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::{cursor, execute, queue, style, terminal};
use std::io::{self, Write};
use std::option::Option;

#[derive(PartialEq, Eq)]
pub enum KeyEvent {
    Ctrl(char),
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Null,
}

fn read_key() -> io::Result<KeyEvent> {
    loop {
        if let Event::Key(key_event) = event::read()? {
            let key = match key_event.modifiers {
                KeyModifiers::CONTROL => {
                    if let KeyCode::Char(q) = key_event.code {
                        KeyEvent::Ctrl(q)
                    } else {
                        KeyEvent::Null
                    }
                }
                _ => match key_event.code {
                    KeyCode::Up => KeyEvent::ArrowUp,
                    KeyCode::Down => KeyEvent::ArrowDown,
                    KeyCode::Left => KeyEvent::ArrowLeft,
                    KeyCode::Right => KeyEvent::ArrowRight,
                    _ => KeyEvent::Null,
                },
            };
            return Ok(key);
        };
        return Err(io::Error::new(io::ErrorKind::Other, "Fail to read key"));
    }
}

fn draw_rows(rows: u16, mut stdout: &io::Stdout) -> io::Result<()> {
    for y in 0..rows {
        if y == rows / 3 {
            let welcome = "Kilo editor -- version 0.0.1";
            queue!(
                stdout,
                style::Print("~"),
                style::Print(format!("{:^width$}", welcome, width = rows.into()))
            )?;
        } else {
            queue!(
                stdout,
                style::Print("~"),
                terminal::Clear(terminal::ClearType::UntilNewLine)
            )?;
        }
        if y < rows - 1 {
            queue!(stdout, style::Print("\r\n"))?;
        }
    }
    Ok(())
}

pub struct Editor {
    cursor_x: u16,
    cursor_y: u16,
    size: terminal::WindowSize,
}

impl Editor {
    pub fn new() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        Ok(Self {
            cursor_x: 0,
            cursor_y: 0,
            size: terminal::window_size().expect("Failed to get window size"),
        })
    }

    fn refresh_screen(&self) -> io::Result<()> {
        let mut stdout = io::stdout();
        queue!(stdout, cursor::Hide, cursor::MoveTo(0, 0))?;
        draw_rows(self.size.rows, &stdout)?;
        queue!(
            stdout,
            cursor::MoveTo(self.cursor_x, self.cursor_y),
            cursor::Show
        )?;
        stdout.flush()?;
        Ok(())
    }

    fn move_cursor(&mut self, key_event: KeyEvent) {
        match key_event {
            KeyEvent::ArrowLeft => {
                self.cursor_x = if self.cursor_x > 0 {
                    self.cursor_x - 1
                } else {
                    0
                }
            }
            KeyEvent::ArrowRight => {
                self.cursor_x = if self.cursor_x < self.size.columns {
                    self.cursor_x + 1
                } else {
                    self.size.columns
                }
            }
            KeyEvent::ArrowUp => {
                self.cursor_y = if self.cursor_y > 0 {
                    self.cursor_y - 1
                } else {
                    0
                }
            }
            KeyEvent::ArrowDown => {
                self.cursor_y = if self.cursor_y < self.size.rows {
                    self.cursor_y + 1
                } else {
                    self.size.rows
                }
            }
            _ => {
                // do nothing
            }
        }
    }

    fn process_keypress(&mut self) -> Option<()> {
        match read_key() {
            Ok(key_event) => {
                match key_event {
                    KeyEvent::Ctrl('q') => {
                        // clear the screen before exiting
                        execute!(io::stdout(), terminal::Clear(terminal::ClearType::All)).unwrap();
                        None
                    }
                    _ => {
                        self.move_cursor(key_event);
                        Some(())
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading key: {}", e);
                None
            }
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        loop {
            self.refresh_screen()?;
            if !self.process_keypress().is_some() {
                break;
            }
        }
        Ok(())
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        if let Err(e) = terminal::disable_raw_mode() {
            eprintln!("Error: Failed to disable raw mode: {}", e);
        }
    }
}

fn main() -> io::Result<()> {
    let mut editor = Editor::new()?;
    editor.run()?;
    Ok(())
}
