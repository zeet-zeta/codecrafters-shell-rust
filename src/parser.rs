#[derive(Debug, PartialEq)]
enum State {
    Normal,
    SingleQuote,
    DoubleQuote,
    NormalWithEscape,
    DoubleQuoteWithEscape,
}

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
}

pub fn split(input: &str) -> (Vec<String>, usize) {
    let mut args = Vec::new();
    let mut current_arg = String::new();
    let mut state = State::Normal;
    let mut completion_start: usize = -1;

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
                        args.push(current_arg.clone());
                        current_arg.clear();
                    }
                    completion_start = i;
                }
                '\\' => {
                    state = State::NormalWithEscape;
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
    (args, completion_start.wrapping_add(1))
}

fn parse_redirect(mut tokens: Vec<String>) -> Option<CommandArgs> {
    if tokens.is_empty() {
        return None;
    }
    let mut result = CommandArgs::default();
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

pub fn parse(input: &str) -> Option<CommandArgs> {
    let (tokens, _) = split(input);
    let command = parse_redirect(tokens);
    command
}
