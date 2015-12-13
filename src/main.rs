pub mod vm_code_writer;
pub mod vm_parser;

use std::env;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use vm_parser::{Parser, VMCommand};
use vm_code_writer::CodeWriter;

fn main() {
    // first command line argument is input file name
    let file_name = match env::args().nth(1) {
        Some(arg) => arg,
        None => {
            println!("Usage: {} [file.asm]", env::args().nth(0).unwrap());
            return;
        }
    };

    // get the output file name from the input file name
    let output_file = if file_name.ends_with(".vm") {
        file_name[..file_name.len() - 3].to_string()
    } else {
        file_name.to_string()
    } + ".asm";

    let input = match File::open(&file_name) {
        Ok(f) => f,
        Err(_) => {
            println!("Error opening file '{}'!", file_name);
            return;
        }
    };

    let reader = BufReader::new(input);
    let lines: Vec<_> = reader.lines().map(|l| l.unwrap()).collect();

    // create new parser from input lines
    let mut parser = Parser::new(lines);

    let mut cw = match CodeWriter::new(&output_file[..]) {
        Ok(cw) => cw,
        Err(e) => panic!("Error! {}", e),
    };

    cw.write_init().unwrap();

    while !parser.eof() {
        match parser.advance() {
            // don't write anything for Nothing
            Ok(VMCommand::Nothing) => {}
            Ok(cmd) => {
                cw.write_command(cmd).expect("Couldn't write command!");
            }
            Err(err) => println!("Error: {} - line {}", err, parser.index),
        }
    }

    println!("Wrote assembly to '{}'", output_file);
}
