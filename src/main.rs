mod assembler;
mod ast;
mod compiler;
mod encodings;
mod executable_buffer;

use ast::AstNode;
use compiler::Compiler;
use encodings::LispValue;
use executable_buffer::ExecBuffer;

fn main() {
    let mut compiler = Compiler::new();
    let expr = 42;
    let ast_node = AstNode::Integer(expr);
    let result = compiler.compile_function(&ast_node);
    assert!(result.is_ok());
    let code = result.unwrap();
    let exec = ExecBuffer::new(&code).unwrap();

    let func = unsafe { exec.as_function::<unsafe extern "C" fn() -> i64>() };
    let encoded_result = unsafe { func() };
    let lisp_val = LispValue::from_raw_word(encoded_result);
    assert_eq!(lisp_val.as_integer(), Some(expr));
}
