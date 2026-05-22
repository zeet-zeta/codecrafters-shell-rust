#[allow(unused_imports)]
use std::io::{self, Write};

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode},
};

use crate::executor::get_all_external_commands;

mod executor;
mod parser;

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut input_buffer = String::new();
    let mut stdout = io::stdout();
    execute!(stdout, Print("$ "))?;

    let mut tab_flag = false;
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
                    let mut candidates = get_all_external_commands().unwrap_or_default();
                    candidates.push("exit".to_string());
                    candidates.push("echo".to_string());

                    candidates.sort();
                    candidates.dedup();

                    if !input_buffer.is_empty() {
                        let matches: Vec<&String> = candidates
                            .iter()
                            .filter(|cmd| cmd.starts_with(&input_buffer))
                            .collect();

                        match matches.len() {
                            0 => {
                                execute!(stdout, Print("\x07"))?;
                                tab_flag = false;
                            }
                            1 => {
                                let completion = &matches[0][input_buffer.len()..];
                                input_buffer.push_str(completion);
                                input_buffer.push_str(" ");
                                execute!(stdout, Print(completion), Print(" "))?;
                                tab_flag = false;
                            }
                            _ => {
                                if tab_flag {
                                    execute!(stdout, Print("\r\n"))?;
                                    matches.iter().for_each(|s| print!("{} ", s));
                                    execute!(stdout, Print("\r\n"))?;
                                    execute!(stdout, Print("$ "))?;
                                    print!("{}", input_buffer);
                                    stdout.flush()?;
                                    tab_flag = false;
                                } else {
                                    execute!(stdout, Print("\x07"))?;
                                    tab_flag = true;
                                }
                            }
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
