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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
    let start = terminal.snip_start as usize;
    let end = terminal.curs_x as usize;
    let slice = &terminal.content[terminal.curs_y as usize][start..end + 1];
    return String::from(slice);
}

fn calculate_english_freq(word: String) -> f64 {
    let frequencies = LANGUAGE_FREQUENCIES.read().unwrap();
    // loops through the string from snip_start to curs_x
    let length = word.chars().count();
    let mut cumfreq: f64 = 0.0;
    for char in word.chars() {
        cumfreq += match frequencies.english.get(&char) {
            Some(t) => t,
            None => &0.0,
        };
    }
    return cumfreq / (length as f64);
}

fn calculate_arabic_freq(word: String) -> f64 {
    let frequencies = LANGUAGE_FREQUENCIES.read().unwrap();
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
    let eng = word.to_lowercase();
    let dictionary = EN_TO_AR.read().unwrap();
    let mut ara: String = String::new();
    for char in eng.chars() {
        if let Some(ch) = dictionary.get(&char) {
            ara.push_str(ch);
        }
    }

    return ara;
}

pub(crate) fn arabizi_process_input(terminal: &mut Terminal) {
    // get english and arabic words
    let english = get_snippet_to_change(terminal);
    let arabic = convert_to_arabic(&english);

    // calculate their respective frequencies
    let eng_freq = calculate_english_freq(english);
    let ara_freq = calculate_arabic_freq(arabic);
    println!("eng: {} vs ara: {}", eng_freq, ara_freq);

    // return the arabic string to be written out
}
