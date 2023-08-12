#![allow(non_snake_case)]
#![allow(unused_imports)]

mod input;
mod output;

use crate::input::{editorProcessKeypress, editorReadKey};
use crate::output::editorRefreshScreen;

mod terminal;

use terminal::Terminal;

use nix::libc::STDIN_FILENO;
use nix::sys::termios;
use std::fs::File;
use std::io::ErrorKind::Other;
use std::io::{BufRead, Read, stdin, stdout, Write};
use std::{env, io};

#[allow(dead_code)]
fn keycodes() -> io::Result<()> {
    let mut c: char;
    //loop through all input bytes
    for byte in stdin().bytes() {
        let b = byte?;
        c = b as char;
        if c == 'q' {
            //q exits the program
            break;
        } else if c.is_ascii_control() {
            //^ + letter gives the number of that letter
            println!("{}\r\n", b);
        } else {
            //otherwise just display the character then it's ascii value
            println!("[`{}`]: , {}\r\n", c, b);
        }
    }
    Ok(())
}

// entry point
fn main() -> io::Result<()> {
    let args: Vec<_> = env::args().collect();

    let mut terminal = Terminal {
        orig_termios: termios::tcgetattr(STDIN_FILENO)?,
        screen_rows: 0,
        screen_cols: 0,
        curs_x: 0,
        curs_y: 0,
        v_offset: 0,
        fp: String::new(),
        content: Vec::new(),
    };

    terminal.enableRawMode()?;
    terminal.initEditor()?;
    if args.len() >= 2 {
        terminal.editorOpenFile(&args[1])?;
    }

    // keycodes();

    loop {
        editorRefreshScreen(&mut terminal)?;
        match editorProcessKeypress(&mut terminal) {
            Ok(exit) => {
                if exit {
                    stdout().write_all(b"\x1b[2J")?;
                    stdout().write_all(b"\x1b[H")?;
                    break;
                } else {
                }
            }
            Err(_e) => {
                editorRefreshScreen(&mut terminal)?;
            }
        }
    }

    Ok(())
}
