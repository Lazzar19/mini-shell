
use std::{
    env,
    io::ErrorKind,
    os::unix::{fs::PermissionsExt, process::CommandExt},
    path::PathBuf,
    process::Command,
};

use rustyline::{error::ReadlineError, Editor};

mod completion;
mod parser;

use completion::ShellHelper;
use parser::parse_command;

fn is_builtin(cmd: &str) -> bool {
    matches!(cmd, "echo" | "exit" | "type" | "pwd" | "cd")
}

fn execute_non_builtin_commands(tokens: &[String]) {
    let Some(command) = tokens.first() else {
        return;
    };

    let Some(path) = locate_executable_files(command) else {
        println!("{}: command not found", command);
        return;
    };

    let result = Command::new(path)
        .arg0(command)
        .args(&tokens[1..])
        .status();

    match result {
        Ok(status) if !status.success() => {
            eprintln!("{} exited with status {}", command, status);
        }

        Ok(_) => {},
        Err(error) => {
            eprintln!("failed to execute {}: {}", command, error);
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


fn func_change_directory(args: &[String]) {
    let directory = if args.is_empty() || args[0] == "~" {
        let Some(home) = env::var_os("HOME") else {
            eprintln!("cd: HOME is not set");
            return;
        };

        PathBuf::from(home)
    } else {
        PathBuf::from(&args[0])
    };

    if let Err(error) = env::set_current_dir(&directory) {
        match error.kind() {
            ErrorKind::NotFound => {
                eprintln!("cd: {}: No such file or directory", directory.display());
            }
            ErrorKind::PermissionDenied => {
                eprintln!("cd: {}: Permission denied", directory.display());
            }
            ErrorKind::NotADirectory => {
                eprintln!("cd: {}: Not a directory", directory.display());
            }
            _ => eprintln!("cd: {}: {}", directory.display(), error),
        }
    }
}


fn func_type(command: &str) {

    if is_builtin(command) {
        println!("{} is shell builtin", command);
        return;
    }

    match locate_executable_files(command) {
        Some(path) => println!("{} is {}", command, path.display()),
        None => println!("{}: not found", command),
    }


}



fn main() {

    let helper = ShellHelper::new();
    let mut editor = Editor::new().expect("Failed to create editor");
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
        if command.is_empty() { continue; }

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

        let Some(cmd) = tokens.first().map(String::as_str) else {
            continue;
        };

        match cmd {
            "exit" => break,
            "echo" => println!("{}", tokens[1..].join(" ")),
            "type" => {
                for arg in &tokens[1..] {
                    func_type(arg);
                }
            }
            "pwd" => println!("{}", env::current_dir().unwrap().display()),
            "cd" => func_change_directory(&tokens[1..]),
            _ => execute_non_builtin_commands(&tokens),

        }


    }

}
