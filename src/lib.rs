extern crate rand;
extern crate openssl;

use std::io;
use std::cmp;
use std::fmt;

mod read_dir;
mod read_all;
mod hash;

fn output(string: String, sink: &mut io::Write) -> () {
    sink.write_all(string.as_bytes()).map_err(|err| format!("could not output: {}; err: {}", string, err)).unwrap()
}

pub fn hash(dir: &str, sink: &mut io::Write) -> () {
    let files = read_dir::read_dir(dir, "").map(|relative_path| {
        match relative_path {
            Err(err) => panic!(format!("could not list dir {}; err: {}", dir.to_string(), err)),
            Ok(relative_path) => hash::hash(dir, &relative_path)
        }
    });
    for file in files {
        output(file.unwrap().to_string(), sink);
    }
}

pub fn import(all_filename: &str, sink: &mut io::Write) -> () {
    read_all::read_all(all_filename);
}

pub fn list_errors(dir: &str, sink: &mut io::Write) -> () {
    for file in read_dir::read_dir(dir, "") {
        match file {
            Err(err) => output(format!("{}\n", err), sink),
            Ok(path) => {
                if path.contains("\n") {
                    output(format!("path {} contains a newline character\n", path), sink)
                }
            }
        }
    }
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

pub struct File {
    path: String,
    hash: String
}

impl fmt::Debug for File {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.to_string())
    }
}

impl cmp::PartialEq for File {
    fn eq(& self, other: &File) -> bool {
        self.path == other.path && self.hash == other.hash
    }
}

impl ToString for File {
    fn to_string(&self) -> String {
        if self.path.contains("\n") {
            panic!(format!("path {} contains a newline character", self.path));
        }
        self.hash.to_string() + " " + &self.path + "\n"
    }
}
