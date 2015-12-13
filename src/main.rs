pub mod vm_code_writer;
pub mod vm_parser;

use std::env;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use vm_parser::{Parser, VMCommand};
use vm_code_writer::CodeWriter;

fn main() {
    let args: Vec<_> = env::args().collect();
    
    if args.len() < 2 {
        println!("Usage: {} [file.asm]", env::args().nth(0).unwrap());
        return;
    }

    for i in 1..args.len() {
        println!("{}", args[i]);
    }
    
    let file_name = args[1].clone();
    
    // get the output file name from the input file name
    let output_file = if args.len() > 2 {
        "out".to_string()   
    } else {
        if file_name.ends_with(".vm") {
            file_name[..file_name.len() - 3].to_string()
        } else {
            file_name.to_string()
        }
    } + ".asm";
    
    println!("{}", output_file);

    let mut cw = match CodeWriter::new(&output_file[..]) {
        Ok(cw) => cw,
        Err(e) => panic!("Error! {}", e),
    };
    
    //have to keep parsers in a vec because of Sys.init thing
    let mut parsers: Vec<Parser> = Vec::new();
    let mut found_sysinit: bool = false;
    
    for i in 1..args.len() {
        let input = match File::open(&args[i]) {
            Ok(f) => f,
            Err(_) => {
                println!("Error opening file '{}'!", file_name);
                return;
            }
        };

        let reader = BufReader::new(input);
        let lines: Vec<_> = reader.lines().map(|l| l.unwrap()).collect();
        
        for line in lines.clone() {
            if line.contains("function Sys.init") {
                found_sysinit = true;
            }
        }
        
        let cur_file = args[i].clone();
        cw.file_name = if cur_file.ends_with(".vm") {
            cur_file[..cur_file.len() - 3].to_string()
        } else {
            cur_file.to_string()
        };
        
        let parser = Parser::new(cur_file, lines);
        parsers.push(parser);
    }
    
    //if the file contains a Sys.init function
    //we have to write the init code
    if found_sysinit {
        cw.write_init().unwrap();
    }
    
    for mut parser in parsers {
        cw.file_name = parser.file_name.clone();
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
    }
    
    println!("Wrote assembly to '{}'", output_file);
}
