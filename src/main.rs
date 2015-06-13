extern crate dirhash;

use std::env;
use std::io;

pub fn main() {
    let mut out = io::stdout();
    let info = "Available commands are hash and list_errors.".to_string();

    match env::args().nth(1) {
        Some(command) => match command.as_ref() {
            "hash" => match env::args().nth(2) {
                Some(directory) => dirhash::hash(&directory, &mut out),
                None => println!("Directory unspecified.")
            },
            "list_errors" => match env::args().nth(2) {
                Some(directory) => dirhash::list_errors(&directory, &mut out),
                None => println!("Directory unspecified.")
            },
            _ => println!("Unrecognized command {}. {}", command, info)
        },
        None => println!("Command unspecified. {}", info)
    }
}
