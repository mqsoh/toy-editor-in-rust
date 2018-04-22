extern crate termios;

use std::env;
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
        ANSI::move_cursor(self.cursor.row, self.cursor.col);
        ANSI::flush();
    }

    fn handle_input(&mut self) -> Command {
        let mut buf: [u8; 1] = [0; 1];

        self.io.stdin.read_exact(&mut buf).unwrap();

        match buf[0] {
            // Ctrl-q
            17 => Command::Quit,
            // up: Ctrl-k
            11 => {
                self.cursor = self.cursor.up(&self.buffer);
                Command::NoOp
            },
            // down: Ctrl-j
            10 => {
                self.cursor = self.cursor.down(&self.buffer);
                Command::NoOp
            },
            // left: Ctrl-h
            8 => {
                self.cursor = self.cursor.left(&self.buffer);
                Command::NoOp
            },
            // right: Ctrl-l
            12 => {
                self.cursor = self.cursor.right(&self.buffer);
                Command::NoOp
            },
            // carriage return
            13 => {
                self.buffer = self.buffer.split_line(self.cursor.row, self.cursor.col);
                self.cursor = self.cursor.down(&self.buffer).move_to_col(0);
                Command::NoOp
            },
            // backspace
            127 => {
                if self.cursor.col > 0 {
                    self.buffer = self.buffer.delete(self.cursor.row, self.cursor.col - 1);
                    self.cursor = self.cursor.left(&self.buffer);
                }
                Command::NoOp
            },
            any => {
                self.buffer = self.buffer.insert(any, self.cursor.row, self.cursor.col);
                self.cursor = self.cursor.right(&self.buffer);
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

    fn num_lines(&self) -> u8 {
        self.lines.len() as u8
    }

    fn num_chars(&self, row: u8) -> u8 {
        if row as usize >= self.lines.len() {
            0
        } else {
            self.lines[row as usize].len() as u8
        }
    }

    fn _clone_lines(&self) -> Vec<String> {
        let mut lines: Vec<String> = Vec::new();
        for line in self.lines.iter() {
            lines.push(line.clone())
        };
        lines
    }

    fn insert(&self, character: u8, row: u8, col: u8) -> Buffer {
        let mut lines = self._clone_lines();
        lines[row as usize].insert(col as usize, char::from(character));
        Buffer {
            lines: lines
        }
    }

    fn delete(&self, row: u8, col: u8) -> Buffer {
        let mut lines = self._clone_lines();
        lines[row as usize].remove(col as usize);
        Buffer {
            lines: lines,
        }
    }

    fn split_line(&self, row: u8, col: u8) -> Buffer {
        let mut lines: Vec<String> = Vec::new();
        for (i, line) in self.lines.iter().enumerate() {
            if i == row as usize {
                let (a, b) = line.split_at(col as usize);
                lines.push(String::from(a));
                lines.push(String::from(b));
            } else {
                lines.push(line.clone());
            }
        }
        Buffer {
            lines: lines,
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

    fn up(&self, buffer: &Buffer) -> Cursor {
        Cursor {
            row: match self.row.checked_sub(1) {
                Some(v) => v,
                None => 0,
            },
            ..*self
        }
    }

    fn down(&self, buffer: &Buffer) -> Cursor {
        Cursor {
            row: self.row+1,
            ..*self
        }.clamp(buffer)
    }

    fn left(&self, buffer: &Buffer) -> Cursor {
        Cursor {
            col: match self.col.checked_sub(1) {
                Some(v) => v,
                None => 0,
            },
            ..*self
        }.clamp(buffer)
    }

    fn right(&self, buffer: &Buffer) -> Cursor {
        Cursor {
            col: self.col + 1,
            ..*self
        }.clamp(buffer)
    }

    fn clamp(&self, buffer: &Buffer) -> Cursor {
        // I don't need to check for < 0 because of the use of `checked_sub` in
        // the directional methods.
        let row: u8 = if self.row >= buffer.num_lines() {
            buffer.num_lines() - 1
        } else {
            self.row
        };
        let col: u8 = if self.col > buffer.num_chars(row) {
            buffer.num_chars(self.row)
        } else {
            self.col
        };
        Cursor {
            row: row,
            col: col,
        }
    }

    fn move_to_col(&self, col: u8) -> Cursor {
        Cursor {
            col: col,
            ..*self
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

    fn move_cursor_col(col: u8) {
        stdout().write_fmt(format_args!("[{}G", col + 1));
    }

    fn flush() {
        stdout().flush().unwrap();
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 && args[1] == "key-inspector" {
        key_inspector();
    } else {
        Editor::new().run();
    }
}

fn key_inspector() {
    let mut io = EditorIO::new();
    loop {
        let mut buf: [u8; 1] = [0; 1];
        io.stdin.read_exact(&mut buf).unwrap();
        println!("Key read as: {:?}", buf);
        ANSI::move_cursor_col(0);
        ANSI::flush();
        if buf[0] == 17 {
            println!("Quitting... (You hit Ctrl-q.)");
            break
        }
    }
    io.restore();
}
