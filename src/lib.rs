#![feature(path_ext)]
#![feature(core)]

extern crate openssl;
extern crate rand;
extern crate lazysort;
extern crate core;

use std::io;
use std::str;
use std::fs;
use openssl::crypto::hash;
use core::cmp;
use core::clone;

use rand::Rng;
use std::io::Write;
use std::io::Read;
use std::fs::PathExt;
use lazysort::Sorted;

struct DirEntry {
is_dir: bool,
dir: String,
filename: String
}

fn format_line<'a>(path: &'a str, hash: &'a str) -> String {
        hash.to_string() + " " + path + "\n"
}

impl clone::Clone for DirEntry {
fn clone(& self) -> DirEntry {
    DirEntry {dir: self.dir.clone(), is_dir: self.is_dir, filename: self.filename.clone()}
}
}

impl cmp::Eq for DirEntry {

}

impl cmp::PartialEq for DirEntry {
fn eq(& self, other: &DirEntry) -> bool {
    self.filename == other.filename
}
fn ne(& self, other: &DirEntry) -> bool {
    self.filename != other.filename
}
}

impl cmp::Ord for DirEntry {
fn cmp(& self, other: &DirEntry) -> cmp::Ordering {
        self.filename.cmp(&other.filename).reverse()
}
}

impl cmp::PartialOrd for DirEntry {
fn partial_cmp(& self, other: &DirEntry) -> Option<cmp::Ordering> {
        Some(self.filename.partial_cmp(&other.filename).unwrap().reverse())
}
}

impl DirEntry {
fn from_dir_entry(dir: &str, entry: fs::DirEntry) -> DirEntry {
    let path = entry.path();
    DirEntry {dir: dir.to_string(), is_dir: path.is_dir(), filename: path.file_name().unwrap().to_str().unwrap().to_string()}
}
}

fn read_dir<'_>(dir: &str, relative_dir: &str) -> lazysort::LazySortIterator<'_, DirEntry> {
                        fs::read_dir(&format!("{}{}", dir, relative_dir)).ok().unwrap().map(|entry| DirEntry::from_dir_entry(relative_dir, entry.ok().unwrap())).sorted()
}

pub fn hash(dir: &str, sink: &mut io::Write) -> () {
    let mut queue: Vec<_> = read_dir(dir, "").collect();
    while !queue.is_empty() {
        let entry = queue.pop().unwrap();
        let relative_path = &format!("{}/{}", entry.dir, entry.filename);
        if entry.is_dir {
            for entry in read_dir(dir, relative_path) {
                    queue.push(entry);
            }
        } else {
            let mut bytes = vec![];
            fs::File::open(format!("{}{}", dir, relative_path)).ok().unwrap().read_to_end(&mut bytes).ok();

            let hash_str = hash::hash(hash::Type::SHA512, &bytes)
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .fold("".to_string(), |hash_str, byte_str| hash_str + &byte_str);

            sink.write_all(format_line(relative_path, &hash_str).as_bytes()).ok().unwrap();
        }
    }
}

struct Output {
buffer: Vec<u8>
}

impl Output {
fn to_str(& self) -> &str {
            str::from_utf8(& self .buffer).unwrap()
}
}

impl io::Write for Output {
fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    for b in buf.iter() {
            self .buffer.push(*b);
    }
    Ok(buf.len())
}
fn flush(&mut self) -> io::Result<()> { Ok(())}
}

#[test]
fn hashes_directory_with_a_nonempty_subdir_and_file() {
    let mut rng = rand::thread_rng();
    let dir = &format!("test.{}", rng.gen::<i32>());
    let file1 = "B";
    let subdir = "A";
    let file2 = "A";
    let mut output = Output {
    buffer: vec![]
    };
    fs::create_dir(dir).ok();
    fs::create_dir(format!("{}/{}", dir, subdir)).ok();
    fs::File::create(format!("{}/{}", dir, file1)).ok();
    fs::File::create(format!("{}/{}/{}", dir, subdir, file2)).ok();

    hash(dir, &mut output);

    let hash = "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e";
    assert_eq!(output.to_str(), format_line(&format!("/{}/{}", subdir, file2), hash) + &format_line(&format!("/{}", file1), hash));

    fs::remove_dir_all(dir).ok();
}

#[test]
fn hashes_directory_with_a_nonempty_subdir() {
    let mut rng = rand::thread_rng();
    let dir = &format!("test.{}", rng.gen::<i32>());
    let subdir = &format!("test.{}", rng.gen::<i32>());
    let file = &format!("test.{}", rng.gen::<i32>());
    let mut output = Output {
    buffer: vec![]
    };
    fs::create_dir(dir).ok();
    fs::create_dir(format!("{}/{}", dir, subdir)).ok();
    fs::File::create(format!("{}/{}/{}", dir, subdir, file)).ok().unwrap().write_all("testas".as_bytes()).ok();

    hash(dir, &mut output);

    let hash = "2e3c6bb28df6cb0603f00fdf520539200d05ab237a1348ec1c598e8c6864d93f6a6da9c81b5ae7117687d9e1b1b41682afc2d02269854b5779a2bd645917e05c";
    assert_eq!(output.to_str(), format_line(&format!("/{}/{}", subdir, file), hash));

    fs::remove_dir_all(dir).ok();
}

#[test]
fn hashes_directory_sorted_by_filename() {
    let mut rng = rand::thread_rng();
    let dir = &format!("test.{}", rng.gen::<i32>());
    let file_a = "A";
    let file_b = "B";
    let mut output = Output {
    buffer: vec![]
    };
    fs::create_dir(dir).ok();
    fs::File::create(format!("{}/{}", dir, file_a)).ok().unwrap().write_all("testas".as_bytes()).ok();
    fs::File::create(format!("{}/{}", dir, file_b)).ok().unwrap().write_all("testas2".as_bytes()).ok();

    hash(dir, &mut output);

    let hashA = "2e3c6bb28df6cb0603f00fdf520539200d05ab237a1348ec1c598e8c6864d93f6a6da9c81b5ae7117687d9e1b1b41682afc2d02269854b5779a2bd645917e05c";
    let hashB = "47a968f5324c4cb0225c65948e30b3681f348f6ed9d4b4d6968f870743a93ea1cb4597247868442431edb5e858942c95146e1f82704d37a6d3ab9515cab8fd0c";
    assert_eq!(output.to_str(), format_line(&format!("/{}", file_a), hashA) + &format_line(&format!("/{}", file_b), hashB));

    fs::remove_dir_all(dir).ok();
}

#[test]
fn hashes_directory_with_one_nonempty_file() {
    let mut rng = rand::thread_rng();
    let dir = &format!("test.{}", rng.gen::<i32>());
    let file = &format!("test.{}", rng.gen::<i32>());
    let mut output = Output {
    buffer: vec![]
    };
    fs::create_dir(dir).ok();
    fs::File::create(format!("{}/{}", dir, file)).ok().unwrap().write_all("testas".as_bytes()).ok();

    hash(dir, &mut output);

    let hash = "2e3c6bb28df6cb0603f00fdf520539200d05ab237a1348ec1c598e8c6864d93f6a6da9c81b5ae7117687d9e1b1b41682afc2d02269854b5779a2bd645917e05c";
    assert_eq!(output.to_str(), format_line(&format!("/{}", file), hash));

    fs::remove_dir_all(dir).ok();
}

#[test]
fn hashes_directory_with_one_empty_file() {
    let mut rng = rand::thread_rng();
    let dir = &format!("test.{}", rng.gen::<i32>());
    let file = &format!("test.{}", rng.gen::<i32>());
    let mut output = Output {
    buffer: vec![]
    };
    fs::create_dir(dir).ok();
    fs::File::create(format!("{}/{}", dir, file)).ok();

    hash(dir, &mut output);

    let hash = "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e";
    assert_eq!(output.to_str(), format_line(&format!("/{}", file), hash));

    fs::remove_dir_all(dir).ok();
}

#[test]
fn hashes_empty_directory() {
    let mut rng = rand::thread_rng();
    let dir = &format!("test.{}", rng.gen::<i32>());
    let mut output = Output {
    buffer: vec![]
    };
    fs::create_dir(dir).ok();

    hash(dir, &mut output);

    assert_eq!(output.to_str(), "");

    fs::remove_dir_all(dir).ok();
}
