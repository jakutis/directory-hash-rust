use std::cmp;
use std::fmt;
use std::io;
use std::fs;
use openssl::crypto::hash;
use read_dir;

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

pub fn hash(dir: &str) -> Result<Vec<File>, String> {
    let mut files: Vec<File> = vec![];

    for relative_path in try!(read_dir::read_dir(dir, "")) {
        let absolute_path = format!("{}{}", dir, relative_path);

        let mut file = try!(fs::File::open(&absolute_path).map_err(|err|
            format!("error opening file {}: {}", absolute_path, err)
        ));
        let mut hasher = hash::Hasher::new(hash::Type::SHA512);

        try!(io::copy(&mut file, &mut hasher).map_err(|err|
            format!("error hashing file {}: {}", absolute_path, err)
        ));

        files.push(File {
            path: relative_path,
            hash: hasher.finish()
                    .iter()
                    .map(|byte| format!("{:02x}", byte))
                    .fold("".to_string(), |hash_str, byte_str| hash_str + &byte_str)
        });
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use rand;
    use rand::Rng;
    use hash::hash;
    use hash::File;
    use std::io::Write;

    #[test]
    fn hashes_directory_with_a_nonempty_subdir_and_file() {
        let mut rng = rand::thread_rng();
        let dir = &format!("test.{}", rng.gen::<i32>());
        let file1 = "B";
        let subdir = "A";
        let file2 = "A";
        fs::create_dir(dir).unwrap();
        fs::create_dir(format!("{}/{}", dir, subdir)).unwrap();
        fs::File::create(format!("{}/{}", dir, file1)).unwrap();
        fs::File::create(format!("{}/{}/{}", dir, subdir, file2)).unwrap();

        let files = hash(dir).unwrap();

        let hash = "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e".to_string();
        assert_eq!(files, vec![
            File{path: format!("/{}/{}", subdir, file2), hash: hash.clone()},
            File{path: format!("/{}", file1), hash: hash}
        ]);

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn hashes_directory_with_a_nonempty_subdir() {
        let mut rng = rand::thread_rng();
        let dir = &format!("test.{}", rng.gen::<i32>());
        let subdir = &format!("test.{}", rng.gen::<i32>());
        let file = &format!("test.{}", rng.gen::<i32>());
        fs::create_dir(dir).unwrap();
        fs::create_dir(format!("{}/{}", dir, subdir)).unwrap();
        fs::File::create(format!("{}/{}/{}", dir, subdir, file)).unwrap().write_all("testas".as_bytes()).unwrap();

        let files = hash(dir).unwrap();

        let hash = "2e3c6bb28df6cb0603f00fdf520539200d05ab237a1348ec1c598e8c6864d93f6a6da9c81b5ae7117687d9e1b1b41682afc2d02269854b5779a2bd645917e05c".to_string();
        assert_eq!(files, vec![
            File{path: format!("/{}/{}", subdir, file), hash: hash.to_string()}
        ]);

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn hashes_directory_sorted_by_filename() {
        let mut rng = rand::thread_rng();
        let dir = &format!("test.{}", rng.gen::<i32>());
        let file_a = "A";
        let file_b = "B";
        fs::create_dir(dir).unwrap();
        fs::File::create(format!("{}/{}", dir, file_a)).unwrap().write_all("testas".as_bytes()).unwrap();
        fs::File::create(format!("{}/{}", dir, file_b)).unwrap().write_all("testas2".as_bytes()).unwrap();

        let files = hash(dir).unwrap();

        let hash_a = "2e3c6bb28df6cb0603f00fdf520539200d05ab237a1348ec1c598e8c6864d93f6a6da9c81b5ae7117687d9e1b1b41682afc2d02269854b5779a2bd645917e05c";
        let hash_b = "47a968f5324c4cb0225c65948e30b3681f348f6ed9d4b4d6968f870743a93ea1cb4597247868442431edb5e858942c95146e1f82704d37a6d3ab9515cab8fd0c";
        assert_eq!(files, vec![
            File{path: format!("/{}", file_a), hash: hash_a.to_string()},
            File{path: format!("/{}", file_b), hash: hash_b.to_string()}
        ]);

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn hashes_directory_with_one_nonempty_file() {
        let mut rng = rand::thread_rng();
        let dir = &format!("test.{}", rng.gen::<i32>());
        let file = &format!("test.{}", rng.gen::<i32>());
        fs::create_dir(dir).unwrap();
        fs::File::create(format!("{}/{}", dir, file)).unwrap().write_all("testas".as_bytes()).unwrap();

        let files = hash(dir).unwrap();

        let hash = "2e3c6bb28df6cb0603f00fdf520539200d05ab237a1348ec1c598e8c6864d93f6a6da9c81b5ae7117687d9e1b1b41682afc2d02269854b5779a2bd645917e05c".to_string();
        assert_eq!(files, vec![
            File{path: format!("/{}", file), hash: hash}
        ]);

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn hashes_directory_with_one_empty_file() {
        let mut rng = rand::thread_rng();
        let dir = &format!("test.{}", rng.gen::<i32>());
        let file = &format!("test.{}", rng.gen::<i32>());
        fs::create_dir(dir).unwrap();
        fs::File::create(format!("{}/{}", dir, file)).unwrap();

        let files = hash(dir).unwrap();

        let hash = "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e".to_string();
        assert_eq!(files, vec![
            File{path: format!("/{}", file), hash: hash}
        ]);

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn hashes_empty_directory() {
        let mut rng = rand::thread_rng();
        let dir = &format!("test.{}", rng.gen::<i32>());
        fs::create_dir(dir).unwrap();

        let files = hash(dir).unwrap();

        assert_eq!(files, vec![]);

        fs::remove_dir_all(dir).unwrap();
    }
}
