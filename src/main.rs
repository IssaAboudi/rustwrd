mod Arabizi;
mod input;
mod output;
mod utils;
use crate::input::{editor_check_keycodes, editor_process_keypress, EditorMode, EditorStates};
use crate::output::editor_refresh_screen;
use crate::utils::constants::{CLR_SCREEN, MOV_CURS_HOME};
use crate::Arabizi::{import_frequencies, LanguageFrequencies};
use once_cell::sync::Lazy;

mod terminal;

use terminal::Terminal;

use nix::libc::STDIN_FILENO;
use nix::sys::termios;
use phf::phf_map;
use std::collections::HashMap;
use std::io::{stdout, Write};
use std::sync::OnceLock;
use std::{env, io};

static LANGUAGE_FREQUENCIES: OnceLock<LanguageFrequencies> = OnceLock::new();

fn get_frequencies() -> &'static LanguageFrequencies {
    LANGUAGE_FREQUENCIES.get_or_init(|| match import_frequencies() {
        Ok(freq) => freq,
        Err(_) => LanguageFrequencies::default(),
    })
}
static EN_TO_AR: phf::Map<char, &'static str> = phf_map! {
    'a' => "ش",
    's' => "س",
    'd' => "ي",
    'f' => "ب",
    'g' => "ل",
    'h' => "ا",
    'j' => "ت",
    'k' => "ن",
    'l' => "م",
    ';' => "ك",
    '\'' => "ط",
    '`' => "ذ",
    'q' => "ض",
    'w' => "ص",
    'e' => "ث",
    'r' => "ق",
    't' => "ف",
    'y' => "غ",
    'u' => "ع",
    'i' => "ه",
    'o' => "خ",
    'p' => "ح",
    '[' => "ج",
    ']' => "د",
    'z' => "ئ",
    'x' => "ء",
    'c' => "ؤ",
    'v' => "ر",
    'b' => "لا",
    'n' => "ى",
    'm' => "ة",
    ',' => "و",
    '.' => "ز",
    '/' => "ظ",
};

// entry point
fn main() -> io::Result<()> {
    let args: Vec<_> = env::args().collect();

    let mut terminal = Terminal {
        orig_termios: termios::tcgetattr(STDIN_FILENO)?,
        screen_rows: 0,
        screen_cols: 0,
        snip_start: 0,
        curs_x: 0,
        term_mode: EditorMode::Normal,
        curs_y: 0,
        v_offset: 0,
        fp: String::new(),
        content: Vec::new(),
    };

    terminal.enable_raw_mode()?;
    terminal.init_editor()?;
    if args.len() >= 2 {
        terminal.editor_open_file(&args[1])?;
    }

    loop {
        match terminal.term_mode {
            EditorMode::Normal | EditorMode::Arabizi => {
                editor_refresh_screen(&mut terminal)?;
                match editor_process_keypress(&mut terminal) {
                    Ok(EditorStates::Exit) => {
                        stdout().write_all(CLR_SCREEN)?;
                        stdout().write_all(MOV_CURS_HOME)?;
                        break;
                    }
                    Ok(EditorStates::ChangeMode(new_mode)) => {
                        stdout().write_all(CLR_SCREEN)?;
                        stdout().write_all(MOV_CURS_HOME)?;
                        terminal.term_mode = new_mode;
                    }
                    Ok(_) => continue,
                    Err(_e) => {
                        editor_refresh_screen(&mut terminal)?;
                    }
                }
            }
            EditorMode::KeyCodeReader => match editor_check_keycodes() {
                Ok(EditorStates::ChangeMode(new_mode)) => {
                    stdout().write_all(CLR_SCREEN)?;
                    stdout().write_all(MOV_CURS_HOME)?;
                    terminal.term_mode = new_mode;
                }
                Ok(_) => continue,
                Err(_e) => return Err(_e),
            },
        }
    }

    Ok(())
}
