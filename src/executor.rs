use std::{
    io::{ErrorKind, Write},
    os::unix::io::AsRawFd,
    path::PathBuf,
    process::{Child, Command, Stdio},
};

use crate::utils::{find_executable, open_file};
use crate::{parser::CommandArgs, state::JobDetail};

#[derive(PartialEq)]
enum Builtin {
    Exit,
    Echo,
    Type,
    Pwd,
    Cd,
    Complete,
    Jobs,
    History,
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
            "history" => Some(Builtin::History),
            _ => None,
        }
    }
}

pub fn execute_single(c: CommandArgs) {
    if let Some(b) = Builtin::new(&c.cmd) {
        execute_builtin(b, c);
    } else {
        execute_external(c);
    }
}

pub fn execute_pipeline(commands: Vec<CommandArgs>) {
    // 相信管道是最普通的，没有&，没有重定向，没有内部命令
    if commands.is_empty() {
        return;
    }

    let mut last_stdout: Option<Stdio> = None;
    let mut child_processes: Vec<Child> = Vec::new();
    let num_commands = commands.len();

    for (i, cmd_args) in commands.into_iter().enumerate() {
        assert!(cmd_args.backup_the_whole_cmd.is_none());
        assert!(cmd_args.stderr.is_none());
        assert!(cmd_args.stdout.is_none());

        match Builtin::new(&cmd_args.cmd) {
            Some(builtin_type) => {
                if builtin_type == Builtin::Echo && i == 0 {
                    let (reader, mut writer) = os_pipe::pipe().unwrap();
                    let child_stdin: Stdio = reader.into();
                    let mut to_print = cmd_args.args.join(" ");
                    to_print.push_str("\n");
                    let _ = writer.write_all(to_print.as_bytes());
                    drop(writer);
                    last_stdout = Some(child_stdin);
                } else if i == num_commands - 1 {
                    // 测试样例里面有 ls | type exit
                    // 什么都不做 管道的读端我们不需要了
                    // 此处其实最好想办法排空管道然后关闭这个fd
                    execute_builtin(builtin_type, cmd_args);
                } else {
                    panic!();
                }
            }
            None => {
                let mut cmd = Command::new(&cmd_args.cmd);
                cmd.args(&cmd_args.args);
                if let Some(stdout) = last_stdout.take() {
                    cmd.stdin(stdout);
                }
                if i < num_commands - 1 {
                    cmd.stdout(Stdio::piped());
                }
                let mut child = cmd.spawn().unwrap();
                if i < num_commands - 1 {
                    if let Some(stdout) = child.stdout.take() {
                        last_stdout = Some(Stdio::from(stdout));
                    }
                }
                child_processes.push(child);
            }
        }
    }

    for mut child in child_processes {
        let _ = child.wait();
    }
}

fn execute_external(c: CommandArgs) {
    let mut child = Command::new(&c.cmd);
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
        match child.spawn() {
            Ok(handler) => {
                let job_id = crate::state::alloc_id();
                println!("[{}] {}", job_id, handler.id());
                let datail = JobDetail::new(job_id, handler, command_string);
                crate::state::with_global_jobs(|x| {
                    let detail = datail;
                    x.add(detail);
                });
            }
            Err(err) => {
                if err.kind() == ErrorKind::NotFound {
                    eprintln!("{}: command not found", c.cmd);
                } else {
                    panic!();
                }
            }
        }
    } else {
        if let Err(err) = child.status() {
            if err.kind() == ErrorKind::NotFound {
                eprintln!("{}: command not found", c.cmd);
            } else {
                panic!();
            }
        }
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
        Builtin::History => {
            if c.args.len() == 2 {
                crate::state::with_global_history(|x| {
                    let result = match c.args[0].as_str() {
                        "-r" => x.read_from_file(&c.args[1]),
                        "-w" => x.write_to_file(&c.args[1], crate::parser::RedirectMode::Overwrite),
                        "-a" => x.write_to_file(&c.args[1], crate::parser::RedirectMode::Append),
                        _ => return,
                    };
                    if let Err(e) = result {
                        eprintln!("history: failed to write {}: {}", c.args[1], e);
                    }
                });
            } else {
                let n = c.args.get(0).and_then(|s| s.parse().ok());
                crate::state::with_global_history(|x| x.print(n));
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
