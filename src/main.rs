mod assembler;
mod ast;
mod compiler;
mod encodings;
mod executable_buffer;
mod reader;
mod tokenizer;

use iced_x86::{Decoder, DecoderOptions, Formatter, NasmFormatter};
use std::io::{self, Write};

use ast::AstNode;
use compiler::Compiler;
use encodings::LispValue;
use executable_buffer::ExecBuffer;
use libc::CODESET;

use crate::reader::Parser;

fn main() {
    // let mut compiler = Compiler::new();
    // let expr = 42;
    // let ast_node = AstNode::Integer(expr);
    // let result = compiler.compile_function(&ast_node);
    // assert!(result.is_ok());
    // let code = result.unwrap();
    // let exec = ExecBuffer::new(&code).unwrap();

    // let func = unsafe { exec.as_function::<unsafe extern "C" fn() -> i64>() };
    // let encoded_result = unsafe { func() };
    // let lisp_val = LispValue::from_raw_word(encoded_result);
    // assert_eq!(lisp_val.as_integer(), Some(expr));
    //
    let mut compiler = Compiler::new();
    loop {
        print!("lisp> ");
        io::stdout().flush().unwrap();

        let input = read_line();
        if input == "quit" {
            break;
        }
        let mut parser = Parser::new(&input);
        if let Ok(ast) = parser.read_form() {
            println!("Parsed AST: {:?}", ast);
            let code = compiler.compile_function(&ast);
            match code {
                Ok(code) => {
                    print_disassembly(code, 64);
                }
                Err(err) => println!("Compilation error: {:?}", err),
            }
        } else {
            println!("Invalid input");
        }
    }
}
fn read_line() -> String {
    let mut buffer = String::new();
    io::stdin()
        .read_line(&mut buffer)
        .expect("Failed to read line");
    buffer.trim().to_string()
}

fn print_disassembly(bytes: &[u8], bitness: u32) {
    println!("--- Disassembly ---");
    // This is the "virtual" address our code starts at.
    // We need it to calculate the offset into our `bytes` slice.
    const BASE_IP: u64 = 0x0000_0001_0000_0000;

    let mut decoder = Decoder::new(bitness, bytes, DecoderOptions::NONE);
    decoder.set_ip(BASE_IP);

    let mut formatter = NasmFormatter::new();
    formatter.options_mut().set_digit_separator(" ");
    formatter.options_mut().set_first_operand_char_index(10);

    let mut output = String::new();

    for instruction in &mut decoder {
        // Format the instruction
        output.clear();
        formatter.format(&instruction, &mut output);

        // --- THIS IS THE FIX ---

        // 1. Calculate the start index in the *original* byte slice
        let start_index = (instruction.ip() - BASE_IP) as usize;

        // 2. Get the slice of raw bytes for this instruction
        let instruction_bytes = &bytes[start_index..start_index + instruction.len()];

        // 3. Format the bytes into a string
        let bytes_str: String = instruction_bytes
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");

        // --- END FIX ---

        // Print the IP, the raw bytes, and the instruction
        println!("0x{:016X} {: <30} {}", instruction.ip(), bytes_str, output);
    }
    println!("--- End ---");
}
