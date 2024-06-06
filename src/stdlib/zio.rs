use std::fs;
use std::path::PathBuf;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

pub fn read_file(fname: PathBuf) -> Option<String> {
    match fs::canonicalize(&fname) {
        Ok(fname) => {
            match File::open(fname) {
                Ok(file) => {
                    let mut buf_reader = BufReader::new(file);
                    let mut contents = String::new();
                    match buf_reader.read_to_string(&mut contents) {
                        Ok(_) => {
                            return Some(contents.clone());
                        }
                        Err(_) => return None,
                    }
                }
                Err(_) => return None,
            }
        }
        Err(_) => return None,
    }
}
