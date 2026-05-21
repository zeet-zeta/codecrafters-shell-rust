#[allow(unused_imports)]
use std::io::{self, Write};

mod executor;
mod parser;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        if let Some(c) = parser::parse(&input) {
            executor::execute(c);
        }
    }
}
