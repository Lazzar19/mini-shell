use core::error;
use std::{io::Read, process::{Command, Termination}};
#[allow(unused_imports)]
use std::io::{self, Write};

fn is_builtin(cmd: &str) -> bool {
    matches!(cmd, "echo" | "exit" | "type")
}


fn func_type(arg: &str) {

    if is_builtin(arg) {
        println!("{} is shell builtin", arg);
    } else {
        println!("{}: not found", arg);
    }


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
            "type" => func_type(args),
            "" => continue,
            _ => println!("{}: comamnd not found", &cmd)

        }


    }


}
