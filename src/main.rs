extern crate dirhash;

use std::env;
use std::io;

pub fn main() {
    let path = env::current_dir().ok().unwrap();
    let filename = path.to_str().unwrap();
    let mut out = io::stdout();
    let info = "Available commands are hash and list_errors.".to_string();

    match env::args().nth(1) {
        Some(command) => match command.as_ref() {
            "hash" => {
                dirhash::hash(filename, &mut out);
            },
            "list_errors" => {
                dirhash::list_errors(filename, &mut out);
            },
            _ => println!("Unrecognized command {}. {}", command, info)
        },
        None => println!("Command unspecified. {}", info)
    }
}
