#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unknown_lints)]

//For File Struct
use std::fs;
use std::fs::{read_dir, metadata};

//For read to string
use std::io::prelude::*;

//For exists
use std::path::Path;

//For get current dir
use std::env;

use std::time::{SystemTime, Duration};
use std::io::Result;

pub struct File {
    pub path: String,
}
impl File {
    pub fn new(path: String) -> Self {
        File {
            path,
        }
    }
    
    ///Returns content of file
    pub fn read(&self) -> String {
        let mut file = fs::File::open(&self.path).expect("File could not be opened");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Import failed");
        return contents;
    }

    ///Overwrites file with content
    pub fn write(&self, content: String) {
        let mut file = fs::File::create(&self.path).expect("File could not be created");
        file.write_all(content.as_bytes()).expect("File could not be described");
        drop(file);
    }

    pub fn file_exists(&self) -> bool {
        let path = Path::new(&self.path);
        path.exists() && path.is_file()
    }


    pub fn dir_exists(&self) -> bool {
        let path = Path::new(&self.path);
        path.exists() && path.is_dir()
    }

    pub fn file_modify_time(&self) -> Option<u64> {
        let path = Path::new(&self.path);
        if let Ok(fs) = metadata(path) {
            if let Ok(time) = fs.modified() {
                if let Ok(now) = SystemTime::now().duration_since(time) {
                    return Some(now.as_secs());
                }
            }
        }
        return None;
    }
}


pub fn cur_dir() -> String {
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

//#[cfg(test)]
//mod tests {
//    use super::*;
//    #[test]
//    fn test_write_read() {
//        self::write_file(String::from("E:\\Rust\\test\\file_for_test\\ForTestRead.txt"), String::from("Hello World!"));
//        let a:String = self::read_file(String::from("E:\\Rust\\test\\file_for_test\\ForTestRead.txt"));
//        assert_eq!(a, "Hello World!");
//    }
//    #[test]
//    fn test_file_exists() {
//        assert_eq!(file_exists(&String::from("E:\\Rust\\test\\file_for_test\\ForTestRead.txt")), true);
//    }
//    #[test]
//    fn test_file_not_exists() {
//        assert_eq!(file_exists(&String::from("E:\\Rust\\test\\file_for_test\\not_exists.txt")), false);
//    }
//
//    #[test]
//    fn test_dir_exists() {
//        assert_eq!(dir_exists(&String::from("E:\\Rust\\test\\file_for_test")), true);
//    }
//
//    #[test]
//    fn test_dir_not_exists() {
//        assert_eq!(dir_exists(&String::from("E:\\Rust\\test\\not_exists")), false);
//    }
//
//    #[test]
//    fn test_cur_dir() {
//        let a:String = cur_dir();
//        assert_eq!(a, "E:\\Rust\\test");
//    }
//}