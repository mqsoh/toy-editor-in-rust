use std::fs::File;
use std::io::{BufRead, BufReader};

struct Editor {
    pub lines: Vec<String>,
}

impl Editor {
    fn new() -> Editor {
        Editor{
            lines: match File::open("foo.txt") {
                Ok(f) => {
                    BufReader::new(f).lines().map(|line| line.unwrap()).collect()
                },
                _ => panic!("failed to open file"),
            },
        }
    }
}

fn main() {
    println!("Hello, world!");
    println!("{:?}", Editor::new().lines);
}
