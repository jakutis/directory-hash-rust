use std::fs;
use std::fs::PathExt;

pub struct Paths{dir: String, queue: Vec<Path>}

impl Iterator for Paths {
    type Item = Result<String, String>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.queue.pop() {
            None => None,
            Some(Path{is_dir: true, path: directory}) => {
                match read_dir_shallow(&self.dir, &directory) {
                    Ok(paths) => {
                        self.queue.extend(paths);
                        self.next()
                    },
                    Err(error) => Some(Err(error))
                }
            },
            Some(Path{is_dir: false, path: file}) =>
                Some(Ok(file))
        }
    }
}

pub fn read_dir(dir: &str, relative_path: &str) -> Paths {
    Paths{dir: dir.to_string(), queue: vec![Path{is_dir: true, path: relative_path.to_string()}]}
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

        let mut paths = read_dir(&ctx.dir, "");

        assert_eq!(paths.next(), None);

        after(ctx);
    }

    #[test]
    fn reads_directory_with_one_file() {
        let ctx = before();
        fs::File::create(format!("{}/{}", &ctx.dir, &ctx.file)).ok();

        let mut paths = read_dir(&ctx.dir, "");

        assert_eq!(paths.next(), Some(Ok(format!("/{}", &ctx.file))));
        assert_eq!(paths.next(), None);

        after(ctx);
    }

    #[test]
    fn reads_directory_with_one_subdirectory() {
        let ctx = before();
        fs::create_dir(format!("{}/{}", &ctx.dir, &ctx.subdir)).ok();

        let mut paths = read_dir(&ctx.dir, "");

        assert_eq!(paths.next(), None);

        after(ctx);
    }

    #[test]
    fn reads_directory_with_one_file_in_subdir() {
        let ctx = before();
        fs::create_dir(format!("{}/{}", &ctx.dir, &ctx.subdir)).ok();
        fs::File::create(format!("{}/{}/{}", &ctx.dir, &ctx.subdir, &ctx.file)).ok();

        let mut paths = read_dir(&ctx.dir, "");

        assert_eq!(paths.next(), Some(Ok(format!("/{}/{}", &ctx.subdir, &ctx.file))));
        assert_eq!(paths.next(), None);

        after(ctx);
    }

    #[test]
    fn reads_directory_in_sorted_order() {
        let ctx = before();
        fs::File::create(format!("{}/{}", &ctx.dir, &ctx.file)).ok();
        fs::File::create(format!("{}/{}", &ctx.dir, &ctx.file_b)).ok();

        let mut paths = read_dir(&ctx.dir, "");

        assert_eq!(paths.next(), Some(Ok(format!("/{}", &ctx.file))));
        assert_eq!(paths.next(), Some(Ok(format!("/{}", &ctx.file_b))));
        assert_eq!(paths.next(), None);

        after(ctx);
    }
}
