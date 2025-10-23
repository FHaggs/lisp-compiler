use crate::assembler::{Assembler, Register};
use crate::ast::AstNode;
use crate::encodings;

/// A simple error type for our compiler.
#[derive(Debug)]
pub enum CompilerError {
    UnexpectedNodeType,
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
        match node {
            AstNode::Integer(value) => {
                let encoded_value = encodings::encode_integer(value);

                self.asm.mov_reg_imm32(Register::Rax, encoded_value as i32);
                Ok(())
            }
            AstNode::Bool(value) => {
                let encoded_value = encodings::encode_bool(value);
                self.asm.mov_reg_imm32(Register::Rax, encoded_value as i32);
                Ok(())
            }
            AstNode::Char(char) => {
                let encoded_value = encodings::encode_char(char);
                self.asm.mov_reg_imm32(Register::Rax, encoded_value as i32);
                Ok(())
            }
            AstNode::Nil => {
                let encoded_value = encodings::object_nil();
                self.asm.mov_reg_imm32(Register::Rax, encoded_value as i32);
                Ok(())
            }
        }
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
        let r = unsafe { func() };
        assert_eq!(encodings::decode_integer(r), expr);
    }
}
