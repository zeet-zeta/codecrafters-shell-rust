use crossterm::{
    event::{self, Event},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::io;

use crate::editor::LineEditor;

mod editor;
mod executor;
mod parser;
mod state;
mod utils;

fn main() -> io::Result<()> {
    if let Some(history_filename) = std::env::var_os("HISTFILE") {
        crate::state::with_global_history(|x| {
            if let Err(_) = x.read_from_file(&history_filename.to_str().unwrap()) {
                eprintln!("failed to load history from file");
            }
        });
    }
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
    if let Some(history_filename) = std::env::var_os("HISTFILE") {
        crate::state::with_global_history(|x| {
            if let Err(_) = x.write_to_file(
                &history_filename.to_str().unwrap(),
                parser::RedirectMode::Overwrite,
            ) {
                eprintln!("failed to write history to file");
            }
        });
    }
    Ok(())
}
