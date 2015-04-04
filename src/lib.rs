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
filename: String
}

fn format_line<'a>(path: &'a str, hash: &'a str) -> String {
        hash.to_string() + " " + path + "\n"
}

impl clone::Clone for DirEntry {
fn clone(& self) -> DirEntry {
    DirEntry {is_dir: self.is_dir, filename: self.filename.clone()}
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
        self.filename.cmp(&other.filename)
}
}

impl cmp::PartialOrd for DirEntry {
fn partial_cmp(& self, other: &DirEntry) -> Option<cmp::Ordering> {
        self.filename.partial_cmp(&other.filename)
}
}

impl DirEntry {
fn fromDirEntry(entry: fs::DirEntry) -> DirEntry {
    let path = entry.path();
    DirEntry {is_dir: path.is_dir(), filename: path.file_name().unwrap().to_str().unwrap().to_string()}
}
}

pub fn hash(dir: &str, relative_dir: &str, sink: &mut io::Write) -> io::Result<()> {
    let mut s = "".to_string();
    let full_dir = &format!("{}/{}", dir, relative_dir);
    for entry in fs::read_dir(full_dir).ok().unwrap().map(|entry| DirEntry::fromDirEntry(entry.ok().unwrap())).sorted() {
        if entry.is_dir {
                hash(dir, &format!("{}/{}", relative_dir, entry.filename), sink);
        } else {
            let mut bytes = vec![];
            let mut file = fs::File::open(format!("{}/{}", full_dir, entry.filename)).ok().unwrap();
            file.read_to_end(&mut bytes).ok();
            let hash = hash::hash(hash::Type::SHA512, &bytes);
            let mut hash_str = "".to_string();
            for byte in hash.iter() {
                    hash_str.push_str(&format!("{:02x}", byte));
            }
            s.push_str(&format_line(&format!("{}/{}", relative_dir, entry.filename), &hash_str));
        }
    }
    sink.write_all(s.as_bytes())
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

    hash(dir, "", &mut output).ok().unwrap();

    let hash = "2e3c6bb28df6cb0603f00fdf520539200d05ab237a1348ec1c598e8c6864d93f6a6da9c81b5ae7117687d9e1b1b41682afc2d02269854b5779a2bd645917e05c";
    assert_eq!(output.to_str(), format_line(&format!("/{}/{}", subdir, file), hash));

    fs::remove_dir_all(dir).ok();
}

#[test]
fn hashes_directory_sorted_by_filename() {
    let mut rng = rand::thread_rng();
    let dir = &format!("test.{}", rng.gen::<i32>());
    let fileA = "A";
    let fileB = "B";
    let mut output = Output {
    buffer: vec![]
    };
    fs::create_dir(dir).ok();
    fs::File::create(format!("{}/{}", dir, fileA)).ok().unwrap().write_all("testas".as_bytes()).ok();
    fs::File::create(format!("{}/{}", dir, fileB)).ok().unwrap().write_all("testas2".as_bytes()).ok();

    hash(dir, "", &mut output).ok().unwrap();

    let hashA = "2e3c6bb28df6cb0603f00fdf520539200d05ab237a1348ec1c598e8c6864d93f6a6da9c81b5ae7117687d9e1b1b41682afc2d02269854b5779a2bd645917e05c";
    let hashB = "47a968f5324c4cb0225c65948e30b3681f348f6ed9d4b4d6968f870743a93ea1cb4597247868442431edb5e858942c95146e1f82704d37a6d3ab9515cab8fd0c";
    assert_eq!(output.to_str(), format_line(&format!("/{}", fileA), hashA) + &format_line(&format!("/{}", fileB), hashB));

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

    hash(dir, "", &mut output).ok().unwrap();

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

    hash(dir, "", &mut output).ok().unwrap();

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

    hash(dir, "", &mut output).ok().unwrap();

    assert_eq!(output.to_str(), "");

    fs::remove_dir_all(dir).ok();
}
