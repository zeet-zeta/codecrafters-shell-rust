#[derive(Debug, PartialEq)]
enum State {
    Normal,
    SingleQuote,
    DoubleQuote,
}
pub fn parse_command_line(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current_arg = String::new();
    let mut state = State::Normal;
    let mut escape_flag = false;

    for ch in input.chars() {
        if ch == '\\' {
            escape_flag = true;
            continue;
        }
        if escape_flag {
            current_arg.push(ch);
            escape_flag = false;
        }
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
                _ => current_arg.push(ch),
            },
        }
    }
    args
}
