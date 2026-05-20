#[allow(unused_imports)]
use std::io::{self, Write};
use std::{os::unix::fs::MetadataExt, panic, process::Command};

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
    let cmd = args[0];
    match find_executable(cmd) {
      Some(_) => {
        let _ = Command::new(cmd).args(&args[1..]).status();
      }
      None => println!("{}: command not found", cmd),
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
            None => println!("{}: not found", s),
          }
        }
      });
    }
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
