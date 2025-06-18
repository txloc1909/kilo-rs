use std::io;
use std::path::Path;

mod editor;
use editor::Editor;

fn main() -> io::Result<()> {
    let mut editor = Editor::new()?;
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 2 {
        let _ = editor.open(Path::new(&args[1]));
    }
    editor.run()?;
    Ok(())
}
