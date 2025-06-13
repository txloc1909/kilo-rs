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
    PageUp,
    PageDown,
    HomeKey,
    EndKey,
    DelKey,
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
                    KeyCode::PageUp => KeyEvent::PageUp,
                    KeyCode::PageDown => KeyEvent::PageDown,
                    KeyCode::Home => KeyEvent::HomeKey,
                    KeyCode::End => KeyEvent::EndKey,
                    KeyCode::Delete => KeyEvent::DelKey,
                    _ => KeyEvent::Null,
                },
            };
            return Ok(key);
        };
        return Err(io::Error::new(io::ErrorKind::Other, "Fail to read key"));
    }
}

pub struct Editor {
    cursor_x: u16,
    cursor_y: u16,
    size: terminal::WindowSize,
    num_rows: u16,
    row: String,
}

impl Editor {
    pub fn new() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        Ok(Self {
            cursor_x: 0,
            cursor_y: 0,
            size: terminal::window_size().expect("Failed to get window size"),
            num_rows: 0,
            row: "".to_string(),
        })
    }

    pub fn open(&mut self) {
        self.num_rows = 1;
        self.row = "Hello, World!".to_string();
    }

    fn refresh_screen(&self) -> io::Result<()> {
        let mut stdout = io::stdout();
        queue!(stdout, cursor::Hide, cursor::MoveTo(0, 0))?;
        self.draw_rows(&stdout)?;
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
                    KeyEvent::ArrowUp
                    | KeyEvent::ArrowDown
                    | KeyEvent::ArrowLeft
                    | KeyEvent::ArrowRight => {
                        self.move_cursor(key_event);
                        Some(())
                    }
                    KeyEvent::PageUp => {
                        for _ in 1..self.size.rows {
                            self.move_cursor(KeyEvent::ArrowUp);
                        }
                        Some(())
                    }
                    KeyEvent::PageDown => {
                        for _ in 1..self.size.rows {
                            self.move_cursor(KeyEvent::ArrowDown);
                        }
                        Some(())
                    }
                    KeyEvent::HomeKey => {
                        self.cursor_x = 0;
                        Some(())
                    }
                    KeyEvent::EndKey => {
                        self.cursor_x = self.size.columns - 1;
                        Some(())
                    }
                    _ => {
                        // do nothing
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

    fn draw_rows(&self, mut stdout: &io::Stdout) -> io::Result<()> {
        let rows = self.size.rows as usize;
        let num_rows = self.num_rows as usize;
        for y in 0..rows {
            if y >= num_rows {
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
            } else {
                let line_len = std::cmp::min(self.row.len(), self.size.columns as usize);
                queue!(
                    stdout,
                    style::Print(&self.row[..line_len]),
                    terminal::Clear(terminal::ClearType::UntilNewLine)
                )?;
            }
            if y < rows - 1 {
                queue!(stdout, style::Print("\r\n"))?;
            }
        }
        Ok(())
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
    editor.open();
    editor.run()?;
    Ok(())
}
