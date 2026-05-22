#[allow(unused_imports)]
use std::io::{self, Write};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
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
        if let Event::Key(KeyEvent { code, .. }) = event::read()? {
            match code {
                KeyCode::Enter => {
                    if input_buffer == "exit" {
                        break;
                    }

                    execute!(stdout, Print("\r\n"))?;
                    if let Some(c) = parser::parse(&input_buffer) {
                        disable_raw_mode()?;
                        executor::execute(c);
                        enable_raw_mode()?;
                    }
                    input_buffer.clear();
                    execute!(stdout, Print("$ "))?;
                }
                KeyCode::Tab => {
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
                        }
                    }
                }
                KeyCode::Backspace | KeyCode::Char('\x7f') | KeyCode::Char('\x08') => {
                    if input_buffer.pop().is_some() {
                        execute!(stdout, Print("\x08 \x08"))?;
                    }
                }
                KeyCode::Char(c) => {
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
