/// This mod compiles quahog code into stack machine bytecode.
mod parser;
mod memory;
mod compiler;
mod bytecode;
mod assembler;
mod disassembler;
use std::{fs::File, io::Read};

pub use bytecode::Command;

pub use {compiler::compile_str, assembler::assemble_str, disassembler::disassemble_bytes};

/// Compile a file of Biscuit code to binary
pub fn compile_file(filename: &str) -> Result<Vec<u8>, String> {
    let mut file = File::open(filename).map_err(|_| format!("Could not find file {}", filename))?;
    let mut text = "".to_owned();
    file.read_to_string(&mut text).map_err(|_| format!("Could not read file {}", filename))?;

    compile_str(&text, filename)
}

// Compile a file of Biscuit assembly to binary
pub fn assemble_file(filename: &str) -> Result<Vec<u8>, String> {
    let mut file = File::open(filename).map_err(|_| format!("Could not find file {}", filename))?;
    let mut text = "".to_owned();
    file.read_to_string(&mut text).map_err(|_| format!("Could not read file {}", filename))?;

    assemble_str(&text, filename)
}

// Compile a file (usually read from a file) of Biscuit binary to assembly
pub fn disassemble_file(filename: &str) -> Result<String, String> {
    let bytes = std::fs::read(filename).map_err(|_| format!("Could not find file {}", filename))?;

    disassemble_bytes(&bytes, filename)
}