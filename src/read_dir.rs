use std::fs;
use std::fs::PathExt;

pub fn read_dir(dir: &str, relative_path: &str) -> Result<Vec<String>, String> {
    let mut output = vec![];
    let mut queue = try!(read_dir_shallow(dir, relative_path));

    while !queue.is_empty() {
        match queue.pop().unwrap() {
            Path{is_dir: true, path: directory} =>
                queue.extend(try!(read_dir_shallow(dir, &directory))),
            Path{is_dir: false, path: file} =>
                output.push(file)
        }
    }

    Ok(output)
}

struct Path{is_dir: bool, path: String}

fn read_dir_shallow(dir: &str, relative_path: &str) -> Result<Vec<Path>, String> {
    let absolute_path = format!("{}{}", dir, relative_path);
    let entries = try!(fs::read_dir(&absolute_path).map_err(|err|
        format!("could not read dir: {}; err: {}", absolute_path, err)
    ));
    let mut paths: Vec<Path> = vec![];

    for entry in entries {
        let path = try!(entry.map_err(|err|
            format!("could not read entry in dir: {}; err: {}", absolute_path, err)
        )).path();

        let name_osstr = try!(path.file_name()
            .ok_or(format!("could not get entry filename in dir: {}", absolute_path))
        );

        let name = try!(name_osstr.to_str()
            .ok_or(format!("could not convert file OS string to string in dir: {}", absolute_path))
        ).to_string();

        paths.push(Path {
            is_dir: path.is_dir(),
            path: format!("{}/{}", relative_path, name)
        })
    }
    paths.sort_by(|a, b| b.path.cmp(&a.path));
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use rand;
    use rand::Rng;
    use read_dir::read_dir;

    struct Context {
        dir: String, 
        file: String,
        file_b: String,
        subdir: String
    }

    fn before() -> Context {
        let mut rng = rand::thread_rng();
        let dir = format!("test.{}", rng.gen::<i32>());
        fs::create_dir(&dir).ok();

        return Context {
            dir: dir,
            file: format!("testA.{}", rng.gen::<i32>()),
            file_b: format!("testB.{}", rng.gen::<i32>()),
            subdir: format!("test.{}", rng.gen::<i32>())
        };
    }

    fn after(ctx: Context) {
                fs::remove_dir_all(ctx.dir).ok();
    }

    #[test]
    fn reads_empty_directory() {
        let ctx = before();

        let paths = read_dir(&ctx.dir, "").unwrap();

        assert_eq!(paths.len(), 0);

        after(ctx);
    }

    #[test]
    fn reads_directory_with_one_file() {
        let ctx = before();
        fs::File::create(format!("{}/{}", &ctx.dir, &ctx.file)).ok();

        let paths = read_dir(&ctx.dir, "").unwrap();

        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], format!("/{}", &ctx.file));

        after(ctx);
    }

    #[test]
    fn reads_directory_with_one_subdirectory() {
        let ctx = before();
        fs::create_dir(format!("{}/{}", &ctx.dir, &ctx.subdir)).ok();

        let paths = read_dir(&ctx.dir, "").unwrap();

        assert_eq!(paths.len(), 0);

        after(ctx);
    }

    #[test]
    fn reads_directory_with_one_file_in_subdir() {
        let ctx = before();
        fs::create_dir(format!("{}/{}", &ctx.dir, &ctx.subdir)).ok();
        fs::File::create(format!("{}/{}/{}", &ctx.dir, &ctx.subdir, &ctx.file)).ok();

        let paths = read_dir(&ctx.dir, "").unwrap();

        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], format!("/{}/{}", &ctx.subdir, &ctx.file));

        after(ctx);
    }

    #[test]
    fn reads_directory_in_sorted_order() {
        let ctx = before();
        fs::File::create(format!("{}/{}", &ctx.dir, &ctx.file)).ok();
        fs::File::create(format!("{}/{}", &ctx.dir, &ctx.file_b)).ok();

        let paths = read_dir(&ctx.dir, "").unwrap();

        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0], format!("/{}", &ctx.file));
        assert_eq!(paths[1], format!("/{}", &ctx.file_b));

        after(ctx);
    }
}
