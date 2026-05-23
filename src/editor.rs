use std::{
    io::{self},
    path::Path,
    process::Command,
};

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
                self.input_buffer.push('\n');
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
        let (temp, mut completion_start, pending) = parser::split(&self.input_buffer);

        if temp.len() != 0 {}

        let mut candidates = if temp.len() == 0 {
            utils::get_all_commands()
        } else {
            if let Some(completer) = crate::state::find_completion(&temp[0]) {
                let arg3 = if temp.len() == 1 {
                    "".to_string()
                } else {
                    temp.last().unwrap().clone()
                };
                get_completor_results(
                    &completer,
                    &temp[0],
                    &pending,
                    &arg3,
                    &self.input_buffer,
                    &completion_start.to_string(),
                )
            } else {
                let path = match pending.rsplit_once('/') {
                    Some((dir, _)) => {
                        completion_start += dir.len() + 1;
                        Path::new(dir)
                    }
                    None => Path::new("."),
                };
                utils::get_files_and_directories(path)?
            }
        };

        candidates.sort();
        candidates.dedup();

        let prefix: String = self.input_buffer[completion_start..].to_string();
        if !self.input_buffer.is_empty() {
            let matches: Vec<&str> = candidates
                .iter()
                .filter_map(|x| x.strip_prefix(&prefix))
                .collect();

            let backup_tab_flag = self.tab_flag;
            self.tab_flag = false;
            match matches.len() {
                0 => {
                    execute!(stdout, Print("\x07"))?;
                }
                1 => {
                    let completion = matches[0];
                    self.append_to_buffer(completion)?;
                    if !self.input_buffer.ends_with('/') {
                        self.append_to_buffer(" ")?;
                    }
                }
                _ => {
                    if backup_tab_flag {
                        let to_print = matches
                            .iter()
                            .map(|s| format!("{}{}", prefix, s))
                            .collect::<Vec<_>>()
                            .join(" ");
                        execute!(stdout, Print("\r\n"), Print(to_print), Print("\r\n"))?;
                        self.print_prompt()?;
                        execute!(stdout, Print(&self.input_buffer))?;
                    } else {
                        let first = matches[0];
                        let last = matches[matches.len() - 1];
                        let common_len = first
                            .chars()
                            .zip(last.chars())
                            .take_while(|(c1, c2)| c1 == c2)
                            .count();
                        if common_len > 0 {
                            let completion = &first[0..common_len];
                            self.append_to_buffer(completion)?;
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

    fn append_to_buffer(&mut self, s: &str) -> io::Result<()> {
        let mut stdout = io::stdout();
        self.input_buffer.push_str(s);
        execute!(stdout, Print(s))
    }
}

fn get_completor_results(
    script: &str,
    arg1: &str,
    arg2: &str,
    arg3: &str,
    env1: &str,
    env2: &str,
) -> Vec<String> {
    let output = match Command::new(script)
        .arg(arg1)
        .arg(arg2)
        .arg(arg3)
        .env("COMP_LINE", env1)
        .env("COMP_POINT", env2)
        .output()
    {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };
    if !output.status.success() {
        return Vec::new();
    }
    let result = String::from_utf8_lossy(&output.stdout);
    result
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect()
}
