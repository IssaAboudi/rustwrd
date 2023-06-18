#![allow(non_snake_case)]

use std::io;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

fn read_data<'a>(buf: &'a mut String, filepath: &str) -> Result<&'a str, std::io::Error>{
    let mut file = File::open(filepath)?;

    file.read_to_string(buf)?;
    println!("Reading from file:\n{}", buf);
    Ok("Ran Successfully")
}

fn write_data<'a>(buf: &'a mut String, filepath: &str) -> Result<&'a str, std::io::Error>{
    let mut file = File::create(filepath)?;

    println!("What would you like to write to the file: \n");
    io::stdin().read_line(buf).expect("Error with message");

    file.write(buf.trim().as_bytes())?;
    Ok("Ran Successfully")
}

fn main() {

    //Select menu option - 0 is invalid
    let mut option_sel = String::from("0");

    //buffer of data we write to a file and load from file
    let mut buf = String::new();

    loop {
        println!("Welcome to VIM Clone v0.0.1 Text Editor");
        println!("1) Edit data");
        println!("2) Read data");
        println!("3) Exit app");



        option_sel.clear();
        io::stdin().read_line(  &mut option_sel).expect("Error with message");

        let result = match option_sel.as_str().trim() {
            "1" => {write_data(&mut buf, "./random_data.txt").expect("bad message"); "ran as planned"},
            "2" => {read_data(&mut buf, "./random_data.txt").expect("bad"); "ran as planned"}
            _ => "nothing happened",
        };

        println!("{}", result);

        if option_sel.trim().to_string() == "3" {
            break;
        }
    }

}
