use core::error;
use std::env;
use std::os::unix::fs::PermissionsExt;
use std::{io::Read, process::{Command, Termination}};
#[allow(unused_imports)]
use std::io::{self, Write};

fn is_builtin(cmd: &str) -> bool {
    matches!(cmd, "echo" | "exit" | "type")
}


fn func_type(command: &str) {

    if is_builtin(command) {
        println!("{} is shell builtin", command);
        return;
    }

    let Some(path) = env::var_os("PATH") else { // checking for PATH variables
        println!("{}: not found", command);
        return;
    };

    for dir in env::split_paths(&path) {

        let candidate = dir.join(command); 

        let Ok(metadata) = candidate.metadata() else {
            continue;
        };

        // checking whether it is a executable file
        // 0o100 => Owner,  0o010 => Group, 0o001 => Others
        // checking for just one execute bit
        if metadata.is_file() && metadata.permissions().mode() & 0o111 != 0 { 
            println!("{} is {}", command, candidate.display());
            return;
        }

    }

    println!("{}: not found", command);



}

fn main() {

    loop {

        print!("$ ");
        io::stdout().flush().unwrap();

        let mut command: String = String::new();

        io::stdin().read_line(&mut command).unwrap();
        command = command.trim().to_string();

        let (cmd, args) = command.split_once(' ').unwrap_or((&command, ""));

        match cmd {
            "exit" => break,
            "echo" => println!("{}", args.trim_start()),
            "type" => for arg in args.split_whitespace() {
                func_type(&command);
            }
            "" => continue,
            _ => println!("{}: comamnd not found", &cmd)

        }


    }


}
