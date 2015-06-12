use std::cmp;
use std::fmt;
use std::io;
use std::fs;
use openssl::crypto::hash;

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

pub fn hash(dir: &str, relative_path: &str) -> Result<File, String> {
    let absolute_path = format!("{}{}", dir, relative_path);

    let mut file = try!(fs::File::open(&absolute_path).map_err(|err|
        format!("error opening file {}: {}", absolute_path, err)
    ));
    let mut hasher = hash::Hasher::new(hash::Type::SHA512);

    try!(io::copy(&mut file, &mut hasher).map_err(|err|
        format!("error hashing file {}: {}", absolute_path, err)
    ));

    Ok(File {
        path: relative_path.to_string(),
        hash: hasher.finish()
                .iter()
                .map(|byte| format!("{:02x}", byte))
                .fold("".to_string(), |hash_str, byte_str| hash_str + &byte_str)
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use rand;
    use rand::Rng;
    use hash::hash;
    use hash::File;

    #[test]
    fn hashes_a_file() {
        let mut rng = rand::thread_rng();
        let dir = &format!("test.{}", rng.gen::<i32>());
        let file1 = "B";
        fs::create_dir(dir).unwrap();
        fs::File::create(format!("{}/{}", dir, &file1)).unwrap();

        let file = hash(dir, &format!("/{}", &file1)).unwrap();

        let hash = "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e".to_string();
        assert_eq!(file, File{path: format!("/{}", file1), hash: hash.clone()});

        fs::remove_dir_all(dir).unwrap();
    }
}
