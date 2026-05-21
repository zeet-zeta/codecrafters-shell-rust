#[derive(Debug, PartialEq)]
enum State {
    Normal,
    SingleQuote,
    DoubleQuote,
    NormalWithEscape,
    DoubleQuoteWithEscape,
}

pub fn parse_command_line(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current_arg = String::new();
    let mut state = State::Normal;

    for ch in input.chars() {
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
    args
}
