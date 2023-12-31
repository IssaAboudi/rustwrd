use std::io;
use std::io::ErrorKind::Other;
use std::io::{stdin, stdout, Error, ErrorKind, Read, Write};

use crate::Terminal;

//Macro to add CTRL modifier to each key
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
    () => {1009};
}
macro_rules! BACKSPACE_KEY {
    () => {
        1010
    };
}

pub(crate) fn editorProcessKeypress(terminal: &mut Terminal) -> io::Result<bool> {
    let mut input_buf = String::new();
    match editorReadKey(&mut input_buf) {
        Ok(keyPressed) => {
            if keyPressed == CTRL_KEY!(b'q') as i32 {
                Ok(true) //exit the program
            } else if keyPressed == CTRL_KEY!(b'u') as i32 {
                //clear line
                terminal.content[terminal.curs_y as usize] = String::new();
                terminal.curs_x = 0;
                Ok(false)
            } else if keyPressed == CTRL_KEY!(b's') as i32 {
                let fp = terminal.fp.clone();
                terminal.editorWriteFile(fp)?;
                Ok(false)
            }
            else {
                match keyPressed {
                    HOME_KEY!() => {
                        terminal.curs_x = 0;
                    }
                    END_KEY!() => {
                        let invalidString = String::from("");
                        let curr_row = terminal.content.get(terminal.curs_y as usize).unwrap_or(&invalidString);
                        terminal.curs_x = curr_row.len() as i32;
                    }
                    PAGE_UP!() => {
                        let mut times = terminal.screen_rows;
                        while times > 0 {
                            match editorMoveCursor(terminal,  ARROW_UP!() ) {
                                Ok(_t) => { times -= 1; }
                                Err(e) => return Err(Error::new(Other, e)),
                            };
                        }
                    }
                    PAGE_DOWN!() => {
                        let mut times = terminal.screen_rows;
                        while times > 0 {
                            match editorMoveCursor(terminal, ARROW_DOWN!() ) {
                                Ok(_t) => { times -= 1; }
                                Err(e) => return Err(Error::new(Other, e)),
                            };
                        }
                    }
                    ENTER_KEY!() => {
                        terminal.content.push(String::new());
                        terminal.curs_y += 1;

                        let invalidString = String::from("");
                        let curr_row = terminal.content.get(terminal.curs_y as usize).unwrap_or(&invalidString);

                        if terminal.curs_x >= curr_row.len() as i32 {
                            //if we exceed the boundary for our new row,
                            // snap back to last character in the row
                            terminal.curs_x = curr_row.len() as i32;
                        }

                    }
                    //trigger cursor movement
                    ARROW_UP!() => {
                        return match editorMoveCursor(terminal, keyPressed) {
                            Ok(_t) => Ok(false),
                            Err(e) => Err(Error::new(Other, e)),
                        };
                    }
                    ARROW_DOWN!() => {
                        return match editorMoveCursor(terminal, keyPressed) {
                            Ok(_t) => Ok(false),
                            Err(e) => Err(Error::new(Other, e)),
                        };
                    }
                    ARROW_LEFT!() => {
                        return match editorMoveCursor(terminal, keyPressed) {
                            Ok(_t) => Ok(false),
                            Err(e) => Err(Error::new(Other, e)),
                        };
                    }
                    ARROW_RIGHT!() => {
                        return match editorMoveCursor(terminal, keyPressed) {
                            Ok(_t) => Ok(false),
                            Err(e) => Err(Error::new(Other, e)),
                        };
                    }
                    BACKSPACE_KEY!() => {
                        if terminal.curs_x > 0 { // constrain backspace to beginning of line
                            terminal.content[terminal.curs_y as usize].remove(terminal.curs_x  as usize -1);
                            terminal.curs_x -= 1;
                        } else if terminal.curs_x == 0 && terminal.curs_y > 0 {
                            terminal.content.pop();
                            terminal.curs_y -= 1;

                            let invalidString = String::from("");
                            let curr_row = terminal.content.get(terminal.curs_y as usize).unwrap_or(&invalidString);

                            terminal.curs_x = curr_row.len() as i32;
                        }
                    }
                    //default typing behavior
                    _ => {
                        terminal.content[terminal.curs_y as usize].insert(terminal.curs_x as usize, input_buf.chars().next().unwrap());
                        terminal.curs_x += 1;
                    }
                }
                Ok(false)
            }
        }
        Err(_e) => Err(Error::new(Other, "failed at editorReadKey")),
    }
}

// process input
pub(crate) fn editorReadKey(buf: &mut String) -> io::Result<i32> {
    let mut c = [0u8; 1];
    loop {
        //read 1 byte in
        match stdin().read(&mut c) {
            Ok(t) => {
                if t == 1 {
                    //print the byte we read in
                    buf.push(c[0] as char);
                    break;
                }
            }
            Err(e) => return Err(Error::new(Other, e)),
        };
    }

    // if key pressed was escape sequence beginning
    if c[0] == b'\x1b' {
        let mut seq = [0u8; 3];
        //read the next bytes
        let _ = stdin().read(&mut seq)?;

        //if escape follows with a [
        // then it's an escape sequence
        if seq[0] == '[' as u8 {
            if seq[1] >= b'0' && seq[1] <= b'9' {
                if seq[2] == b'~' {
                    return match seq[1] {
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
                //translate arrow keys
                return match seq[1] {
                    b'A' => Ok(ARROW_UP!()),
                    b'B' => Ok(ARROW_DOWN!()),
                    b'C' => Ok(ARROW_RIGHT!()),
                    b'D' => Ok(ARROW_LEFT!()),
                    b'H' => Ok(HOME_KEY!()),
                    b'F' => Ok(END_KEY!()),
                    _ => Ok(b'\x1b' as i32),
                };
            }
        } else if seq[0] == b'O' {
            return match seq[1] {
                b'H' => Ok(HOME_KEY!()),
                b'F' => Ok(END_KEY!()),
                _ => Ok(b'\x1b' as i32),
            };
        }

        Ok(b'\x1b' as i32)
    } else {
        match c[0] {
            13 => Ok(ENTER_KEY!()),
            127 => Ok(BACKSPACE_KEY!()),
            _ => Ok(c[0] as i32)
        }
    }
}

pub(crate) fn editorMoveCursor(terminal: &mut Terminal, key: i32) -> io::Result<()> {
    let invalidString = String::from("");
    let mut curr_row = terminal.content.get(terminal.curs_y as usize).unwrap_or(&invalidString);

    //movement with bounds checking
    //left is 0
    //top is 0
    match key {
        ARROW_LEFT!() => {
            if terminal.curs_x > 0 { //bounds checking
                terminal.curs_x -= 1 // - means move left
            }
            //handle pressing left at start of line
            if terminal.curs_x == 0
                && terminal.curs_y > 0 {
                //move cursor up 1 row
                terminal.curs_y -= 1;
                //recalculate the current row's length
                curr_row = terminal.content.get(terminal.curs_y as usize).unwrap_or(&invalidString);
                //bring us to last character in previous row
                terminal.curs_x = curr_row.len() as i32;
            }
        }
        ARROW_RIGHT!() => {
            if terminal.curs_x < curr_row.len() as i32 { //bounds checking
                terminal.curs_x += 1 // + means move right
            }
            //handle pressing right at end of line
            if terminal.curs_x == curr_row.len() as i32
                && terminal.curs_y < terminal.content.len() as i32 - 1 {
                //move cursor down 1 row
                terminal.curs_y += 1;
                //recalculate the current row's length
                curr_row = terminal.content.get(terminal.curs_y as usize).unwrap_or(&invalidString);
                //bring us to first character in next row
                terminal.curs_x = 0;
            }
        }
        ARROW_UP!() => {
            if terminal.curs_y > 0 { // bounds checking
                terminal.curs_y -= 1; // - means move up

                //recalculate the current row's length
                curr_row = terminal.content.get(terminal.curs_y as usize).unwrap_or(&invalidString);
                if terminal.curs_x >= curr_row.len() as i32 {
                    //if we exceed the boundary for our new row,
                    // snap back to last character in the row
                    terminal.curs_x = curr_row.len() as i32;
                }
            }
        }
        ARROW_DOWN!() => {
            if terminal.curs_y < terminal.content.len() as i32 - 1 { //bounds checking
                terminal.curs_y += 1; // + means move down

                //recalculate the current row's length
                curr_row = terminal.content.get(terminal.curs_y as usize).unwrap_or(&invalidString);
                if terminal.curs_x >= curr_row.len() as i32 {
                    //if we exceed the boundary for our new row,
                    // snap back to last character in the row
                    terminal.curs_x = curr_row.len() as i32;
                }
            }
        }

        //to catch any keys that slip past
        _ => return Err(Error::new(Other, "Invalid key in editorMoveCursor")),
    }
    Ok(())
}
