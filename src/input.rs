use std::io;
use std::io::ErrorKind::Other;
use std::io::{stdin, stdout, Error, ErrorKind, Read};

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

// process input
pub(crate) fn editorReadKey() -> io::Result<i32> {
    let mut c = [0u8; 1];
    loop {
        //read 1 byte in
        match stdin().read(&mut c) {
            Ok(t) => {
                if t == 1 {
                    //print the byte we read in
                    print!("{}", c[0] as char);
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
        stdin().read(&mut seq)?;

        //if escape follows with a [
        // then it's an escape sequence
        if seq[0] == '[' as u8 {
            if seq[1] >= b'0' && seq[1] <= b'9' {
                //translate page up and page down
                if seq[2] == b'~' {
                    return match seq[1] {
                        b'5' => Ok(PAGE_UP!()),
                        b'6' => Ok(PAGE_DOWN!()),
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
                    _ => Ok(b'\x1b' as i32),
                };
            }
        }

        Ok(b'\x1b' as i32)
    } else {
        Ok(c[0] as i32)
    }
}

pub(crate) fn editorProcessKeypress(terminal: &mut Terminal) -> io::Result<bool> {
    match editorReadKey() {
        Ok(c) => {
            if c == CTRL_KEY!('q' as u8) as i32 {
                Ok(true)
            } else {
                //trigger page up and page down
                if c == PAGE_UP!() || c == PAGE_DOWN!() {
                    let mut times = terminal.screen_rows;
                    while times > 0 {
                        match editorMoveCursor(
                            terminal,
                            if c == PAGE_UP!() {
                                ARROW_UP!()
                            } else {
                                ARROW_DOWN!()
                            },
                        ) {
                            Ok(_t) => {
                                times -= 1;
                            }
                            Err(e) => return Err(Error::new(Other, e)),
                        };
                    }
                }

                //trigger cursor movement
                if c == ARROW_UP!()
                    || c == ARROW_DOWN!()
                    || c == ARROW_LEFT!()
                    || c == ARROW_RIGHT!()
                {
                    return match editorMoveCursor(terminal, c) {
                        Ok(_t) => Ok(false),
                        Err(e) => Err(Error::new(Other, e)),
                    };
                };
                Ok(false)
            }
        }
        Err(_e) => Err(Error::new(Other, "failed at editorReadKey")),
    }
}

pub(crate) fn editorMoveCursor(terminal: &mut Terminal, key: i32) -> io::Result<()> {
    //movement with bounds checking
    match key {
        ARROW_LEFT!() => {
            if terminal.curs_x > 0 {
                terminal.curs_x -= 1
            }
        }
        ARROW_RIGHT!() => {
            if terminal.curs_x < terminal.screen_cols - 1 {
                terminal.curs_x += 1
            }
        }
        ARROW_UP!() => {
            if terminal.curs_y > 0 {
                terminal.curs_y -= 1
            }
        }
        ARROW_DOWN!() => {
            if terminal.curs_y < terminal.screen_rows - 1 {
                terminal.curs_y += 1
            }
        }
        //to catch any keys that slip past
        _ => return Err(Error::new(Other, "Invalid key in editorMoveCursor")),
    }
    Ok(())
}
