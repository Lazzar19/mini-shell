use core::error;
use std::{io::Read, process::Command};
#[allow(unused_imports)]
use std::io::{self, Write};


fn main() {

    while true {

        print!("$ ");
        io::stdout().flush().unwrap();

        let mut command: String = String::new();

        io::stdin().read_line(&mut command).unwrap();
        println!("{}: command not found", command.trim());


    }


}
