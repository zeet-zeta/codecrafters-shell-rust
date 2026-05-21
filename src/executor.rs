use std::{
    fs::{File, OpenOptions},
    io::Write,
    os::{fd::AsRawFd, unix::fs::MetadataExt},
    path::PathBuf,
    process::Command,
};

use crate::parser::{CommandArgs, RedirectMode};

enum Builtin {
    Exit,
    Echo,
    Type,
    Pwd,
    Cd,
}

impl Builtin {
    fn new(s: &str) -> Option<Self> {
        match s {
            "exit" => Some(Builtin::Exit),
            "echo" => Some(Builtin::Echo),
            "type" => Some(Builtin::Type),
            "pwd" => Some(Builtin::Pwd),
            "cd" => Some(Builtin::Cd),
            _ => None,
        }
    }
}

pub fn execute(c: CommandArgs) {
    if let Some(b) = Builtin::new(&c.cmd) {
        execute_builtin(b, c);
    } else {
        execute_external(c);
    }
}

fn execute_external(c: CommandArgs) {
    match find_executable(&c.cmd) {
        Some(_) => {
            let mut child = Command::new(c.cmd);
            child.args(c.args);
            if let Some((path, mode)) = c.stdout {
                if let Ok(file) = open_file(&path, mode) {
                    child.stdout(file);
                }
            }
            if let Some((path, mode)) = c.stderr {
                if let Ok(file) = open_file(&path, mode) {
                    child.stderr(file);
                }
            }
            let _ = child.status();
        }
        None => eprintln!("{}: command not found", c.cmd),
    }
}

fn execute_builtin(builtin_type: Builtin, c: CommandArgs) {
    let mut backup_stdout = None;
    let mut backup_stderr = None;
    if let Some((path, mode)) = c.stdout {
        if let Ok(file) = open_file(&path, mode) {
            let file_fd = file.as_raw_fd();
            unsafe {
                let old = libc::dup(libc::STDOUT_FILENO);
                backup_stdout = Some(old);
                libc::dup2(file_fd, libc::STDOUT_FILENO);
            }
        }
    }
    if let Some((path, mode)) = c.stderr {
        if let Ok(file) = open_file(&path, mode) {
            let file_fd = file.as_raw_fd();
            unsafe {
                let old = libc::dup(libc::STDERR_FILENO);
                backup_stderr = Some(old);
                libc::dup2(file_fd, libc::STDERR_FILENO);
            }
        }
    }

    match builtin_type {
        Builtin::Exit => std::process::exit(0),
        Builtin::Echo => println!("{}", c.args.join(" ")),
        Builtin::Type => {
            c.args.iter().for_each(|s| {
                if Builtin::new(s).is_some() {
                    println!("{} is a shell builtin", s);
                } else {
                    match find_executable(s) {
                        Some(p) => println!("{} is {}", s, p.display()),
                        None => eprintln!("{}: not found", s),
                    }
                }
            });
        }
        Builtin::Pwd => {
            let pwd = std::env::current_dir().unwrap();
            println!("{}", pwd.display());
        }
        Builtin::Cd => {
            if c.args.len() == 0 {
                return;
            }
            let target_path = match c.args[0].as_str() {
                "~" => match std::env::var_os("HOME") {
                    Some(os_str) => PathBuf::from(os_str),
                    None => PathBuf::from("/"),
                },
                path_str => PathBuf::from(path_str),
            };
            if std::env::set_current_dir(&target_path).is_err() {
                eprintln!("cd: {}: No such file or directory", target_path.display());
            }
        }
    }

    let _ = std::io::stdout().flush();
    let _ = std::io::stderr().flush();

    unsafe {
        if let Some(old_out) = backup_stdout {
            libc::dup2(old_out, libc::STDOUT_FILENO);
            libc::close(old_out);
        }

        if let Some(old_err) = backup_stderr {
            libc::dup2(old_err, libc::STDERR_FILENO);
            libc::close(old_err);
        }
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

fn open_file(path: &str, mode: RedirectMode) -> std::io::Result<File> {
    match mode {
        RedirectMode::Overwrite => OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path),
        RedirectMode::Append => OpenOptions::new().append(true).create(true).open(path),
    }
}
