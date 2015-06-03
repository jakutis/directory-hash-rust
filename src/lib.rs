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

mod read_dir;

struct File {
    path: String,
    hash: String
}

impl ToString for File {
    fn to_string(&self) -> String {
        if self.path.contains("\n") {
            panic!(format!("path {} contains a newline character", self.path));
        }
        self.hash.to_string() + " " + &self.path + "\n"
    }
}

pub fn hash(dir: &str, sink: &mut io::Write) -> () {
    match read_dir::read_dir(dir, "") {
        Ok(relative_paths) => {
            for relative_path in relative_paths {
                let absolute_path = format!("{}{}", dir, relative_path);
                match fs::File::open(&absolute_path) {
                    Ok(mut file) => {
                        let mut hasher = hash::Hasher::new(hash::Type::SHA512);

                        match io::copy(&mut file, &mut hasher) {
                            Ok(..) => (),
                            Err(err) => panic!("error hashing file \"{}\": {}", &absolute_path, err)
                        };

                        let file = File {
                            path: relative_path,
                            hash: hasher.finish()
                                    .iter()
                                    .map(|byte| format!("{:02x}", byte))
                                    .fold("".to_string(), |hash_str, byte_str| hash_str + &byte_str)
                        };

                        match sink.write_all(file.to_string().as_bytes()) {
                            Ok(result) => result,
                            Err(err) => panic!(format!("could not output: {}; err: {}", file.to_string(), err))
                        };
                    },
                    _ => ()
                };
            }
        }
        Err(err) => panic!(err)
    }
}

struct Output {
buffer: Vec<u8>
}

impl Output {
fn to_str(& self) -> &str {
    match str::from_utf8(& self .buffer) {
        Ok(str) => str,
        Err(err) => panic!(format!("could not convert a buffer to string; err: {}", err))
    }
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

/*
pub fn import(dir: &str, all: &str, path: &str, absolute_path: &str, backup_dir: &str, sink: &mut io::Write) -> () {
}

#[test]
fn imports_new_files() {
    let mut rng = rand::thread_rng();
    let dir = &format!("test.{}", rng.gen::<i32>());
    let mut output = Output {
    buffer: vec![]
    };
    fs::create_dir(dir).ok();

    /*
    // directory all.file path absolute-path backup-directory
    import(dir, "", "", "", "", &mut output);

    Finds all new, changed and missing paths.
	Backs up files (actually happens in the next step, to avoid copying).
	Moves new and changed, deletes missing.
	Lists all paths and their hashes.
     */
    let current_paths = read_dir(dir, path);
    let known_items = read_all(all);
    let new_items = find_new(known_items, current_paths);
    let changed_items = find_changed(known_items, current_paths);
    let missing_items = find_missing(known_items, current_paths);
    backup_move_cleanup(dir, target_dir, new_items);
    backup_move_cleanup(dir, target_dir, changed_items);
    delete(target_dir, missing_items);
    merge_add(known_items, new_items);
    merge_update(known_items, changed_items);
    merge_remove(known_items, missing_items);
    output(known_items);

    fs::remove_dir_all(dir).ok();
}

pub fn list_paths_with_utf8_errors(TODO) -> ()

fn get_lossy(dir: &str, entry: fs::DirEntry) -> Option<String> {
    match entry.path().file_name() {
        Some(filename) => match filename.to_str() {
            Some(..) => None,
            None => Some(filename.to_string_lossy().into_owned().replace("�", "_"))
        },
        None => panic!(format!("could not get some filename in dir: {}", dir.to_string()))
    }
}

fn rename_if_utf_errors(root: &str, dir: &str, entry: fs::DirEntry) -> Result<(), io::Error> {
    match entry.path().file_name() {
        Some(filename) => match filename.to_str() {
            Some(..) => Ok(()),
            None => {
                let lossy = filename.to_string_lossy().into_owned().replace("�", "_");
                fs::rename(entry.path(), format!("{}{}/{}", root, dir.to_string(), &lossy))
            }
        },
        None => panic!(format!("could not get some filename in dir: {}", dir.to_string()))
    }
}
*/

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
    assert_eq!(output.to_str(), File{path: format!("/{}/{}", subdir, file2), hash: hash.to_string()}.to_string() + &File{path: format!("/{}", file1), hash: hash.to_string()}.to_string());

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
    assert_eq!(output.to_str(), File{path: format!("/{}/{}", subdir, file), hash: hash.to_string()}.to_string());

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
    assert_eq!(output.to_str(), File{path: format!("/{}", file_a), hash: hashA.to_string()}.to_string() + &File{path: format!("/{}", file_b), hash: hashB.to_string()}.to_string());

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
    assert_eq!(output.to_str(), File{path: format!("/{}", file), hash: hash.to_string()}.to_string());

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
    assert_eq!(output.to_str(), File {path: format!("/{}", file), hash: hash.to_string()}.to_string());

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
