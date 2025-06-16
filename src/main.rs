use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::{cursor, execute, queue, style, terminal};
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::option::Option;
use std::path::Path;

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

fn read_lines(path: &Path) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);

    reader.lines().collect()
}

pub struct Editor {
    cursor_x: u16,
    cursor_y: u16,
    row_offset: u16,
    col_offset: u16,
    size: terminal::WindowSize,
    rows: Vec<String>,
}

impl Editor {
    pub fn new() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        Ok(Self {
            cursor_x: 0,
            cursor_y: 0,
            row_offset: 0,
            col_offset: 0,
            size: terminal::window_size().expect("Failed to get window size"),
            rows: vec![String::new()],
        })
    }

    pub fn open(&mut self, path: &std::path::Path) -> Result<(), io::Error> {
        if let Ok(lines) = read_lines(path) {
            self.rows = lines;
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Error: Could not read file at {:?}", path),
            ))
        }
    }

    fn refresh_screen(&mut self) -> io::Result<()> {
        self.scroll();
        let mut stdout = io::stdout();
        queue!(stdout, cursor::Hide, cursor::MoveTo(0, 0))?;
        self.draw_rows(&stdout)?;
        queue!(
            stdout,
            cursor::MoveTo(
                (self.cursor_x - self.col_offset) as u16,
                (self.cursor_y - self.row_offset) as u16
            ),
            cursor::Show
        )?;
        stdout.flush()?;
        Ok(())
    }

    fn scroll(&mut self) {
        if self.cursor_y < self.row_offset {
            self.row_offset = self.cursor_y;
        } else if self.cursor_y >= self.row_offset + self.size.rows {
            self.row_offset = self.cursor_y - self.size.rows + 1;
        }

        if self.cursor_x < self.col_offset {
            self.col_offset = self.cursor_x;
        } else if self.cursor_x >= self.col_offset + self.size.columns {
            self.col_offset = self.cursor_x - self.size.columns + 1;
        }
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
                let row: Option<&String> = self.rows.get(self.cursor_y as usize);
                if let Some(row) = row {
                    self.cursor_x = if self.cursor_x < row.len() as u16 {
                        self.cursor_x + 1
                    } else {
                        row.len() as u16
                    };
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
                let num_rows = self.rows.len() as u16;
                self.cursor_y = if self.cursor_y < num_rows {
                    self.cursor_y + 1
                } else {
                    self.size.rows
                }
            }
            _ => {
                // do nothing
            }
        }

        let row: Option<&String> = self.rows.get(self.cursor_y as usize);
        if let Some(row) = row {
            self.cursor_x = self.cursor_x.min(row.len() as u16);
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
        let row_offset = self.row_offset as usize;
        let num_rows = self.rows.len();

        for (y, row) in (0..rows).zip(row_offset..row_offset + rows) {
            if row >= num_rows {
                if num_rows == 0 && y == rows / 3 {
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
                let curr_row = &self.rows[row];
                let visible_line = &curr_row[self.col_offset as usize..];
                let line_len = curr_row.len() as i32 - self.col_offset as i32;
                let line_len = line_len.clamp(0, self.size.columns as i32) as usize;
                queue!(
                    stdout,
                    style::Print(visible_line[..line_len].to_string()),
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
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 2 {
        let _ = editor.open(Path::new(&args[1]));
    }
    editor.run()?;
    Ok(())
}
