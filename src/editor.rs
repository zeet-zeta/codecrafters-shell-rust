use std::io::{self, Write};

use crossterm::{
    event::{KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode},
};

use crate::executor;
use crate::parser;
use crate::utils;

pub struct LineEditor {
    input_buffer: String,
    tab_flag: bool,
    should_exit: bool,
}

impl LineEditor {
    pub fn new() -> Self {
        Self {
            input_buffer: String::new(),
            tab_flag: false,
            should_exit: false,
        }
    }

    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    pub fn print_prompt(&mut self) -> io::Result<()> {
        let mut stdout = io::stdout();
        execute!(stdout, Print("$ "))
    }

    pub fn handle_key(&mut self, key_event: KeyEvent) -> io::Result<()> {
        let mut stdout = io::stdout();
        match (key_event.code, key_event.modifiers) {
            (KeyCode::Enter, KeyModifiers::NONE) | (KeyCode::Char('j'), KeyModifiers::CONTROL) => {
                execute!(stdout, Print("\r\n"))?;
                if let Some(c) = parser::parse(&self.input_buffer) {
                    if c.cmd == "exit" {
                        self.should_exit = true;
                        return Ok(());
                    }
                    disable_raw_mode()?;
                    executor::execute(c);
                    enable_raw_mode()?;
                }
                self.input_buffer.clear();
                self.print_prompt()?;
            }
            (KeyCode::Tab, KeyModifiers::NONE) => {
                self.handle_tab()?;
            }
            (KeyCode::Backspace, KeyModifiers::NONE) => {
                if self.input_buffer.pop().is_some() {
                    execute!(stdout, Print("\x08 \x08"))?;
                }
            }
            (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                self.input_buffer.push(c);
                execute!(stdout, Print(c))?;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_tab(&mut self) -> io::Result<()> {
        let mut stdout = io::stdout();
        let mut candidates = utils::get_all_external_commands().unwrap_or_default();
        candidates.push("exit".to_string());
        candidates.push("echo".to_string());

        candidates.sort();
        candidates.dedup();

        if !self.input_buffer.is_empty() {
            let matches: Vec<&String> = candidates
                .iter()
                .filter(|cmd| cmd.starts_with(&self.input_buffer))
                .collect();

            match matches.len() {
                0 => {
                    execute!(stdout, Print("\x07"))?;
                    self.tab_flag = false;
                }
                1 => {
                    let completion = &matches[0][self.input_buffer.len()..];
                    self.input_buffer.push_str(completion);
                    self.input_buffer.push_str(" ");
                    execute!(stdout, Print(completion), Print(" "))?;
                    self.tab_flag = false;
                }
                _ => {
                    if self.tab_flag {
                        execute!(stdout, Print("\r\n"))?;
                        matches.iter().for_each(|s| print!("{} ", s));
                        execute!(stdout, Print("\r\n"))?;
                        self.print_prompt()?;
                        print!("{}", self.input_buffer);
                        stdout.flush()?;
                        self.tab_flag = false;
                    } else {
                        let first = matches[0];
                        let last = matches[matches.len() - 1];
                        let common_len = first
                            .chars()
                            .zip(last.chars())
                            .take_while(|(c1, c2)| c1 == c2)
                            .count();
                        let orig_len = self.input_buffer.len();
                        if common_len > orig_len {
                            let completion = &first[orig_len..common_len];
                            execute!(stdout, Print(completion))?;
                            self.input_buffer = first[..common_len].to_string();
                            self.tab_flag = false;
                        } else {
                            execute!(stdout, Print("\x07"))?;
                            self.tab_flag = true;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
