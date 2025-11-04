use crate::assembler::{Assembler, Register};
use crate::ast::AstNode;
use crate::encodings::LispValue;

#[derive(Debug)]
pub enum CompilerError {
    ValueTooLarge,
    IntegerTooLarge(i64),
}

pub struct Compiler {
    asm: Assembler,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            asm: Assembler::new(),
        }
    }
    pub fn compile_function(&mut self, ast_node: AstNode) -> Result<&Vec<u8>, CompilerError> {
        self.compile_expr(ast_node)?;
        self.asm.ret();
        Ok(self.asm.finalize())
    }
    fn compile_expr(&mut self, node: AstNode) -> Result<(), CompilerError> {
        // 1. Convert the high-level AST node into a low-level LispValue.
        let lisp_val = match node {
            AstNode::Integer(value) => LispValue::from_integer(value),
            AstNode::Bool(value) => LispValue::from_bool(value),
            AstNode::Char(value) => LispValue::from_char(value),
            AstNode::Nil => LispValue::nil(),
        };

        // 2. Get the raw Word for the assembler.
        let raw_word = lisp_val.as_raw_word();

        // 3. Emit the instruction.
        // (You still need to check if the *final* raw value fits in 32 bits)
        if raw_word > i32::MAX as i64 || raw_word < i32::MIN as i64 {
            return Err(CompilerError::ValueTooLarge);
        }
        self.asm.mov_reg_imm32(Register::Rax, raw_word as i32);
        Ok(())
    }
}
mod tests {

    use super::*;
    use crate::ExecBuffer;
    #[test]
    fn test_compiler() {
        let mut compiler = Compiler::new();
        let expr = 42;
        let ast_node = AstNode::Integer(expr);
        let result = compiler.compile_function(ast_node);
        assert!(result.is_ok());
        let code = result.unwrap();
        let exec = ExecBuffer::new(code).unwrap();

        let func = unsafe { exec.as_function::<unsafe extern "C" fn() -> i64>() };
        let encoded_result = unsafe { func() };
        let lisp_val = LispValue::from_raw_word(encoded_result);
        assert_eq!(lisp_val.as_integer(), Some(expr));
    }
    #[test]
    fn test_bool() {
        let mut compiler = Compiler::new();
        let expr = true;
        let ast_node = AstNode::Bool(expr);
        let result = compiler.compile_function(ast_node);
        assert!(result.is_ok());
        let code = result.unwrap();
        let exec = ExecBuffer::new(code).unwrap();

        let func = unsafe { exec.as_function::<unsafe extern "C" fn() -> i64>() };
        let encoded = unsafe { func() };
        let lisp_val = LispValue::from_raw_word(encoded);
        assert_eq!(lisp_val.as_bool(), Some(expr));
    }
}
