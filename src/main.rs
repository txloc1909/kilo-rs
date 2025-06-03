use crossterm::terminal;
use std::io::{self, Read};

fn main() -> io::Result<()> {
    terminal::enable_raw_mode()?;

    let mut stdin = io::stdin().lock();
    let mut byte = [0u8; 1];

    while stdin.read(&mut byte)? == 1 {
        if byte[0] == b'q' {
            break;
        }
    }

    terminal::disable_raw_mode()?;
    Ok(())
}
