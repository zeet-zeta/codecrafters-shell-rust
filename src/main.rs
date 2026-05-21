#[allow(unused_imports)]
use std::io::{self, Write};
use std::{env, os::unix::fs::MetadataExt, panic, path::PathBuf, process::Command};

mod parser;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let parsed_strings = parser::parse_command_line(&input);
        let args: Vec<&str> = parsed_strings.iter().map(|s| s.as_str()).collect();
        if !args.is_empty() {
            execute(&args);
        }
    }
}

fn is_buildin(cmd: &str) -> bool {
    match cmd {
        "exit" | "echo" | "type" | "pwd" | "cd" => true,
        _ => false,
    }
}

fn execute(args: &[&str]) {
    if is_buildin(args[0]) {
        execute_builtin(args);
    } else {
        let cmd = args[0];
        match find_executable(cmd) {
            Some(_) => {
                let _ = Command::new(cmd).args(&args[1..]).status();
            }
            None => eprintln!("{}: command not found", cmd),
        }
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
                    match find_executable(s) {
                        Some(p) => println!("{} is {}", s, p.display()),
                        None => eprintln!("{}: not found", s),
                    }
                }
            });
        }
        "pwd" => {
            let pwd = std::env::current_dir().unwrap();
            println!("{}", pwd.display());
        }
        "cd" => builtin_cd(&args[1..]),
        _ => panic!(),
    }
}

fn find_executable(cmd: &str) -> Option<std::path::PathBuf> {
    let path_os = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_os) {
        let full_path = dir.join(cmd);
        if is_executable(&full_path) {
            return Some(full_path);
        }
    }
    None
}

fn is_executable(path: &std::path::Path) -> bool {
    if let Ok(metadata) = path.metadata() {
        metadata.is_file() && (metadata.mode() & 0o111) != 0
    } else {
        false
    }
}

fn builtin_cd(args: &[&str]) {
    // 此时的args是不含cd命令本身的
    if args.len() >= 2 {
        eprintln!("Too many args for cd command");
        return;
    }
    if args.len() == 0 {
        eprintln!("not supported");
    }
    let target_path = match args[0] {
        "~" => match env::var_os("HOME") {
            Some(os_str) => PathBuf::from(os_str),
            None => PathBuf::from("/"),
        },
        path_str => PathBuf::from(path_str),
    };
    match std::env::set_current_dir(&target_path) {
        Ok(_) => {}
        Err(_) => {
            eprintln!("cd: {}: No such file or directory", target_path.display());
        }
    }
}
