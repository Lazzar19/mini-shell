
use std::{
    env,
    io::{self, Write},
    os::unix::fs::PermissionsExt,
    os::unix::process::CommandExt,
    path::PathBuf,
    process::Command,
};


fn is_builtin(cmd: &str) -> bool {
    matches!(cmd, "echo" | "exit" | "type" | "pwd")
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

    loop {

        print!("$ ");
        io::stdout().flush().unwrap();

        let mut command: String = String::new();

        if io::stdin().read_line(&mut command).unwrap() == 0 {
            break;
        };

        command = command.trim().to_string();

        let (cmd, args) = command.split_once(' ').unwrap_or((&command, ""));

        match cmd {
            "exit" => break,
            "echo" => println!("{}", args.trim_start()),
            "type" => for arg in args.split_whitespace() {
                func_type(arg);
            }
            "pwd" => println!("{}", env::current_dir().unwrap().display()),
            "" => continue,
            _ => execute_non_builtin_commands(cmd),

        }


    }


}
