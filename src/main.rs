mod input;
mod output;
mod utils;
use crate::input::{editor_process_keypress, editor_read_key, EditorStates};
use crate::output::editor_refresh_screen;
use crate::utils::constants::{CLR_SCREEN, MOV_CURS_HOME};

mod terminal;

use terminal::Terminal;

use nix::libc::STDIN_FILENO;
use nix::sys::termios;
use std::io::{stdin, stdout, Read, Write};
use std::{env, io};

#[allow(dead_code)]
fn keycodes() -> io::Result<bool> {
    let mut c: char;
    //loop through all input bytes
    for byte in stdin().bytes() {
        let b = byte?;
        c = b as char;
        if c == 'q' {
            //q exits the program
            return Ok(false);
        } else if c.is_ascii_control() {
            //^ + letter gives the number of that letter
            println!("{}\r\n", b);
        } else {
            //otherwise just display the character then it's ascii value
            println!("[`{}`]: , {}\r\n", c, b);
        }
    }
    Ok(true)
}

// entry point
fn main() -> io::Result<()> {
    let args: Vec<_> = env::args().collect();

    let mut terminal = Terminal {
        orig_termios: termios::tcgetattr(STDIN_FILENO)?,
        screen_rows: 0,
        screen_cols: 0,
        snip_start: 0,
        curs_x: 0,
        curs_y: 0,
        v_offset: 0,
        fp: String::new(),
        content: Vec::new(),
    };

    terminal.enable_raw_mode()?;
    terminal.init_editor()?;
    if args.len() >= 2 {
        terminal.editor_open_file(&args[1])?;
    }

    // loop {
    //     match keycodes() {
    //         Ok(false) => break,
    //         Ok(true) => continue,
    //         Err(_e) => return Err(_e),
    //     }
    // }

    loop {
        editor_refresh_screen(&mut terminal)?;
        match editor_process_keypress(&mut terminal) {
            Ok(exit) => {
                if exit == EditorStates::Exit {
                    stdout().write_all(CLR_SCREEN)?;
                    stdout().write_all(MOV_CURS_HOME)?;
                    break;
                } else { /* continue execution */
                }
            }
            Err(_e) => {
                editor_refresh_screen(&mut terminal)?;
            }
        }
    }

    Ok(())
}
