# About RUST WRD

This project was worked on over the course of Summer 2023 as a way to learn the Rust Programming Language
as well as some basic systems programming. I took inspiration from a class assignment at UCLA. However, they 
had some starter code for the text editor that they built off of. My project was built from the ground up
following a guide written in C found [here](https://viewsourcecode.org/snaptoken/kilo/index.html) by [Salvatore Sanfilippo](https://github.com/antirez).

I wrote this completely from scratch in Rust using the guide and supporting documentation websites as reference.
The bulk of the logic such as reading/writing to a file was done on my own - wheras stuff I've never done before 
like sending escape sequences and setting terminal flags were copied and translated from C into Rust.

I learned a lot throughout this project, and I'm proud of the end result!

## Steps to build:

You need to install the rust compiler and cargo package manager. Once those are installed, you can simply
clone the repository and run `cargo run` as seen below:

```shell
git clone git@github.com:IssaAboudi/rustwrd.git
cargo run
```

Make sure you run this in a terminal not in an IDE - or else you'll
get an error that says "Inappropriate ioctl for device".

## Opening a file:

Supply a file path as the only argument to this program like:

```shell
cargo run test.txt
```

## How to use

- Ctrl + q = Quit the application
- Ctrl + u = Clears current line completely
- Ctrl + s = Saves the file to disk
- Home = Jumps to beginning of the line
- End = Jumps to end of the line
- PgUp = Scrolls Up
- PgDwn = Scrolls Down

I may extend this to have more in the future!