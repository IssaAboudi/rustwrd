#![allow(non_snake_case)]
#![allow(unused_imports)]

use std::fs::read;
use std::io;
use std::io::{Read, stdin};
use std::os::fd::AsRawFd;
use crossterm::terminal::enable_raw_mode;
use nix::libc::{ISTRIP, STDIN_FILENO};
use nix::sys::termios;

struct Terminal {
    orig_termios : termios::Termios
}

impl Terminal {

    fn enableRawMode(&mut self) {
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
        raw.local_flags.remove(termios::LocalFlags::ECHO //disables your input being spit back to you
            | termios::LocalFlags::ICANON //disables cannonical mode
            | termios::LocalFlags::ISIG // disables interrupts (^c, ^z)
            | termios::LocalFlags::IEXTEN // disables typing other characters literally (^v)
        );
        raw.output_flags.remove(termios::OutputFlags::OPOST // disable '\n' -> '\r\n' translation
        );
        termios::tcsetattr(fd, termios::SetArg::TCSAFLUSH, &raw).unwrap();
        raw.control_flags.remove( termios::ControlFlags::CS8 // sets character size to 8 bits per byte
        );
    }

    fn disableRawMode(&self) {
        let fd = stdin().as_raw_fd();
        termios::tcsetattr(fd, termios::SetArg::TCSAFLUSH, &self.orig_termios).unwrap();
    }
}

fn main() -> io::Result<()> {
    let mut terminal = Terminal {
        orig_termios: termios::tcgetattr(STDIN_FILENO).unwrap(),
    };

    terminal.enableRawMode();

    let mut c: char;
    //loop through all input bytes
    for byte in stdin().bytes() {
        let b = byte?;
        c = b as char;
        if c == 'q' {
            //q exits the program
            break;
        } else if c.is_ascii_control() {
            //^ + letter gives the number of that letter
            println!("{}\r\n", b);
        } else {
            //otherwise just display the character then it's ascii value
            println!("[`{}`]: , {}\r\n", c, b);
        }
    }

    terminal.disableRawMode();

    println!("Program Ending");
    Ok(())
}
