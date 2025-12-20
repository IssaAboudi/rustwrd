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

fn editor_scroll(terminal: &mut Terminal) {
    if terminal.curs_y < terminal.v_offset {
        terminal.v_offset = terminal.curs_y;
    }
    if terminal.curs_y >= terminal.v_offset + terminal.screen_rows {
        terminal.v_offset = terminal.curs_y - terminal.screen_rows + 1;
    }
}

// write out
pub(crate) fn editor_refresh_screen(terminal: &mut Terminal) -> io::Result<()> {
    let mut append_buf: Vec<u8> = Vec::new();

    //add escape sequences to buffer and build up a batch of stuff to do
    // as opposed to small writes
    append_buf.extend(b"\x1b[?25l"); //hide cursor
    append_buf.extend(b"\x1b[H"); //move cursor

    editor_draw_rows(terminal, &mut append_buf)?;

    let curs_x = terminal.curs_x + 1;
    let curs_y = terminal.curs_y + 1;

    let buf = format!("\x1b[{};{}H", (curs_y - terminal.v_offset), curs_x); //move cursor
    append_buf.extend(buf.as_bytes());

    editor_scroll(terminal); //adjust scroll offset

    append_buf.extend(b"\x1b[?25h"); //show cursor

    //write out everything in buffer
    let _status = stdout().write_all(&append_buf);
    stdout().flush()?;
    Ok(())
}

fn display_credits(terminal: &Terminal, ab: &mut Vec<u8>) {
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
    let padding = (terminal.screen_cols - author.len() as i32) / 2;
    if padding > 0 {
        ab.extend(PRFX!());
        let spaces = " ".repeat(padding as usize);
        ab.extend(spaces.as_bytes());
    }
    ab.extend(author.as_bytes());
}

pub(crate) fn editor_draw_rows(terminal: &Terminal, ab: &mut Vec<u8>) -> io::Result<()> {
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
            let has_content = terminal.content.iter().any(|line| !line.trim().is_empty());
            //add welcome message in the bottom 1/3 of the window
            if i == (terminal.screen_rows / 3 + 10) && !has_content {
                if !has_content {
                    display_credits(terminal, ab);
                } else {
                    ab.extend(b"\r\n"); //adds another space - don't remove this
                }
            } else if i == terminal.screen_rows as i32 {
                ab.extend(terminal.term_mode.to_string().as_bytes());
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
