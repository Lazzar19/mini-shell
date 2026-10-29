
use std::{
    env,
    fs::File,
    io::{ErrorKind, Write},
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

fn split_stdout_redirect(tokens: &[String],) -> Result<(&[String], Option<&str>), String> {
    let Some(operator_index) = tokens.iter().position(|token| token == ">" || token == "1>") else {
        return Ok((tokens, None));
    };

    let Some(filename) = tokens.get(operator_index + 1) else {
        return Err("syntax error: expected filename after '>'".to_string());
    };

    if operator_index == 0 {
        return Err("syntax error: expected command before '>'".to_string());
    }

    if operator_index + 2 < tokens.len() {
        return Err("syntax error: unexpected token after redirect filename".to_string());
    }

    Ok((&tokens[..operator_index], Some(filename)))
}

fn execute_non_builtin_commands(tokens: &[String], redirect: Option<&str>) {
    let Some(command) = tokens.first() else {
        return;
    };

    let Some(path) = locate_executable_files(command) else {
        println!("{}: command not found", command);
        return;
    };

    let mut process = Command::new(path);
    process.arg0(command).args(&tokens[1..]);

    if let Some(filename) = redirect {
        let output_file = match File::create(filename) {
            Ok(file) => file,
            Err(error) => {
                eprintln!("{}: {}", filename, error);
                return;
            }
        };

        process.stdout(output_file);
    }

    let result = process.status();

    match result {
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


fn change_directory(args: &[String]) {
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

        let (tokens, redirect) = match split_stdout_redirect(&tokens) {
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

                if let Some(filename) = redirect {
                    match File::create(filename) {
                        Ok(mut file) => {
                            if let Err(error) = writeln!(file, "{output}") {
                                eprintln!("{}: {}", filename, error);
                            }
                        }
                        Err(error) => eprintln!("{}: {}", filename, error),
                    }
                } else {
                    println!("{output}");
                }
            }
            "type" => {
                for arg in &tokens[1..] {
                    func_type(arg);
                }
            }
            "pwd" => println!("{}", env::current_dir().unwrap().display()),
            "cd" => change_directory(&tokens[1..]),
            _ => execute_non_builtin_commands(tokens, redirect),

        }


    }

}
