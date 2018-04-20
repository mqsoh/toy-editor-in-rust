extern crate termios;

use std::fs::File;
use std::io::{
    BufRead,
    BufReader,
    Read,
    Stdin,
    stdin,
};
use std::os::unix::io::{AsRawFd, RawFd};

use termios::{cfmakeraw, Termios, TCSANOW, tcsetattr};

enum Command {
    NoOp,
    Quit,
}

struct EditorIO {
    stdin: Stdin,
    fd: RawFd,
    original_termios: Termios,
}

impl EditorIO {
    fn new() -> EditorIO {
        let stdin = stdin();

        let stdin_fd = stdin.as_raw_fd();

        let ot = Termios::from_fd(stdin_fd).unwrap();
        let mut t = ot.clone();

        cfmakeraw(&mut t);
        tcsetattr(stdin_fd, TCSANOW, &mut t).unwrap();

        EditorIO{
            stdin: stdin,
            fd: stdin_fd,
            original_termios: ot,
        }
    }

    fn restore(&mut self) {
        tcsetattr(self.fd, TCSANOW, & self.original_termios).unwrap();
    }
}

struct Editor {
    lines: Vec<String>,
    io: EditorIO,
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
            io: EditorIO::new(),
        }
    }

    fn run(&mut self) {
        println!("[31;49mI have a butt![39;49m");

        loop {
            self.render();
            match self.handle_input() {
                Command::Quit => break,
                _ => {},
            }
        }

        self.io.restore();
    }

    fn render(&mut self) {
    }

    fn handle_input(&mut self) -> Command {
        let mut buf: [u8; 1] = [0; 1];

        self.io.stdin.read_exact(&mut buf).unwrap();

        match buf[0] {
            17 => Command::Quit,
            any => {
                println!("I don't know what {} is. Ctrl-q to quit.", any);
                Command::NoOp
            },
        }
    }
}

struct Buffer {
}

struct Cursor {
}

fn main() {
    println!("Hello, world!");
    Editor::new().run();
}
