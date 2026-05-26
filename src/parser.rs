use std::collections::HashMap;

use regex::{Captures, Regex};

use crate::state::with_local_vars;
#[derive(Debug, PartialEq)]
enum State {
    Normal,
    SingleQuote,
    DoubleQuote,
    NormalWithEscape,
    DoubleQuoteWithEscape,
}

#[derive(Clone, Copy)]
pub enum RedirectMode {
    Overwrite,
    Append,
}

#[derive(Default)]
pub struct CommandArgs {
    pub cmd: String,
    pub args: Vec<String>,
    pub stdout: Option<(String, RedirectMode)>,
    pub stderr: Option<(String, RedirectMode)>,
    pub backup_the_whole_cmd: Option<String>, //同时用于标识是不是后台命令
}

pub fn split(input: &str) -> (Vec<Vec<String>>, usize, String) {
    let mut result = Vec::new();
    let mut args = Vec::new();
    let mut current_arg = String::new();
    let mut state = State::Normal;
    let mut completion_start: usize = usize::MAX;

    for (i, ch) in input.char_indices() {
        match state {
            State::Normal => match ch {
                '\'' => {
                    state = State::SingleQuote;
                }
                '"' => {
                    state = State::DoubleQuote;
                }
                ' ' | '\t' | '\n' => {
                    if !current_arg.is_empty() {
                        args.push(std::mem::take(&mut current_arg));
                    }
                    completion_start = i;
                }
                '\\' => {
                    state = State::NormalWithEscape;
                }
                '|' => {
                    if !current_arg.is_empty() {
                        args.push(std::mem::take(&mut current_arg));
                    }
                    completion_start = i;

                    result.push(std::mem::take(&mut args));
                }
                _ => {
                    current_arg.push(ch);
                }
            },
            State::SingleQuote => match ch {
                '\'' => state = State::Normal,
                _ => current_arg.push(ch),
            },
            State::DoubleQuote => match ch {
                '"' => state = State::Normal,
                '\\' => state = State::DoubleQuoteWithEscape,
                _ => current_arg.push(ch),
            },
            State::NormalWithEscape => {
                current_arg.push(ch);
                state = State::Normal;
            }
            State::DoubleQuoteWithEscape => {
                current_arg.push(ch);
                state = State::DoubleQuote;
            }
        }
    }
    result.push(args);
    (result, completion_start.wrapping_add(1), current_arg)
}

fn parse_redirect(mut tokens: Vec<String>) -> Option<CommandArgs> {
    if tokens.is_empty() {
        return None;
    }
    let mut result = CommandArgs::default();

    if tokens.last().map_or(false, |x| x == "&") {
        result.backup_the_whole_cmd = Some(tokens.join(" "));
        tokens.pop();
    }

    if let Some(idx) = tokens.iter().rposition(|x| x == "2>" || x == "2>>") {
        if idx + 1 < tokens.len() {
            let mode = if tokens[idx] == "2>>" {
                RedirectMode::Append
            } else {
                RedirectMode::Overwrite
            };
            let file = tokens.remove(idx + 1);
            tokens.remove(idx);
            result.stderr = Some((file, mode));
        } else {
            return None;
        }
    }

    if let Some(idx) = tokens
        .iter()
        .rposition(|x| x == ">" || x == ">>" || x == "1>" || x == "1>>")
    {
        if idx + 1 < tokens.len() {
            let mode = if tokens[idx] == ">>" || tokens[idx] == "1>>" {
                RedirectMode::Append
            } else {
                RedirectMode::Overwrite
            };
            let file = tokens.remove(idx + 1);
            tokens.remove(idx);
            result.stdout = Some((file, mode));
        } else {
            return None;
        }
    }

    if tokens.is_empty() {
        return None;
    }

    result.cmd = tokens.remove(0);
    result.args = tokens;
    Some(result)
}

fn parse_vars(input: &str, context: &HashMap<String, String>) -> String {
    let re = Regex::new(r"\$(?:([a-zA-Z_][a-zA-Z0-9_]*)|\{([a-zA-Z_][a-zA-Z0-9_]*)\})").unwrap();
    let result = re.replace_all(input, |caps: &Captures| {
        let var_name = caps
            .get(1)
            .or_else(|| caps.get(2))
            .map(|m| m.as_str())
            .unwrap_or("");
        match context.get(var_name) {
            Some(value) => value.clone(),
            None => caps[0].to_string(),
        }
    });
    result.into_owned()
}

pub fn parse(input: &str) -> Option<Vec<CommandArgs>> {
    let expand_vars = with_local_vars(|x| parse_vars(input, &x.table));
    let (commands, _, _) = split(&expand_vars);
    commands
        .into_iter()
        .map(|x| parse_redirect(x))
        .into_iter()
        .collect()
}
