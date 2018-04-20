extern crate termion;

use std::{thread, time};
use std::fs::File;
use std::io::{
    self,
    BufRead,
    BufReader,
    Read,
    stdin,
    stdout,
    Stdout,
    Write,
};
use std::process::exit;

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::{AlternateScreen, ToAlternateScreen, ToMainScreen};

struct Editor {
    lines: Vec<String>,
    screen: AlternateScreen<RawTerminal<Stdout>>,
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
            screen: AlternateScreen::from(stdout().into_raw_mode().unwrap()),
        }
    }

    fn run(&mut self) {
        write!(self.screen, "{}", ToAlternateScreen).unwrap();

        loop {
            self.render();
            match self.handle_input() {
                true => break,
                _ => {},
            }
        }

        write!(self.screen, "{}", ToMainScreen).unwrap();
    }

    fn render(&self) {
    }

    fn handle_input(&mut self) -> bool {
        //let line = {
        //    let mut b = String::new();
        //    match io::stdin().read_line(&mut b) {
        //        Ok(_) => b,
        //        _ => panic!("TODO failed getting user input. Is that scary?"),
        //    }
        //};

        //match line.as_ref() {
        //    "q\n" => exit(0),
        //    any => println!("{:?} isn't a command. Hit q to quit.", any),
        //}
        match stdin().keys().next().unwrap() {
            Ok(Key::Ctrl('q')) => {
                write!(self.screen, "Quitting...\n").unwrap();
                self.screen.flush().unwrap();
                true
            },
            Ok(any_key) => {
                write!(self.screen, "I don't know what to do with {:?}\n", any_key).unwrap();
                self.screen.flush().unwrap();
                false
            },
            Err(e) => panic!("{:?}", e),
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
