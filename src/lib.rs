#![feature(path_ext)]

extern crate rand;
extern crate openssl;

use std::io;

mod read_dir;
mod hash;

pub fn hash(dir: &str, sink: &mut io::Write) -> () {
    for file in hash::hash(dir).unwrap() {
        sink.write_all(file.to_string().as_bytes()).map_err(|err|
            format!("could not output: {}; err: {}", file.to_string(), err)
        ).unwrap();
    }
}

/*
 * TODO mock the hash::hash() and test hash()
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
*/

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

