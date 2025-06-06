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
    for i in 0..rows {
        queue!(stdout, style::Print("~"))?;
        if i < rows - 1 {
            queue!(stdout, style::Print("\r\n"))?;
        }
    }
    Ok(())
}

fn process_keypress() -> Option<()> {
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

pub struct Editor {
    size: terminal::WindowSize,
}

impl Editor {
    pub fn new() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        Ok(Self {
            size: terminal::window_size().expect("Failed to get window size"),
        })
    }

    fn refresh_screen(&self) -> io::Result<()> {
        let mut stdout = io::stdout();
        queue!(
            stdout,
            cursor::Hide,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )?;
        draw_rows(self.size.rows, &stdout)?;
        queue!(stdout, cursor::MoveTo(0, 0), cursor::Show)?;
        stdout.flush()?;
        Ok(())
    }

    pub fn run(&mut self) -> io::Result<()> {
        loop {
            self.refresh_screen()?;
            if !process_keypress().is_some() {
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
