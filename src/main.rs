#![allow(non_snake_case)]
#![allow(unused_imports)]

use std::f32::consts::E;
use std::fs::read;
use std::io;
use std::io::{Error, ErrorKind, Read, stdin};
use std::io::ErrorKind::Other;
use std::os::fd::AsRawFd;
use crossterm::terminal::enable_raw_mode;
use nix::errno::errno;
use nix::libc::{EAGAIN, exit, ISTRIP, perror, STDIN_FILENO};
use nix::sys::termios;
use nix::sys::termios::SpecialCharacterIndices::{VMIN, VTIME};

//Macro to add CTRL modifier to each key
macro_rules! CTRL_KEY {
    ($k : expr) => {
        $k & 0x1f
    }
}

struct Terminal {
    orig_termios : termios::Termios
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
}

fn main() -> io::Result<()> {
    let mut terminal = Terminal {
        orig_termios: termios::tcgetattr(STDIN_FILENO).unwrap(),
    };

    terminal.enableRawMode()?;

    let mut c: char;
    //loop through all input bytes
    for byte in stdin().bytes() {
        let b = byte?;
        c = b as char;
        if c as u8 == CTRL_KEY!('q' as u8) {
            // ctrl q exits the program
            break;
        }else if c as i8 == -1 && errno() != EAGAIN {
            return Err(Error::new(Other, "failed at read"));
        } else if c.is_ascii_control() {
            //^ + letter gives the number of that letter
            println!("{}\r\n", b);
        } else {
            //otherwise just display the character then it's ascii value
            println!("[`{}`]: , {}\r\n", c, b);
        }
    }

    terminal.disableRawMode()?;

    println!("Program Ending");
    Ok(())
}
