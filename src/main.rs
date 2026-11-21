use std::{
    env,
    io::ErrorKind,
    os::unix::{fs::PermissionsExt, process::CommandExt},
    path::PathBuf,
    process::Command,
};

use rustyline::{
    Config, Editor,
    config::BellStyle,
    error::ReadlineError,
};

mod completion;
mod parser;
mod redirection;

use completion::ShellHelper;
use parser::parse_command;
use redirection::{Redirections, split_redirections};

fn is_builtin(cmd: &str) -> bool {
    matches!(cmd, "echo" | "exit" | "type" | "pwd" | "cd")
}

fn execute_non_builtin_commands(tokens: &[String], redirections: &Redirections) {
    let Some(command) = tokens.first() else {
        return;
    };

    let Some(path) = locate_executable_files(command) else {
        let _ = redirections.write_stderr(&format!("{}: command not found\n", command));
        return;
    };

    let mut process = Command::new(path);
    process.arg0(command).args(&tokens[1..]);

    match redirections.open_stdout() {
        Ok(Some(output_file)) => {
            process.stdout(output_file);
        }
        Ok(None) => {}
        Err(error) => {
            eprintln!("stdout redirect: {}", error);
            return;
        }
    }

    match redirections.open_stderr() {
        Ok(Some(error_file)) => {
            process.stderr(error_file);
        }
        Ok(None) => {}
        Err(error) => {
            eprintln!("stderr redirect: {}", error);
            return;
        }
    }

    let result = process.status();

    match result {
        Ok(_) => {}
        Err(error) => {
            let _ =
                redirections.write_stderr(&format!("failed to execute {}: {}\n", command, error));
        }
    }
}

fn locate_executable_files(command: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;

    for dir in env::split_paths(&path) {
        let candidate = dir.join(command);

        let Ok(metadata) = candidate.metadata() else {
            continue;
        };

        // checking whether it is a executable file
        // 0o100 => Owner,  0o010 => Group, 0o001 => Others
        // checking for just one execute bit
        if metadata.is_file() && metadata.permissions().mode() & 0o111 != 0 {
            return Some(candidate);
        }
    }

    None
}

fn change_directory(args: &[String], redirections: &Redirections) {
    let directory = if args.is_empty() || args[0] == "~" {
        let Some(home) = env::var_os("HOME") else {
            let _ = redirections.write_stderr("cd: HOME is not set\n");
            return;
        };

        PathBuf::from(home)
    } else {
        PathBuf::from(&args[0])
    };

    if let Err(error) = env::set_current_dir(&directory) {
        let message = match error.kind() {
            ErrorKind::NotFound => {
                format!("cd: {}: No such file or directory\n", directory.display())
            }
            ErrorKind::PermissionDenied => {
                format!("cd: {}: Permission denied\n", directory.display())
            }
            ErrorKind::NotADirectory => {
                format!("cd: {}: Not a directory\n", directory.display())
            }
            _ => format!("cd: {}: {}\n", directory.display(), error),
        };
        let _ = redirections.write_stderr(&message);
    }
}

fn func_type(command: &str, redirections: &Redirections) {
    if is_builtin(command) {
        let _ = redirections.write_stdout(&format!("{} is shell builtin\n", command));
        return;
    }

    match locate_executable_files(command) {
        Some(path) => {
            let _ = redirections.write_stdout(&format!("{} is {}\n", command, path.display()));
        }
        None => {
            let _ = redirections.write_stderr(&format!("{}: not found\n", command));
        }
    }
}

fn main() {
    let config = Config::builder().bell_style(BellStyle::Audible).build();
    let helper = ShellHelper::new();
    let mut editor = Editor::with_config(config).expect("Failed to create editor");
    editor.set_helper(Some(helper));

    loop {
        let command = match editor.readline("$ ") {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("exit");
                break;
            }
            Err(err) => {
                eprintln!("Error reading line: {}", err);
                break;
            }
        };

        let command = command.trim();
        if command.is_empty() {
            continue;
        }

        if let Err(error) = editor.add_history_entry(command) {
            eprintln!("Error adding history entry: {}", error);
        }

        let tokens = match parse_command(command) {
            Ok(tokens) => tokens,
            Err(error) => {
                eprintln!("parse error: {}", error);
                continue;
            }
        };

        let (tokens, redirections) = match split_redirections(&tokens) {
            Ok(result) => result,
            Err(error) => {
                eprintln!("{}", error);
                continue;
            }
        };

        let Some(cmd) = tokens.first().map(String::as_str) else {
            continue;
        };

        match cmd {
            "exit" => break,
            "echo" => {
                let output = tokens[1..].join(" ");

                if let Err(error) = redirections.write_stdout(&format!("{output}\n")) {
                    eprintln!("stdout redirect: {}", error);
                }
            }
            "type" => {
                for arg in &tokens[1..] {
                    func_type(arg, &redirections);
                }
            }
            "pwd" => {
                let _ = redirections
                    .write_stdout(&format!("{}\n", env::current_dir().unwrap().display()));
            }
            "cd" => change_directory(&tokens[1..], &redirections),
            _ => execute_non_builtin_commands(&tokens, &redirections),
        }
    }
}
