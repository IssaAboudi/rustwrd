#![allow(non_camel_case_types)]

use crate::input::editorReadKey;

use nix::libc::{
    c_ushort, exit, ioctl, perror, winsize, EAGAIN, ISTRIP, STDIN_FILENO, STDOUT_FILENO, TIOCGWINSZ,
};
use nix::sys::termios;
use nix::sys::termios::SpecialCharacterIndices::{VMIN, VTIME};
use std::ffi::c_int;
use std::fs;
use std::io;
use std::io::ErrorKind::Other;
use std::io::{stdin, stdout, BufRead, BufReader, Error, ErrorKind, Read, Write};
use std::os::fd::AsRawFd;

use nix::sys::socket::AddressFamily::Packet;
use std::fs::{read, File};

pub(crate) struct Terminal {
    pub(crate) orig_termios: termios::Termios,
    pub(crate) screen_rows: c_int,
    pub(crate) screen_cols: c_int,
    pub(crate) curs_x: c_int,
    pub(crate) curs_y: c_int,
    pub(crate) content: Vec<String>,
    pub(crate) num_rows: i32,
    pub(crate) v_offset: i32,
    pub(crate) h_offset: i32,
}

impl Terminal {
    pub(crate) fn enableRawMode(&mut self) -> io::Result<()> {
        let fd = stdin().as_raw_fd(); //file descriptor for raw stdin
        self.orig_termios = termios::tcgetattr(fd).unwrap();

        let mut raw = self.orig_termios.clone();

        raw.input_flags.remove(
            termios::InputFlags::IXON // disable sw control flow comamnds
                | termios::InputFlags::ICRNL
                //Other flags
                | termios::InputFlags::BRKINT //break condition causes a SIGINT signal (^c)
                | termios::InputFlags::INPCK // enables parity checking
                | termios::InputFlags::ISTRIP, // 8th bit of each input byte to be stripped
        );
        raw.output_flags.remove(
            termios::OutputFlags::OPOST, // disable '\n' -> '\r\n' translation
        );
        raw.control_flags.remove(
            termios::ControlFlags::CS8, // sets character size to 8 bits per byte
        );
        raw.local_flags.remove(
            termios::LocalFlags::ECHO //disables your input being spit back to you
                | termios::LocalFlags::ICANON //disables cannonical mode
                | termios::LocalFlags::ISIG // disables interrupts (^c, ^z)
                | termios::LocalFlags::IEXTEN, // disables typing other characters literally (^v)
        );

        raw.control_chars[VMIN as usize] = 0; //return as soon as there is any input to be read
        raw.control_chars[VTIME as usize] = 1; //maximum time to wait ~ 1/10th of a second (100ms)

        termios::tcsetattr(fd, termios::SetArg::TCSAFLUSH, &raw).unwrap();
        Ok(())
    }

    pub(crate) fn disableRawMode(&self) -> io::Result<()> {
        let fd = stdin().as_raw_fd();
        termios::tcsetattr(fd, termios::SetArg::TCSAFLUSH, &self.orig_termios).unwrap();
        Ok(())
    }

    pub(crate) fn getWindowSize(&mut self, rows: &mut c_int, cols: &mut c_int) -> io::Result<()> {
        let ws: winsize = unsafe { std::mem::zeroed() };

        let result = unsafe { ioctl(STDIN_FILENO, TIOCGWINSZ, &ws) };
        if result == -1 || ws.ws_col == 0 {
            // we tell terminal to move to bottom right edge with large values
            match stdout().write_all(b"\x1b[999C\x1b[999B") {
                Ok(_c) => {}
                Err(_e) => return Err(Error::new(Other, "Error: Failed write at getWindowSize")),
            }
            return self.getCursorPosition(rows, cols);
        } else {
            *rows = ws.ws_row as c_int;
            *cols = ws.ws_col as c_int;
            Ok(())
        }
    }

    pub(crate) fn getCursorPosition(&self, rows: &mut c_int, cols: &mut c_int) -> io::Result<()> {
        let mut buf = ['\0'; 32];

        match stdout().write_all(b"\x1b[6n") {
            Ok(_t) => {
                print!("\r\n");

                let mut i = 0;

                let mut c = [0u8; 1];
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
                    return Err(Error::new(
                        Other,
                        "Invalid escape sequence at getCursorPosition",
                    ));
                }

                //parse the buffer ignoring the first byte: \x1b
                let input: String = buf[2..].iter().collect();
                let parts: Vec<&str> = input.split(";").collect();
                if parts.len() == 2 {
                    let parsed_rows = match parts[0].trim().parse::<c_int>() {
                        Ok(t) => t,
                        Err(_e) => {
                            return Err(Error::new(
                                Other,
                                "Invalid parsing: parsed_rows in getCursorPosition",
                            ));
                        }
                    };
                    let parsed_cols = match parts[0].trim().parse::<c_int>() {
                        Ok(t) => t,
                        Err(_e) => {
                            return Err(Error::new(
                                Other,
                                "Invalid parsing: parsed_cols in getCursorPosition",
                            ));
                        }
                    };

                    *rows = parsed_rows;
                    *cols = parsed_cols;
                } else {
                    return Err(Error::new(
                        Other,
                        "Invalid parsing of parts in getCursorPosition",
                    ));
                }

                match editorReadKey() {
                    Ok(_t) => {}
                    Err(e) => return Err(Error::new(Other, e)),
                }
            }
            Err(_e) => return Err(Error::new(Other, "bad write at getCursorPosition")),
        };
        Ok(())
    }

    pub(crate) fn initEditor(&mut self) -> io::Result<()> {
        self.curs_x = 0;
        self.curs_y = 0;
        self.num_rows = 0;
        self.v_offset = 0;

        let mut rows = self.screen_rows;
        let mut cols = self.screen_cols;
        match self.getWindowSize(&mut rows, &mut cols) {
            Ok(_c) => {
                self.screen_rows = rows;
                self.screen_cols = cols;
                Ok(())
            }
            Err(e) => return Err(Error::new(Other, e)),
        }
    }

    // read from file
    pub(crate) fn editorOpenFile(&mut self, fp: &str) -> io::Result<()> {
        match File::open(fp) {
            Ok(file) => {
                let bufreader = BufReader::new(file);
                for line in bufreader.lines() {
                    self.content.push(line.unwrap().trim().to_owned());
                    self.num_rows += 1;
                }
            }
            Err(e) => return Err(Error::new(Other, e)),
        }
        Ok(())
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        println!("Program Ending\r\n");
        Terminal::disableRawMode(self).unwrap();
    }
}
