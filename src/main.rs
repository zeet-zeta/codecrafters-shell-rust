#[allow(unused_imports)]
use std::io::{self, Write};
use std::panic;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let args: Vec<&str> = input.split_whitespace().collect();
        if !args.is_empty() {
            execute(&args);
        }
    }
}

fn is_buildin(cmd: &str) -> bool {
    match cmd {
        "exit" | "echo" | "type" => true,
        _ => false,
    }
}

fn execute(args: &[&str]) {
    if is_buildin(args[0]) {
        execute_builtin(args);
    } else {
        println!("{}: command not found", args[0]);
    }
}

fn execute_builtin(args: &[&str]) {
    match args[0] {
        "exit" => std::process::exit(0),
        "echo" => println!("{}", args[1..].join(" ")),
        "type" => {
            args[1..].iter().for_each(|s| {
                if is_buildin(s) {
                    println!("{} is a shell builtin", s);
                } else {
                    println!("{}: not found", s);
                }
            });
        }
        _ => panic!(),
    }
}
