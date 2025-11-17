mod assembler;
mod ast;
mod compiler;
mod encodings;
mod executable_buffer;
mod reader;
mod tokenizer;

use crate::encodings::LispValue;
use compiler::Compiler;
use executable_buffer::ExecBuffer;
use iced_x86::{Decoder, DecoderOptions, Formatter, NasmFormatter};
use std::io::{self, Write};

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

    loop {
        let compiler = Compiler::new();
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
                    print_disassembly(&code, 64);
                    let exec = ExecBuffer::new(&code).unwrap();

                    let func = unsafe { exec.as_function::<unsafe extern "C" fn() -> i64>() };
                    let encoded_result = unsafe { func() };
                    let lisp_val = LispValue::from_raw_word(encoded_result);
                    lisp_val.print();
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
        output.clear();
        formatter.format(&instruction, &mut output);

        let start_index = (instruction.ip() - BASE_IP) as usize;

        let instruction_bytes = &bytes[start_index..start_index + instruction.len()];

        let bytes_str: String = instruction_bytes
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");

        println!("0x{:016X} {: <30} {}", instruction.ip(), bytes_str, output);
    }
    println!("--- End ---");
}
