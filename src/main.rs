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

fn main() -> io::Result<()> {
    let _raw_mode_guard = RawModeGuard::new()?;

    let mut stdin = io::stdin().lock();
    let mut byte = [0u8; 1];

    while stdin.read(&mut byte)? == 1 {
        let curr_byte = byte[0];
        if curr_byte == b'q' {
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
