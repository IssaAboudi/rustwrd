use crate::get_frequencies;
use crate::io::Error;
use crate::Terminal;
use crate::EN_TO_AR;
use crate::LANGUAGE_FREQUENCIES;
use core::fmt;
use serde::Deserialize;
use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize)]
pub struct LanguageFrequencies {
    english: HashMap<char, f64>,
    arabic: HashMap<char, f64>,
}

impl fmt::Display for LanguageFrequencies {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        println!("English Hashmap:");
        for eng in self.english.clone() {
            let (letter, frequency) = eng;
            println!("\t{}: {}", letter, frequency);
        }
        println!("Arabic Hashmap:");
        for ara in self.arabic.clone() {
            let (letter, frequency) = ara;
            println!("\t{}: {}", letter, frequency);
        }
        Ok(())
    }
}

pub fn import_frequencies() -> Result<LanguageFrequencies, Error> {
    let freq_json: PathBuf =
        PathBuf::from(r"/Users/issaaboudi/Dev/rustwrd/resources/frequencies.json");

    let file = File::open(freq_json).expect("failed to load frequencies.json");
    let mut reader = BufReader::new(file);

    let mut content = String::new();
    reader.read_to_string(&mut content)?;
    let frequencies: LanguageFrequencies =
        serde_json::from_str(&content).expect("invalid json formatting");

    return Ok(frequencies);
}

fn get_snippet_to_change(terminal: &mut Terminal) -> String {
    // string from snip_start to curs_x
    let row = &terminal.content[terminal.curs_y as usize];
    let start = terminal.snip_start as usize;
    let end = terminal.curs_x as usize;

    // count by byte
    let start_byte = row.char_indices().nth(start).map(|(i, _)| i).unwrap_or(0);
    let end_byte = row
        .char_indices()
        .nth(end)
        .map(|(i, _)| i)
        .unwrap_or(row.len());

    return row[start_byte..end_byte].to_string();
}

fn calculate_english_freq(word: &String) -> f64 {
    let frequencies = get_frequencies();
    // loops through the string from snip_start to curs_x
    let length = word.chars().count();
    let mut cumfreq: f64 = 0.0;
    for char in word.chars() {
        cumfreq += match frequencies.english.get(&char) {
            Some(t) => t,
            None => &0.0,
        };
    }

    // if start with ; \ ' . , certainly arabic -> penalize our cumfreq
    match word.chars().next() {
        Some('\\') => cumfreq -= 5.0,
        Some('/') => cumfreq -= 5.0,
        Some(';') => cumfreq -= 5.0,
        Some('\'') => cumfreq -= 5.0,
        Some('.') => cumfreq -= 5.0,
        Some(',') => cumfreq -= 5.0,
        Some('`') => cumfreq -= 5.0,
        Some('[') => cumfreq -= 5.0,
        Some(']') => cumfreq -= 5.0,
        _ => {}
    }

    return cumfreq / (length as f64);
}

fn calculate_arabic_freq(word: &String) -> f64 {
    let frequencies = get_frequencies();
    // loops through the string from snip_start to curs_x
    let length = word.chars().count();
    let mut cumfreq: f64 = 0.0;

    for char in word.chars() {
        cumfreq += match frequencies.arabic.get(&char) {
            Some(t) => t,
            None => &0.0,
        };
    }

    return cumfreq / (length as f64);
}

fn convert_to_arabic(word: &String) -> String {
    // using a map, create arabic string based on keypresses

    // edge cases for single letter words in arabic
    if word.len() == 1 {
        if word == "," {
            return String::from("و");
        }
    }

    let eng = word.to_lowercase();
    let mut ara: String = String::new();
    for char in eng.chars() {
        if let Some(ch) = EN_TO_AR.get(&char) {
            ara.push_str(ch);
        }
    }

    return ara;
}

fn arabizi_replace_last_word(terminal: &mut Terminal, word: &String) {
    let row = &mut terminal.content[terminal.curs_y as usize];

    let row_char_count = row.chars().count();

    let start = terminal.snip_start as usize;
    let end = terminal.curs_x as usize;

    // let curs_pos = terminal.curs_x as usize;
    // let mut start = curs_pos;
    // while start > 0 && row.chars().nth(start - 1) != Some(' ') {
    //     start -= 1;
    // }

    // let end = curs_pos;

    if start >= end || end > row_char_count {
        return;
    }

    let start_byte = row.char_indices().nth(start).map(|(i, _)| i).unwrap_or(0);
    let end_byte = row
        .char_indices()
        .nth(end)
        .map(|(i, _)| i)
        .unwrap_or(row.len());

    if start_byte >= end_byte || end_byte > row.len() {
        return;
    }

    let original = row[start_byte..end_byte].to_string();
    let char_diff = word.chars().count() as i32 - original.chars().count() as i32;

    row.replace_range(start_byte..end_byte, &word);
}

pub(crate) fn arabizi_process_input(terminal: &mut Terminal) {
    // get english and arabic words

    let english = get_snippet_to_change(terminal);

    if english.trim().len() < 1 {
        return;
    }

    if english.len() == 1 {
        if english == "I" {
            return;
        } else if english == "a" {
            return;
        }
    }

    let arabic = convert_to_arabic(&english);

    // calculate their respective frequencies
    let eng_freq = calculate_english_freq(&english);
    let ara_freq = calculate_arabic_freq(&arabic);

    // assume english - don't do replacement
    if ara_freq > eng_freq {
        arabizi_replace_last_word(terminal, &arabic);
    }
}
