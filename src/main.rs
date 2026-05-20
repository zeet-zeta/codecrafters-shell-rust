#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    // TODO: Uncomment the code below to pass the first stage
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let args: Vec<&str> = input.split_whitespace().collect();
        if !args.is_empty() {
            let cmd = args[0];
            let cmd_args = &args[1..];
            match cmd {
                "exit" => {
                    break;
                }
                "echo" => {
                    let output = cmd_args.join(" ");
                    println!("{}", output);
                }
                _ => {
                    println!("{}: command not found", cmd);
                }
            }
        }
    }
}
