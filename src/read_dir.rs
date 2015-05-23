extern crate rand;

use rand::Rng;
use std::fs;
use std::fs::PathExt;
use std::io;
use std::path;
use std::iter;
use core::cmp;

fn read_dir(dir: &str, relative_path: &str) -> Vec<String> {
    let mut queue = read_dir_shallow(dir, relative_path);
    let mut output = vec![];

    while !queue.is_empty() {
        let path = queue
            .pop()
            .unwrap_or_else(||
                panic!("could not pop from non-empty queue")
            );

        if(path.is_dir) {
            queue.extend(read_dir_shallow(dir, &path.path));
        } else {
            output.push(path.path);
        }
    }

    output
}

struct Path{is_dir: bool, path: String}

fn read_dir_shallow(dir: &str, relative_path: &str) -> Vec<Path> {
    let absolute_path = format!("{}{}", dir, relative_path);
    let path_to_string = |path: path::PathBuf| path
        .file_name()
        .unwrap_or_else(||
            panic!(format!("could not get entry filename in dir: {}", absolute_path))
        )
        .to_str()
        .unwrap_or_else(||
            panic!(format!("could not convert file OS string to string in dir: {}", absolute_path))
        )
        .to_string();

    let mut paths: Vec<Path> = fs::read_dir(&absolute_path)
        .unwrap_or_else(|err|
            panic!(format!("could not read dir: {}; err: {}", absolute_path, err))
        )
        .map(|entry| entry
            .unwrap_or_else(|err|
                panic!(format!("could not read entry in dir: {}; err: {}", absolute_path, err))
            )
            .path()
        )
        .map(|path| Path {
            is_dir: path.is_dir(),
            path: format!("{}/{}", relative_path, path_to_string(path))
        })
        .collect();

    paths.sort_by(|a, b| b.path.cmp(&a.path));

    paths
}

struct Context {
    dir: String, 
    file: String,
    fileB: String,
    subdir: String
}

fn before() -> Context {
    let mut rng = rand::thread_rng();
    let dir = format!("test.{}", rng.gen::<i32>());
    fs::create_dir(&dir).ok();

    return Context {
        dir: dir,
        file: format!("testA.{}", rng.gen::<i32>()),
        fileB: format!("testB.{}", rng.gen::<i32>()),
        subdir: format!("test.{}", rng.gen::<i32>())
    };
}

fn after(ctx: Context) {
            fs::remove_dir_all(ctx.dir).ok();
}

#[test]
fn reads_empty_directory() {
    let ctx = before();

    let paths = read_dir(&ctx.dir, "");

    assert_eq!(paths.len(), 0);

    after(ctx);
}

#[test]
fn reads_directory_with_one_file() {
    let ctx = before();
    fs::File::create(format!("{}/{}", &ctx.dir, &ctx.file)).ok();

    let paths = read_dir(&ctx.dir, "");

    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0], format!("/{}", &ctx.file));

    after(ctx);
}

#[test]
fn reads_directory_with_one_subdirectory() {
    let ctx = before();
    fs::create_dir(format!("{}/{}", &ctx.dir, &ctx.subdir)).ok();

    let paths = read_dir(&ctx.dir, "");

    assert_eq!(paths.len(), 0);

    after(ctx);
}

#[test]
fn reads_directory_with_one_file_in_subdir() {
    let ctx = before();
    fs::create_dir(format!("{}/{}", &ctx.dir, &ctx.subdir)).ok();
    fs::File::create(format!("{}/{}/{}", &ctx.dir, &ctx.subdir, &ctx.file)).ok();

    let paths = read_dir(&ctx.dir, "");

    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0], format!("/{}/{}", &ctx.subdir, &ctx.file));

    after(ctx);
}

#[test]
fn reads_directory_in_sorted_order() {
    let ctx = before();
    fs::File::create(format!("{}/{}", &ctx.dir, &ctx.file)).ok();
    fs::File::create(format!("{}/{}", &ctx.dir, &ctx.fileB)).ok();

    let paths = read_dir(&ctx.dir, "");

    assert_eq!(paths.len(), 2);
    assert_eq!(paths[0], format!("/{}", &ctx.file));
    assert_eq!(paths[1], format!("/{}", &ctx.fileB));

    after(ctx);
}
