use crossterm::{cursor::MoveTo, execute, terminal};
use std::io::{self, Read};
use std::option::Option;

struct RawModeGuard;

impl RawModeGuard {
    fn new() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        Ok(RawModeGuard)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        if let Err(e) = terminal::disable_raw_mode() {
            eprintln!("Error: Failed to disable raw mode: {}", e);
        }
    }
}

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

fn refresh_screen() -> io::Result<()> {
    let mut stdout = io::stdout();
    execute!(
        stdout,
        terminal::Clear(terminal::ClearType::All),
        MoveTo(0, 0)
    )
    .unwrap();
    Ok(())
}

fn process_keypress() -> Option<()> {
    match read_key() {
        Ok(byte) => {
            if byte == ctrl_key(b'q').unwrap_or(0) {
                refresh_screen().unwrap();
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

fn main() -> io::Result<()> {
    let _raw_mode_guard = RawModeGuard::new()?;

    loop {
        refresh_screen()?;
        if !process_keypress().is_some() {
            break;
        }
    }

    Ok(())
}
