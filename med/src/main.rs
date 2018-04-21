extern crate termios;

use std::fs::File;
use std::io::{
    BufRead,
    BufReader,
    Read,
    Stdin,
    stdin,
    stdout,
    Write,
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
    io: EditorIO,
    buffer: Buffer,
    cursor: Cursor,
}

impl Editor {
    fn new() -> Editor {
        let lines = match File::open("foo.txt") {
            Ok(f) => {
                BufReader::new(f).lines().map(|line| line.unwrap()).collect()
            },
            _ => panic!("failed to open file"),
        };

        Editor{
            io: EditorIO::new(),
            buffer: Buffer::new(lines),
            cursor: Cursor::new(),
        }
    }

    fn run(&mut self) {
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
        // clear screen
        ANSI::clear_screen();
        // move cursor
        ANSI::move_cursor(0, 0);
        self.buffer.render();
        ANSI::move_cursor(0, 0);
        ANSI::flush();
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
    lines: Vec<String>,
}

impl Buffer {
    fn new(lines: Vec<String>) -> Buffer {
        Buffer{
            lines: lines,
        }
    }

    fn render(&self) {
        for line in self.lines.iter() {
            println!("{}\r", line);
        }
    }
}

struct Cursor {
    pub row: u8,
    pub col: u8,
}

impl Cursor {
    fn new() -> Cursor {
        Cursor{
            row: 1,
            col: 1,
        }
    }
}

struct ANSI {}

impl ANSI {
    fn clear_screen() {
        stdout().write(b"[2J");
    }

    fn move_cursor(row: u8, col: u8) {
        stdout().write_fmt(format_args!("[{};{}H", row + 1, col + 1));
    }

    fn flush() {
        stdout().flush().unwrap();
    }
}

fn main() {
    println!("Hello, world!");
    Editor::new().run();
}
