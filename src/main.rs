
use std::{
    env,
    io::ErrorKind,
    os::unix::fs::PermissionsExt,
    os::unix::process::CommandExt,
    path::PathBuf,
    process::Command,
};

use rustyline::{error::ReadlineError, Editor};

mod completion;
use completion::ShellHelper;


fn is_builtin(cmd: &str) -> bool {
    matches!(cmd, "echo" | "exit" | "type" | "pwd" | "cd")
}



fn execute_non_builtin_commands(input: &str) {

    let mut parts = input.split_whitespace();
    let Some(command) = parts.next() else {
        return;
    };

    let args: Vec<&str> = parts.collect();

    let Some(path) = locate_executable_files(command) else {
        println!("{}: command not found", command);
        return;
    };


    let result = Command::new(path).arg0(command).args(args).status();

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


fn func_change_directory(args: &str) {
    let argument = args.trim();

    let directory = if argument.is_empty() || argument == "~" {
        let Some(home) = env::var_os("HOME") else {
            eprintln!("cd: HOME is not set");
            return;
        };

        PathBuf::from(home)
    } else {
        PathBuf::from(argument)
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

        let (cmd, args) = command.split_once(' ').unwrap_or((&command, ""));

        match cmd {
            "exit" => break,
            "echo" => println!("{}", args.trim_start()),
            "type" => for arg in args.split_whitespace() {
                func_type(arg);
            }
            "pwd" => println!("{}", env::current_dir().unwrap().display()),
            "cd" => func_change_directory(args),
            "" => continue,
            _ => execute_non_builtin_commands(command),

        }


    }


}
