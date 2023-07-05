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
use std::io::{stdin, stdout, Write};
use std::{env, io};

// entry point
fn main() -> io::Result<()> {
    let args: Vec<_> = env::args().collect();

    let mut terminal = Terminal {
        orig_termios: termios::tcgetattr(STDIN_FILENO)?,
        screen_rows: 0,
        screen_cols: 0,
        curs_x: 0,
        curs_y: 0,
        num_rows: 0,
        v_offset: 0,
        h_offset: 0,
        content: Vec::new(),
    };

    terminal.enableRawMode()?;
    terminal.initEditor()?;
    if args.len() >= 2 {
        terminal.editorOpenFile(&args[1])?;
    }

    loop {
        editorRefreshScreen(&mut terminal)?;
        match editorProcessKeypress(&mut terminal) {
            Ok(exit) => {
                if exit == true {
                    let _status = stdout().write_all(b"\x1b[2J")?;
                    let _status = stdout().write_all(b"\x1b[H")?;
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
