use std::{
    fs::{File, OpenOptions},
    io::Write,
    os::fd::AsRawFd,
    path::PathBuf,
    process::Command,
};

use crate::utils::find_executable;
use crate::{
    parser::{CommandArgs, RedirectMode},
    state::JobDetail,
};

enum Builtin {
    Exit,
    Echo,
    Type,
    Pwd,
    Cd,
    Complete,
    Jobs,
}

impl Builtin {
    fn new(s: &str) -> Option<Self> {
        match s {
            "exit" => Some(Builtin::Exit),
            "echo" => Some(Builtin::Echo),
            "type" => Some(Builtin::Type),
            "pwd" => Some(Builtin::Pwd),
            "cd" => Some(Builtin::Cd),
            "complete" => Some(Builtin::Complete),
            "jobs" => Some(Builtin::Jobs),
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
            if let Some(command_string) = c.backup_the_whole_cmd {
                let job_id = crate::state::alloc_id();
                let handler = child.spawn().unwrap();
                println!("[{}] {}", job_id, handler.id());
                let datail = JobDetail::new(job_id, handler, command_string);
                crate::state::with_global_jobs(|x| {
                    let detail = datail;
                    x.add(detail);
                });
            } else {
                let _ = child.status();
            }
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
        Builtin::Exit => {}
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
        Builtin::Complete => {
            if c.args.len() == 2 && c.args[0] == "-p" {
                match crate::state::find_completion(&c.args[1]) {
                    Some(s) => println!("complete -C '{}' {}", s, c.args[1]),
                    None => eprintln!("complete: {}: no completion specification", c.args[1]),
                }
            } else if c.args.len() == 3 && c.args[0] == "-C" {
                crate::state::register_completion(c.args[2].clone(), c.args[1].clone());
            } else if c.args.len() == 2 && c.args[0] == "-r" {
                crate::state::unregister_completion(&c.args[1]);
            }
        }
        Builtin::Jobs => {
            crate::state::with_global_jobs(|x| {
                x.reap_and_print();
            });
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
