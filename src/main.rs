#[allow(unused_imports)]
use std::io::{self, Write};

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode},
};

mod executor;
mod parser;

fn main() -> io::Result<()> {
    let completions = vec!["echo", "exit"];
    enable_raw_mode()?;
    let mut input_buffer = String::new();
    let mut stdout = io::stdout();
    execute!(stdout, Print("$ "))?;
    loop {
        if let Event::Key(key_event) = event::read()? {
            match (key_event.code, key_event.modifiers) {
                (KeyCode::Enter, KeyModifiers::NONE)
                | (KeyCode::Char('j'), KeyModifiers::CONTROL) => {
                    execute!(stdout, Print("\r\n"))?;
                    if let Some(c) = parser::parse(&input_buffer) {
                        if c.cmd == "exit" {
                            break;
                        }
                        disable_raw_mode()?;
                        executor::execute(c);
                        enable_raw_mode()?;
                    }
                    input_buffer.clear();
                    execute!(stdout, Print("$ "))?;
                }
                (KeyCode::Tab, KeyModifiers::NONE) => {
                    if !input_buffer.is_empty() {
                        let matches: Vec<&&str> = completions
                            .iter()
                            .filter(|cmd| cmd.starts_with(&input_buffer))
                            .collect();
                        if matches.len() == 1 {
                            let completion = &matches[0][input_buffer.len()..];
                            input_buffer.push_str(completion);
                            input_buffer.push_str(" ");
                            execute!(stdout, Print(completion), Print(" "))?;
                        } else {
                            execute!(stdout, Print("\x07"))?;
                        }
                    }
                }
                (KeyCode::Backspace, KeyModifiers::NONE) => {
                    if input_buffer.pop().is_some() {
                        execute!(stdout, Print("\x08 \x08"))?;
                    }
                }
                (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                    input_buffer.push(c);
                    execute!(stdout, Print(c))?;
                }
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    Ok(())
}
