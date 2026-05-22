use std::io;

use crossterm::{
    event::{self, Event},
    terminal::{disable_raw_mode, enable_raw_mode},
};

use crate::editor::LineEditor;

mod editor;
mod executor;
mod parser;
mod utils;

fn main() -> io::Result<()> {
    enable_raw_mode()?;

    let mut editor = LineEditor::new();
    editor.print_prompt()?;

    loop {
        if let Event::Key(key_event) = event::read()? {
            editor.handle_key(key_event)?;
            if editor.should_exit() {
                break;
            }
        }
    }

    disable_raw_mode()?;
    Ok(())
}
