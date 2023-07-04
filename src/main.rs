#![allow(non_snake_case)]
#![allow(unused_imports)]

mod input;
mod output;

use crate::input::{editorProcessKeypress, editorReadKey};
use crate::output::editorRefreshScreen;

mod terminal;

use terminal::Terminal;

// use nix::errno::errno;
use nix::libc::STDIN_FILENO;
use nix::sys::termios;
use std::io;
use std::io::{stdin, stdout, Write};

// entry point
fn main() -> io::Result<()> {
    let mut terminal = Terminal {
        orig_termios: termios::tcgetattr(STDIN_FILENO)?,
        screen_rows: 0,
        screen_cols: 0,
        curs_x: 0,
        curs_y: 0,
    };

    terminal.enableRawMode()?;
    terminal.initEditor()?;

    loop {
        editorRefreshScreen(&terminal)?;
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
                editorRefreshScreen(&terminal)?;
            }
        }
    }
    Ok(())
}
