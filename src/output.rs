use crate::Terminal;

use std::io;
use std::io::{stdin, stdout, Write};

//Version of our editor
macro_rules! RUST_WRD {
    () => {
        "0.0.1"
    };
}
//Line Prefix - what's at the beginning of every line in the editor
macro_rules! PRFX {
    () => {
        b"."
    };
}

fn editorScroll(terminal: &mut Terminal) {
    if terminal.curs_y < terminal.v_offset {
        terminal.v_offset = terminal.curs_y;
    }
    if terminal.curs_y >= terminal.v_offset + terminal.screen_rows {
        terminal.v_offset = terminal.curs_y - terminal.screen_rows + 1;
    }
}

// write out
pub(crate) fn editorRefreshScreen(terminal: &mut Terminal) -> io::Result<()> {
    let mut appendBuf: Vec<u8> = Vec::new();

    //add escape sequences to buffer and build up a batch of stuff to do
    // as opposed to small writes
    appendBuf.extend(b"\x1b[?25l"); //hide cursor
    appendBuf.extend(b"\x1b[H"); //move cursor

    editorDrawRows(terminal, &mut appendBuf)?;

    let curs_x = terminal.curs_x + 1;
    let curs_y = terminal.curs_y + 1;

    let buf = format!("\x1b[{};{}H", (curs_y - terminal.v_offset), curs_x); //move cursor
    appendBuf.extend(buf.as_bytes());

    editorScroll(terminal); //adjust scroll offset

    appendBuf.extend(b"\x1b[?25h"); //show cursor

    //write out everything in buffer
    let _status = stdout().write_all(&appendBuf);
    stdout().flush()?;
    Ok(())
}

fn displayCredits(terminal: &Terminal, ab: &mut Vec<u8>) {
    let welcome = "Rust Wrd -- Version ";
    let author = "by Issa Aboudi 2023";

    //center welcome message
    let padding = (terminal.screen_cols - welcome.len() as i32) / 2;
    if padding > 0 {
        ab.extend(PRFX!());
        let spaces = " ".repeat(padding as usize);
        ab.extend(spaces.as_bytes());
    }
    //Write welcome text and version number
    ab.extend(welcome.as_bytes());
    ab.extend(RUST_WRD!().as_bytes());
    ab.extend(b"\r\n");

    //do it again for author
    let padding = ((terminal.screen_cols - author.len() as i32) / 2) + 3;
    if padding > 0 {
        ab.extend(PRFX!());
        let spaces = " ".repeat(padding as usize);
        ab.extend(spaces.as_bytes());
    }
    ab.extend(author.as_bytes());
}

pub(crate) fn editorDrawRows(terminal: &Terminal, ab: &mut Vec<u8>) -> io::Result<()> {
    let mut i = 0;
    loop {
        if i > terminal.screen_rows {
            break;
        }

        // erase content on first line - ideally would like to hide it outright.
        ab.extend(b"\x1b[K");

        //add a new line for every line but the last one
        // if i < terminal.screen_rows + 1 {
        // }
        ab.extend(b"\r\n");

        // erase content on last line
        ab.extend(b"\x1b[K");

        let file_row = i + terminal.v_offset;

        if file_row >= terminal.content.len() as i32 {
            //add welcome message in the bottom 1/3 of the window
            if i == (terminal.screen_rows / 3 + 10) && terminal.content.is_empty() {
                if terminal.content.is_empty() {
                    displayCredits(terminal, ab);
                } else {
                    ab.extend(b"\r\n"); //adds another space - don't remove this
                }
            } else {
                // write a period on every line
                ab.extend(PRFX!());
            }
        } else {
            ab.extend(terminal.content[file_row as usize].as_bytes());
        }

        i += 1;
    }
    Ok(())
}
