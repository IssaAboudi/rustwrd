use core::fmt;
use std::io;
use std::io::ErrorKind::Other;
use std::io::{stdin, Error, Read};

use crate::Arabizi::arabizi_process_input;
use crate::Terminal;

// Macro to add CTRL modifier to each key
macro_rules! CTRL_KEY {
    ($k : expr) => {
        $k & 0x1f
    };
}

// editorKey bindings:
macro_rules! ARROW_UP {
    () => {
        1000
    };
}
macro_rules! ARROW_DOWN {
    () => {
        1001
    };
}
macro_rules! ARROW_LEFT {
    () => {
        1002
    };
}
macro_rules! ARROW_RIGHT {
    () => {
        1003
    };
}
macro_rules! PAGE_UP {
    () => {
        1004
    };
}
macro_rules! PAGE_DOWN {
    () => {
        1005
    };
}
macro_rules! HOME_KEY {
    () => {
        1006
    };
}
macro_rules! END_KEY {
    () => {
        1007
    };
}
macro_rules! DEL_KEY {
    () => {
        1008
    };
}
macro_rules! ENTER_KEY {
    () => {
        1009
    };
}
macro_rules! BACKSPACE_KEY {
    () => {
        1010
    };
}
macro_rules! ESCAPE_KEY {
    () => {
        27
    };
}
macro_rules! SPACE_KEY {
    () => {
        32
    };
}

// editor states
#[derive(PartialEq)]
pub enum EditorStates {
    Exit,                   // quit the program
    Continue,               // continue execution
    Save,                   // save file
    ChangeMode(EditorMode), // change editor mode
}

#[derive(PartialEq)]
pub enum EditorMode {
    Normal,        // simple text editing in English
    Arabizi,       // detect english/arabic text
    KeyCodeReader, // test Keycodes
}

impl fmt::Display for EditorMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditorMode::Normal => write!(f, "Mode: Normal"),
            EditorMode::Arabizi => write!(f, "Mode: Arabizi"),
            EditorMode::KeyCodeReader => write!(f, "Mode: Keycode"),
        }
    }
}

pub(crate) fn editor_check_keycodes() -> io::Result<EditorStates> {
    let mut c: char;
    //loop through all input bytes
    for byte in stdin().bytes() {
        let b = byte?;
        c = b as char;
        if c == 'q' {
            //q exits the program
            return Ok(EditorStates::ChangeMode(EditorMode::Normal));
        } else if c.is_ascii_control() {
            //^ + letter gives the number of that letter
            println!("{}\r\n", b);
        } else {
            //otherwise just display the character then it's ascii value
            println!("[`{}`]: , {}\r\n", c, b);
        }
    }
    Ok(EditorStates::Continue)
}

pub(crate) fn editor_process_keypress(terminal: &mut Terminal) -> io::Result<EditorStates> {
    let mut input_buf = String::new();
    match editor_read_key(&mut input_buf) {
        Ok(key_pressed) => {
            // key combos
            if key_pressed == CTRL_KEY!(b'q') as i32 {
                Ok(EditorStates::Exit) //exit the program
            } else if key_pressed == CTRL_KEY!(b'u') as i32 {
                // clear line
                terminal.content[terminal.curs_y as usize] = String::new();
                terminal.curs_x = 0;
                terminal.snip_start = 0;
                Ok(EditorStates::Continue)
            } else if key_pressed == CTRL_KEY!(b's') as i32 {
                // save file
                let fp = terminal.fp.clone();
                terminal.editor_write_file(fp)?;
                Ok(EditorStates::Save)
            } else if key_pressed == CTRL_KEY!(b'n') as i32 {
                // set normal mode
                Ok(EditorStates::ChangeMode(EditorMode::Normal))
            } else if key_pressed == CTRL_KEY!(b'a') as i32 {
                // set arabizi mode
                Ok(EditorStates::ChangeMode(EditorMode::Arabizi))
            } else if key_pressed == CTRL_KEY!(b'k') as i32 {
                Ok(EditorStates::ChangeMode(EditorMode::KeyCodeReader))
            } else {
                // all other keys
                match key_pressed {
                    HOME_KEY!() => {
                        terminal.curs_x = 0;
                    }
                    END_KEY!() => {
                        let invalid_string = String::from("");
                        let curr_row = terminal
                            .content
                            .get(terminal.curs_y as usize)
                            .unwrap_or(&invalid_string);
                        terminal.curs_x = curr_row.len() as i32;
                    }
                    PAGE_UP!() => {
                        let mut times = terminal.screen_rows;
                        while times > 0 {
                            match editor_move_cursor(terminal, ARROW_UP!()) {
                                Ok(_t) => {
                                    times -= 1;
                                }
                                Err(e) => return Err(Error::new(Other, e)),
                            };
                        }
                    }
                    PAGE_DOWN!() => {
                        let mut times = terminal.screen_rows;
                        while times > 0 {
                            match editor_move_cursor(terminal, ARROW_DOWN!()) {
                                Ok(_t) => {
                                    times -= 1;
                                }
                                Err(e) => return Err(Error::new(Other, e)),
                            };
                        }
                    }
                    ENTER_KEY!() => {
                        terminal.content.push(String::new());
                        terminal.curs_y += 1;

                        let invalid_string = String::from("");
                        let curr_row = terminal
                            .content
                            .get(terminal.curs_y as usize)
                            .unwrap_or(&invalid_string);

                        if terminal.curs_x >= curr_row.len() as i32 {
                            //if we exceed the boundary for our new row,
                            // snap back to last character in the row
                            terminal.curs_x = curr_row.len() as i32;
                        }
                    }
                    //trigger cursor movement
                    ARROW_UP!() => {
                        return match editor_move_cursor(terminal, key_pressed) {
                            Ok(_t) => Ok(EditorStates::Continue),
                            Err(e) => Err(Error::new(Other, e)),
                        };
                    }
                    ARROW_DOWN!() => {
                        return match editor_move_cursor(terminal, key_pressed) {
                            Ok(_t) => Ok(EditorStates::Continue),
                            Err(e) => Err(Error::new(Other, e)),
                        };
                    }
                    ARROW_LEFT!() => {
                        terminal.snip_start = terminal.curs_x;
                        return match editor_move_cursor(terminal, key_pressed) {
                            Ok(_t) => Ok(EditorStates::Continue),
                            Err(e) => Err(Error::new(Other, e)),
                        };
                    }
                    ARROW_RIGHT!() => {
                        terminal.snip_start = terminal.curs_x;
                        return match editor_move_cursor(terminal, key_pressed) {
                            Ok(_t) => Ok(EditorStates::Continue),
                            Err(e) => Err(Error::new(Other, e)),
                        };
                    }
                    BACKSPACE_KEY!() => {
                        if terminal.curs_x > 0 {
                            // constrain backspace to beginning of line
                            let row = &mut terminal.content[terminal.curs_y as usize];
                            if let Some((byte_idx, ch)) =
                                row.char_indices().nth(terminal.curs_x as usize - 1)
                            {
                                let char_len = ch.len_utf8();
                                row.drain(byte_idx..byte_idx + char_len);

                                terminal.curs_x -= 1;
                                if terminal.curs_x < terminal.snip_start {
                                    terminal.snip_start = terminal.curs_x;
                                }
                            }
                        } else if terminal.curs_x == 0 && terminal.curs_y > 0 {
                            let current_row = terminal.content.remove(terminal.curs_y as usize);
                            terminal.curs_y -= 1;

                            let prev_row = &mut terminal.content[terminal.curs_y as usize];
                            terminal.curs_x = prev_row.chars().count() as i32;
                            prev_row.push_str(&current_row);
                        }
                    }
                    ESCAPE_KEY!() => { /* do nothing */ }
                    SPACE_KEY!() => {
                        if terminal.term_mode == EditorMode::Arabizi {
                            arabizi_process_input(terminal);
                        }
                        // append space to the buffer
                        if let Some(ch) = input_buf.chars().next() {
                            let row = &mut terminal.content[terminal.curs_y as usize];
                            let byte_idx = row
                                .char_indices()
                                .nth(terminal.curs_x as usize)
                                .map_or(row.len(), |(i, _)| i);
                            terminal.content[terminal.curs_y as usize]
                                .insert(byte_idx as usize, ch);
                            terminal.curs_x += 1;
                            terminal.snip_start = terminal.curs_x;
                        }
                    }
                    //default typing behavior
                    _ => {
                        if let Some(ch) = input_buf.chars().next() {
                            let row = &mut terminal.content[terminal.curs_y as usize];
                            let byte_idx = row
                                .char_indices()
                                .nth(terminal.curs_x as usize)
                                .map_or(row.len(), |(i, _)| i);
                            terminal.content[terminal.curs_y as usize]
                                .insert(byte_idx as usize, ch);
                            terminal.curs_x += 1;
                        }
                    }
                }
                Ok(EditorStates::Continue)
            }
        }
        Err(_e) => Err(Error::new(Other, "failed at editorReadKey")),
    }
}

pub(crate) fn editor_read_key(buf: &mut String) -> io::Result<i32> {
    // lets read up to 4 bytes
    let mut read = [0u8; 4];
    loop {
        match stdin().read(&mut read) {
            Ok(t) => {
                match std::str::from_utf8(&read[0..t]) {
                    Ok(s) => {
                        // add all utf8 to buf
                        for ch in s.chars() {
                            buf.push(ch);
                        }
                    }
                    Err(_e) => buf.push('\u{FFD}'),
                }
                // get all bytes we read in
                break;
            }
            Err(e) => return Err(Error::new(Other, e)),
        }
    }

    // check if escape sequence
    if read[0] == b'\x1b' {
        // escape sequence is followed by a [
        if read[1] == '[' as u8 {
            // special keys are 0-9 with a ~ after
            if read[2] >= b'0' && read[2] <= b'9' {
                if read[3] == b'~' {
                    return match read[2] {
                        b'1' => Ok(HOME_KEY!()),
                        b'3' => Ok(DEL_KEY!()),
                        b'4' => Ok(END_KEY!()),
                        b'5' => Ok(PAGE_UP!()),
                        b'6' => Ok(PAGE_DOWN!()),
                        b'7' => Ok(HOME_KEY!()),
                        b'8' => Ok(END_KEY!()),
                        _ => Ok(b'\x1b' as i32),
                    };
                }
            } else {
                // arrow keys are A-D
                return match read[2] {
                    b'A' => Ok(ARROW_UP!()),
                    b'B' => Ok(ARROW_DOWN!()),
                    b'C' => Ok(ARROW_RIGHT!()),
                    b'D' => Ok(ARROW_LEFT!()),
                    b'H' => Ok(HOME_KEY!()),
                    b'F' => Ok(END_KEY!()),
                    _ => Ok(b'\x1b' as i32),
                };
            }
        } else if read[1] == b'0' {
            // more bindings for compatibility reasons
            return match read[2] {
                b'H' => Ok(HOME_KEY!()),
                b'F' => Ok(END_KEY!()),
                _ => Ok(b'\x1b' as i32),
            };
        }
        Ok(b'\x1b' as i32)
    } else if read[0] > 127 {
        // non ascii characters are flagged as -1
        // 0-127 is printable ascii characters and over is unicode
        // process keypress will still print the arabic no problem
        Ok(-1)
    } else {
        // regular key presses
        match read[0] {
            13 => Ok(ENTER_KEY!()),
            127 => Ok(BACKSPACE_KEY!()),
            27 => Ok(ESCAPE_KEY!()),
            _ => Ok(read[0] as i32),
        }
    }
}

pub(crate) fn editor_move_cursor(terminal: &mut Terminal, key: i32) -> io::Result<()> {
    let invalid_string = String::from("");
    let mut curr_row = terminal
        .content
        .get(terminal.curs_y as usize)
        .unwrap_or(&invalid_string);

    //movement with bounds checking
    //left is 0
    //top is 0
    match key {
        ARROW_LEFT!() => {
            if terminal.curs_x > 0 {
                //bounds checking
                terminal.curs_x -= 1; // - means move left
                terminal.snip_start = terminal.curs_x;
            }
            // handle pressing left at start of line
            if terminal.curs_x == 0 && terminal.curs_y > 0 {
                //move cursor up 1 row
                terminal.curs_y -= 1;
                //recalculate the current row's length
                curr_row = terminal
                    .content
                    .get(terminal.curs_y as usize)
                    .unwrap_or(&invalid_string);
                //bring us to last character in previous row
                terminal.curs_x = curr_row.len() as i32;
                terminal.snip_start = terminal.curs_x;
            }
        }
        ARROW_RIGHT!() => {
            if terminal.curs_x < curr_row.len() as i32 {
                //bounds checking
                terminal.curs_x += 1; // + means move right
                terminal.snip_start = terminal.curs_x;
            }
            //handle pressing right at end of line
            if terminal.curs_x == curr_row.len() as i32
                && terminal.curs_y < terminal.content.len() as i32 - 1
            {
                //move cursor down 1 row
                terminal.curs_y += 1;
                //recalculate the current row's length
                curr_row = terminal
                    .content
                    .get(terminal.curs_y as usize)
                    .unwrap_or(&invalid_string);
                //bring us to first character in next row
                terminal.curs_x = 0;
                terminal.snip_start = terminal.curs_x;
            }
        }
        ARROW_UP!() => {
            if terminal.curs_y > 0 {
                // bounds checking
                terminal.curs_y -= 1; // - means move up

                //recalculate the current row's length
                curr_row = terminal
                    .content
                    .get(terminal.curs_y as usize)
                    .unwrap_or(&invalid_string);
                if terminal.curs_x >= curr_row.len() as i32 {
                    //if we exceed the boundary for our new row,
                    // snap back to last character in the row
                    terminal.curs_x = curr_row.len() as i32;
                    terminal.snip_start = terminal.curs_x;
                }
            }
        }
        ARROW_DOWN!() => {
            if terminal.curs_y < terminal.content.len() as i32 - 1 {
                //bounds checking
                terminal.curs_y += 1; // + means move down

                //recalculate the current row's length
                curr_row = terminal
                    .content
                    .get(terminal.curs_y as usize)
                    .unwrap_or(&invalid_string);
                if terminal.curs_x >= curr_row.len() as i32 {
                    //if we exceed the boundary for our new row,
                    // snap back to last character in the row
                    terminal.curs_x = curr_row.len() as i32;
                    terminal.snip_start = terminal.curs_x;
                }
            }
        }

        //to catch any keys that slip past
        _ => return Err(Error::new(Other, "Invalid key in editorMoveCursor")),
    }
    Ok(())
}
