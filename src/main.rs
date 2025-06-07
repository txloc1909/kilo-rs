use crossterm::{cursor, execute, queue, style, terminal};
use std::io::{self, Read, Write};
use std::option::Option;

fn ctrl_key(c: u8) -> Option<u8> {
    let c = c.to_ascii_uppercase();
    if c >= b'A' && c <= b'Z' {
        Some((c as u8) & 0x1F)
    } else {
        None
    }
}

fn read_key() -> io::Result<u8> {
    let mut byte = [0u8; 1];
    io::stdin().read_exact(&mut byte)?;
    Ok(byte[0])
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

    fn move_cursor(&mut self, key: u8) {
        match key {
            b'a' => {
                self.cursor_x = self.cursor_x - 1;
            }
            b'd' => {
                self.cursor_x = self.cursor_x + 1;
            }
            b'w' => {
                self.cursor_y = self.cursor_y - 1;
            }
            b's' => {
                self.cursor_y = self.cursor_y + 1;
            }
            _ => {
                // do nothing
            }
        }
    }

    fn process_keypress(&mut self) -> Option<()> {
        match read_key() {
            Ok(byte) => {
                if byte == ctrl_key(b'q').unwrap_or(0) {
                    // clear the screen before exiting
                    execute!(io::stdout(), terminal::Clear(terminal::ClearType::All)).unwrap();
                    return None;
                }
                if byte.is_ascii_control() {
                    println!("Control character detected: {:x}\r", byte);
                } else {
                    println!("Read byte: {}\r", byte);
                }
                Some(())
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
