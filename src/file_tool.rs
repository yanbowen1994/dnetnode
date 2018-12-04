//For File Struct
use std::fs::{File, read_dir};

//For read to string
use std::io::prelude::*;

//For exists
use std::path::Path;

//For get current dir
use std::env;

///Returns content of file
pub fn read_file(filename: String) -> String {
    let mut file = File::open(filename).expect("File could not be opened");

    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Import failed");

    return contents;
}

///Overwrites file with content
pub fn write_file(filename: String, content: String) {
    let mut file = File::create(filename).expect("File could not be created");
    file.write_all(content.as_bytes()).expect("File could not be described");
    drop(file);
}

pub fn file_exists(filename: String) -> bool {
    let path = Path::new(&filename);
    path.exists() && path.is_file()
}


pub fn dir_exists(filename: String) -> bool {
    let path = Path::new(&filename);
    path.exists() && path.is_dir()
}

pub fn cur_dir() -> String{
    let path = env::current_dir()
    .ok()
    .expect("Failed get current dir");
    return path.display().to_string()
}

pub fn list_dir() {
    let paths = read_dir("./").unwrap();
    for path in paths {
        println!("Name: {}", path.unwrap().path().display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_write_read() {
        self::write_file(String::from("E:\\Rust\\test\\file_for_test\\ForTestRead.txt"), String::from("Hello World!"));
        let a:String = self::read_file(String::from("E:\\Rust\\test\\file_for_test\\ForTestRead.txt"));
        assert_eq!(a, "Hello World!");
    }
    #[test]
    fn test_file_exists() {
        assert_eq!(file_exists(String::from("E:\\Rust\\test\\file_for_test\\ForTestRead.txt")), true);
    }
    #[test]
    fn test_file_not_exists() {
        assert_eq!(file_exists(String::from("E:\\Rust\\test\\file_for_test\\not_exists.txt")), false);
    }

    #[test]
    fn test_dir_exists() {
        assert_eq!(dir_exists(String::from("E:\\Rust\\test\\file_for_test")), true);
    }

    #[test]
    fn test_dir_not_exists() {
        assert_eq!(dir_exists(String::from("E:\\Rust\\test\\not_exists")), false);
    }

    #[test]
    fn test_cur_dir() {
        let a:String = cur_dir();
        assert_eq!(a, "E:\\Rust\\test");
    }
}