#![allow(non_snake_case)]
#![allow(unused_imports)]

use std::ffi::c_int;
use std::fs::read;
use std::io;
use std::io::{Error, ErrorKind, Read, stdin, stdout, Write};
use std::io::ErrorKind::Other;
use std::mem::size_of;
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

struct Terminal {
    orig_termios: termios::Termios,
    screen_rows: c_int,
    screen_cols: c_int,
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
        if true || result == -1 || ws.ws_col == 0 { //TODO: remove true from condition
            match stdout().write_all(b"\x1b[999C\x1b[999B") {
                Ok(_c) => {}
                Err(_e) => {
                    return Err(Error::new(Other, "Error: Failed write at getWindowSize"))
                }
            }
            return self.getCursorPosition(rows, cols);
            // return Err(Error::new(Other, "bad ioctl at getWindowSize"))//bad ioctl
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
                    /*
                    let b = c[0] as char;
                    if b.is_ascii_control() {
                        print!("{}\r\n", c[0]);
                    } else {
                        print!("{} ({})\r\n", b, c[0]);
                    }
                    */
                }
                buf[i] = '\0';

                print!("\r\n buf[1]: {:?}\r\n", &buf[1..=i]); //TODO: extract 2 numbers (rows, cols) from buf
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
        let mut rows = self.screen_rows;
        let mut cols = self.screen_cols;
        match self.getWindowSize(&mut rows, &mut cols) {
            Ok(_c) => {
                self.screen_rows = rows;
                self.screen_cols = cols;
                Ok(())
            }
            Err(_e) => {
                return Err(Error::new(Other, "fail at getWindowSize"))
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
    let nread;
    let mut c = [0u8; 1];
    loop {
        nread = io::stdin().read(&mut c)?;
        if nread == 1 {
            print!("{}", c[0] as char);
            break;
        } else {
            return Err(Error::new(Other, "failed at read"))
        }
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
    let _status = stdout().write_all(b"\x1b[2J")?;
    let _status = stdout().write_all(b"\x1b[H")?;
    stdout().flush()?;

    editorDrawRows(terminal)?;
    let _status = stdout().write_all(b"\x1b[H")?;
    Ok(())
}

// draw rows
fn editorDrawRows(terminal : &Terminal) -> io::Result<()> {
    let mut i = 0;
    loop {
        if i > terminal.screen_rows { break; }

        stdout().write(b".\r\n")?;
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
