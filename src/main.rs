extern crate dirhash;

use std::env;
use std::io;

pub fn main() {
    let path = env::current_dir().ok().unwrap();
    let filename = path.to_str().unwrap();
    let mut out = io::stdout();
    dirhash::hash(filename, &mut out).ok();
}
