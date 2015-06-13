use std::io;
use std::fs;
use std::io::Read;

pub fn read_all(filename: &str) -> Result<Files, String> {
    let file = try!(fs::File::open(filename).map_err(|err| {
        format!("could not start reading {}; err: {}", filename, err)
    }));
    Ok(Files{bytes: file.bytes()})
}

pub struct Files {
    bytes: io::Bytes<fs::File>
}

impl Iterator for Files {
    type Item = Result<::File, String>;

    fn next(&mut self) -> Option<Self::Item> {
        match read_to_string_until(&mut self.bytes, 32) {
            None => None,
            Some(Err(err)) => Some(Err(err)),
            Some(Ok(hash)) => match read_to_string_until(&mut self.bytes, 10) {
                None => Some(Err("path is empty".to_string())),
                Some(Err(err)) => Some(Err(err)),
                Some(Ok(path)) => Some(Ok(::File {
                    hash: hash,
                    path: path
                }))
            }
        }
    }
}

fn read_to_string_until(bytes: &mut io::Bytes<fs::File>, byte_until: u8) -> Option<Result<String, String>> {
    let mut file_bytes = vec![];
    for byte in bytes {
        match byte {
            Ok(byte) => {
                if byte == byte_until {
                    break;
                }
                file_bytes.push(byte);
            },
            Err(err) => {
                return Some(Err(format!("cannot read file; err: {}", err)))
            }
        }
    }
    if file_bytes.len() > 0 {
        Some(String::from_utf8(file_bytes).map_err(|err| {
            format!("could not convert to utf8 string from byte array; err: {}", err)
        }))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use rand;
    use rand::Rng;
    use read_all::read_all;
    use std::io::Write;

    struct Context {
        dir: String, 
        file: String
    }

    fn before() -> Context {
        let mut rng = rand::thread_rng();
        let dir = format!("test.{}", rng.gen::<i32>());
        fs::create_dir(&dir).ok();

        return Context {
            dir: dir,
            file: format!("testA.{}", rng.gen::<i32>())
        };
    }

    fn after(ctx: Context) {
        fs::remove_dir_all(ctx.dir).ok();
    }

    #[test]
    fn reads_empty_file() {
        let ctx = before();
        fs::File::create(format!("{}/{}", &ctx.dir, &ctx.file)).unwrap();

        let files = read_all(&format!("{}/{}", &ctx.dir, &ctx.file));

        assert_eq!(files.unwrap().next(), None);

        after(ctx);
    }

    #[test]
    fn reads_file_with_two_lines() {
        let ctx = before();
        let mut file = fs::File::create(format!("{}/{}", &ctx.dir, &ctx.file)).unwrap();
        file.write("abc def\n".as_bytes()).unwrap();
        file.write("ghi jkl\n".as_bytes()).unwrap();

        let mut files = read_all(&format!("{}/{}", &ctx.dir, &ctx.file)).unwrap();

        assert_eq!(files.next(), Some(Ok(::File {path: "def".to_string(),hash:"abc".to_string()})));
        assert_eq!(files.next(), Some(Ok(::File {path: "jkl".to_string(),hash:"ghi".to_string()})));
        assert_eq!(files.next(), None);

        after(ctx);
    }

}
