#![allow(non_snake_case)]
#![allow(unused_imports)]

use std::ffi::c_int;
use std::fs::read;
use std::io;
use std::io::{Error, ErrorKind, Read, stdin, stdout, Write};
use std::io::ErrorKind::Other;
use std::mem::size_of;
use std::ops::Add;
use std::os::fd::AsRawFd;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use nix::errno::errno;
use nix::libc::{c_ushort, EAGAIN, exit, ioctl, ISTRIP, perror, STDIN_FILENO, STDOUT_FILENO, TIOCGWINSZ, winsize};
use nix::sys::termios;
use nix::sys::termios::SpecialCharacterIndices::{VMIN, VTIME};
use nix::unistd::acct::disable;

//Macro to add CTRL modifier to each key
macro_rules! CTRL_KEY {
    ($k : expr) => {
        $k & 0x1f
    }
}

//Version of our editor

macro_rules! RUST_WRD {
    () => {"0.0.1"}
}

struct Terminal {
    orig_termios: termios::Termios,
    screen_rows: c_int,
    screen_cols: c_int,
    curs_x: c_int,
    curs_y: c_int,
}

impl Terminal {

    fn enableRawMode(&mut self) -> io::Result<()> {
        let fd = stdin().as_raw_fd(); //file descriptor for raw stdin
        self.orig_termios = termios::tcgetattr(fd).unwrap();

        let mut raw = self.orig_termios.clone();

        raw.input_flags.remove(termios::InputFlags::IXON // disable sw control flow comamnds
            | termios::InputFlags::ICRNL
            //Other flags
            | termios::InputFlags::BRKINT //break condition causes a SIGINT signal (^c)
            | termios::InputFlags::INPCK // enables parity checking
            | termios::InputFlags::ISTRIP // 8th bit of each input byte to be stripped
        );
        raw.output_flags.remove(termios::OutputFlags::OPOST // disable '\n' -> '\r\n' translation
        );
        raw.control_flags.remove( termios::ControlFlags::CS8 // sets character size to 8 bits per byte
        );
        raw.local_flags.remove(termios::LocalFlags::ECHO //disables your input being spit back to you
            | termios::LocalFlags::ICANON //disables cannonical mode
            | termios::LocalFlags::ISIG // disables interrupts (^c, ^z)
            | termios::LocalFlags::IEXTEN // disables typing other characters literally (^v)
        );

        raw.control_chars[VMIN as usize] = 0; //return as soon as there is any input to be read
        raw.control_chars[VTIME as usize] = 1; //maximum time to wait ~ 1/10th of a second (100ms)

        termios::tcsetattr(fd, termios::SetArg::TCSAFLUSH, &raw).unwrap();
        Ok(())
    }

    fn disableRawMode(&self) -> io::Result<()> {
        let fd = stdin().as_raw_fd();
        termios::tcsetattr(fd, termios::SetArg::TCSAFLUSH, &self.orig_termios).unwrap();
        Ok(())
    }

    fn getWindowSize(&mut self, rows: &mut c_int, cols: &mut c_int) -> io::Result<()> {
        let ws: winsize = unsafe { std::mem::zeroed() };

        let result = unsafe { ioctl(STDIN_FILENO, TIOCGWINSZ, &ws) };
        if result == -1 || ws.ws_col == 0 {
            // we tell terminal to move to bottom right edge with large values
            match stdout().write_all(b"\x1b[999C\x1b[999B") {
                Ok(_c) => {}
                Err(_e) => {
                    return Err(Error::new(Other, "Error: Failed write at getWindowSize"))
                }
            }
            return self.getCursorPosition(rows, cols);
        } else {
            *rows = ws.ws_row as c_int;
            *cols = ws.ws_col as c_int;
            Ok(())
        }

    }

    fn getCursorPosition(&self, rows: &mut c_int, cols: &mut c_int) -> io::Result<()> {
        let mut buf = ['\0'; 32];

        match stdout().write_all(b"\x1b[6n") {
            Ok(_t) => {
                print!("\r\n");

                let mut i = 0;

                let mut c = [0u8;1];
                let mut nread;
                loop {
                    //loop through buffer
                    if i > buf.len() {
                        break;
                    }

                    //read input buffer
                    nread = io::stdin().read(&mut c)?;
                    if nread != 1 {
                        break;
                    }
                    buf[i] = c[0] as char;
                    if buf[i] as char == 'R' {
                        break;
                    }
                    i += 1;
                }

                //If invalid, error out
                if buf[0] != '\x1b' || buf[1] != '[' {
                    return Err(Error::new(Other, "Invalid escape sequence at getCursorPosition"));
                }

                //parse the buffer ignoring the first byte: \x1b
                let input : String = buf[2..].iter().collect();
                let parts: Vec<&str> = input.split(";").collect();
                if parts.len() == 2 {
                    let parsed_rows = match parts[0].trim().parse::<c_int>() {
                        Ok(t) => t,
                        Err(e) => {
                            return Err(Error::new(Other, "Invalid parsing: parsed_rows in getCursorPosition"));
                        }
                    };
                    let parsed_cols = match parts[0].trim().parse::<c_int>() {
                        Ok(t) => t,
                        Err(e) => {
                            return Err(Error::new(Other, "Invalid parsing: parsed_cols in getCursorPosition"));
                        }
                    };

                    *rows = parsed_rows;
                    *cols = parsed_cols;

                } else {
                    return Err(Error::new(Other, "Invalid parsing of parts in getCursorPosition"))
                }


                match editorReadKey() {
                    Ok(_t) => {}
                    Err(e) => {
                        return Err(Error::new(Other, e))
                    }
                }
            }
            Err(_e) => {
                return Err(Error::new(Other, "bad write at getCursorPosition"))
            }
        };
        Ok(())
    }

    fn initEditor(&mut self) -> io::Result<()> {
        self.curs_x = 0;
        self.curs_y = 0;

        let mut rows = self.screen_rows;
        let mut cols = self.screen_cols;
        match self.getWindowSize(&mut rows, &mut cols) {
            Ok(_c) => {
                self.screen_rows = rows;
                self.screen_cols = cols;
                Ok(())
            }
            Err(e) => {
                return Err(Error::new(Other, e))
            }
        }

    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        println!("Program Ending\r\n");
        Terminal::disableRawMode(self).unwrap();
    }
}

// get the pressed keys
fn editorReadKey() -> io::Result<u8>{
    let mut c = [0u8; 1];
    loop {
        match stdin().read(&mut c) {
            Ok(t) => {
                if t == 1 {
                    print!("{}", c[0] as char);
                    break;
                }
            }
            Err(e) => {
                return Err(Error::new(Other, e))
            }
        };
    }
    Ok(c[0])
}

// process input
fn editorProcessKeypress() -> io::Result<bool>{
    match editorReadKey() {
        Ok(c) => {
            if c == CTRL_KEY!('q' as u8){
                Ok(true)
            } else {
                Ok(false)
            }
        }
        Err(_e) => {
            Err(Error::new(Other, "failed at editorReadKey"))
        }
    }

}

// write out
fn editorRefreshScreen(terminal: &Terminal) -> io::Result<()> {

    let mut appendBuf : Vec<u8> = Vec::new();

    //add escape sequences to buffer and build up a batch of stuff to do
    // as opposed to small writes
    appendBuf.extend(b"\x1b[?25l");
    appendBuf.extend(b"\x1b[H");

    editorDrawRows(terminal, &mut appendBuf)?;
    let curs_x = terminal.curs_x + 1;
    let curs_y = terminal.curs_y + 1;

    let buf = "\x1b[".to_string() + &curs_y.to_string() + ";" + &curs_x.to_string() + "H";
    appendBuf.extend(buf.as_bytes());

    appendBuf.extend(b"\x1b[?25h");

    //write out everything in buffer
    let _status = stdout().write_all(&appendBuf);
    stdout().flush()?;
    Ok(())
}

// draw rows
fn editorDrawRows(terminal : &Terminal, ab : &mut Vec<u8>) -> io::Result<()> {
    let mut i = 0;
    loop {
        if i > terminal.screen_rows { break; }

        //add a new line for every line but the last one
        if i < terminal.screen_rows + 1  {
            ab.extend(b"\r\n");
        }

        // erases current part of the line. by default the line to the right of the cursor
        ab.extend(b"\x1b[K");

        //add welcome message in the bottom 1/3 of the window
        if i == (terminal.screen_rows / 3 + 10) {
            let welcome = "Rust Wrd -- Version ";
            let author = "by Issa Aboudi 2023";

            //center welcome message
            let padding = ( terminal.screen_cols - welcome.len() as i32 ) / 2;
            if padding > 0 {
                ab.extend(b".");
                let spaces=" ".repeat(padding as usize);
                ab.extend(spaces.as_bytes());
            }

            ab.extend(welcome.as_bytes()); //add welcome text
            ab.extend(RUST_WRD!().as_bytes()); // add version number macro
            ab.extend(b"\r\n");

            let padding = (( terminal.screen_cols - author.len() as i32) / 2 ) + 3;
            if padding > 0 {
                ab.extend(b".");
                let spaces = " ".repeat(padding as usize);
                ab.extend(spaces.as_bytes());
            }
            ab.extend(author.as_bytes());

        } else {
            // write a period on every line
            ab.extend(b".");
        }

        i += 1;
    }
    stdout().flush()?;
    Ok(())
}

fn main() -> io::Result<()> {
    let mut terminal = Terminal {
        orig_termios: termios::tcgetattr(STDIN_FILENO)?,
        screen_rows: 0,
        screen_cols: 0,
        curs_x: 10,
        curs_y: 50,
    };

    terminal.enableRawMode()?;
    terminal.initEditor()?;

    loop {
        editorRefreshScreen(&terminal)?;
        match editorProcessKeypress() {
            Ok(exit) => {
                if exit == true {
                    let _status = stdout().write_all(b"\x1b[2J")?;
                    let _status = stdout().write_all(b"\x1b[H")?;
                    break;
                } else {}
            }
            Err(_e) => {
                editorRefreshScreen(&terminal)?;
            }
        }
    }
    Ok(())
}
