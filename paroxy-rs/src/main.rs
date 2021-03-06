use std::{fs, path::PathBuf};

use chunk::Chunk;
use clap::Parser;
use scanner::Scanner;
use vm::VM;

mod chunk;
mod opcode;

mod cli;
mod debug;
mod parser;
mod scanner;
mod token;
mod vm;

fn main() {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Commands::Run {
            source,
            file,
            compiled,
        } => {
            if compiled && !file {
                panic!("use '--file' flag when running compiled chunk.");
            }

            match get_chunk(source, file, compiled) {
                Ok(chunk) => run(chunk),
                Err(error) => panic!("{error}"),
            }
        }
        cli::Commands::Compile { source, file, out } => {
            if !file && out.is_none() {
                println!("'--out' must be used when using raw program code.");
                return;
            }

            let program = get_program(source.clone(), file);

            let chunk = match parse(program) {
                Ok(chunk) => chunk,
                Err(_) => return,
            };

            let bytes = chunk.as_bytes().expect("Failed to serialize data");

            let file = match out {
                Some(path) => path,
                None => {
                    let source_file = PathBuf::from(source);
                    let parent = source_file.parent().unwrap();

                    let out_stem = source_file.file_stem().unwrap().to_string_lossy();
                    let out_name = format!("{out_stem}.pxb");

                    parent.join(out_name)
                }
            };

            fs::write(file, bytes).expect("Failed to write bytecode.");
        }
    }
}

fn get_chunk(source: String, file: bool, compiled: bool) -> Result<Chunk, &'static str> {
    if compiled {
        let bytes = fs::read(source).expect("Unable to read file.");

        match Chunk::from_bytes(&bytes) {
            Ok(chunk) => Ok(chunk),
            Err(_) => Err("Failed to load chunk from binary data."),
        }
    } else {
        let program = get_program(source, file);
        parse(program)
    }
}

fn get_program(source: String, file: bool) -> String {
    if file {
        fs::read_to_string(source).expect("Unable to read file.")
    } else {
        source
    }
}

fn parse(program: String) -> Result<Chunk, &'static str> {
    let mut chunk = Chunk::new();

    let scanner = Scanner::new(program.as_str());
    let success = parser::Parser::new(scanner, &mut chunk).compile();

    if success {
        Ok(chunk)
    } else {
        Err("Compilation failed")
    }
}

fn run(chunk: Chunk) {
    let mut vm = VM::new(chunk);
    vm.run();
}
