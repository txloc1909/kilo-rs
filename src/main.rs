use std::io::{self, Read};

fn main() -> io::Result<()> {
    let mut stdin = io::stdin().lock();
    let mut byte = [0u8; 1];

    while stdin.read(&mut byte)? == 1 {
        if byte[0] == b'q' {
            break;
        }
    }
    Ok(())
}
