use crossterm::terminal;
use std::io::{self, Read};

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

fn main() -> io::Result<()> {
    let _raw_mode_guard = RawModeGuard::new()?;

    let mut stdin = io::stdin().lock();
    let mut byte = [0u8; 1];

    while stdin.read(&mut byte)? == 1 {
        let curr_byte = byte[0];
        if curr_byte == ctrl_key(b'q').unwrap_or(0) {
            break;
        }
        if curr_byte.is_ascii_control() {
            println!("Control character detected: {:x}\r", curr_byte);
        } else {
            println!("Read byte: {}\r", curr_byte);
        }
    }

    Ok(())
}
