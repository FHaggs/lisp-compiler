// [File: compiler.rs]

use crate::assembler::{Assembler, Register};
use crate::ast::AstNode; // UnaryOp is no longer imported
use crate::encodings::{LispValue, Pair, Symbol};
use std::collections::HashMap;

#[derive(Debug)]
pub enum CompilerError {
    ValueTooLarge,
    IntegerTooLarge(i64),
    ValueRequires64BitMove,
    AssemblerError(String),
    NotAFunction(String),
    NotASymbol,
    InvalidArguments(String),
}

pub struct Compiler {
    asm: Assembler,
    symbol_table: HashMap<String, *mut Symbol>,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            asm: Assembler::new(),
            symbol_table: HashMap::new(),
        }
    }

    /// Allocates a Pair on the heap and returns a raw pointer.
    fn heap_alloc_pair(&mut self, car: LispValue, cdr: LispValue) -> *mut Pair {
        let pair = Box::new(Pair { car, cdr });
        Box::leak(pair)
    }

    /// Allocates a Symbol on the heap and returns a raw pointer.
    /// TODO: Change this for bump alloc
    fn heap_alloc_symbol(&mut self, name: String) -> *mut Symbol {
        let symbol = Box::new(Symbol { name });
        Box::leak(symbol)
    }

    /// Interns a symbol: ensures only one copy of each symbol string exists.
    fn intern_symbol(&mut self, name: &str) -> LispValue {
        if let Some(ptr) = self.symbol_table.get(name) {
            LispValue::from_symbol_pointer(*ptr)
        } else {
            let ptr = self.heap_alloc_symbol(name.to_string());
            self.symbol_table.insert(name.to_string(), ptr);
            LispValue::from_symbol_pointer(ptr)
        }
    }

    /// Consumes the compiler and returns the compiled machine code.
    pub fn compile_function(
        &mut self, // Takes ownership of self TODO Add this
        ast_node: &AstNode,
    ) -> Result<&Vec<u8>, CompilerError> {
        self.compile_expr(ast_node)?;
        self.asm.ret();
        Ok(self.asm.finalize())
    }

    fn compile_expr(&mut self, node: &AstNode) -> Result<(), CompilerError> {
        match node {
            AstNode::Integer(value) => {
                let lisp_val = LispValue::from_integer(*value);
                self.asm
                    .mov_reg_imm32(Register::Rax, lisp_val.as_raw_word() as i32);
            }
            AstNode::Bool(value) => {
                let lisp_val = LispValue::from_bool(*value);
                self.asm
                    .mov_reg_imm32(Register::Rax, lisp_val.as_raw_word() as i32);
            }
            AstNode::Char(value) => {
                let lisp_val = LispValue::from_char(*value);
                self.asm
                    .mov_reg_imm32(Register::Rax, lisp_val.as_raw_word() as i32);
            }
            AstNode::Nil => {
                let lisp_val = LispValue::nil();
                self.asm
                    .mov_reg_imm32(Register::Rax, lisp_val.as_raw_word() as i32);
            }
            AstNode::Symbol(name) => {
                let lisp_val = self.intern_symbol(name);
                // For now symbols are not fully implemented
                // self.asm
                //     .mov_reg_imm64(Register::Rax, lisp_val.as_raw_word());
            }

            AstNode::Pair { car, cdr } => {
                // A Pair in evaluation position means a function call.
                // We must check that the 'car' is a symbol.
                if let AstNode::Symbol(name) = &**car {
                    match name.as_str() {
                        "add1" => {
                            // This is a primitive unary function.
                            // We expect one argument in the `cdr` list.
                            if let AstNode::Pair {
                                car: arg1,
                                cdr: arg_rest,
                            } = &**cdr
                            {
                                // Check it's (add1 arg1), not (add1 arg1 arg2 ...)
                                if let AstNode::Nil = **arg_rest {
                                    // 1. Compile the argument. Result is in RAX.
                                    self.compile_expr(arg1)?;

                                    // 2. Emit the 'add1' operation
                                    let encoded_one = LispValue::from_integer(1).as_raw_word();
                                    self.asm.add_reg_imm32(Register::Rax, encoded_one as i32);
                                } else {
                                    return Err(CompilerError::InvalidArguments(
                                        "add1 expects 1 argument".to_string(),
                                    ));
                                }
                            } else {
                                return Err(CompilerError::InvalidArguments(
                                    "add1 expects 1 argument".to_string(),
                                ));
                            }
                        }
                        // "sub1" => { ... }
                        // "if" => { ... this will be a "special form" ... }
                        // When is a user defined function we need to jump to the pointer
                        _ => return Err(CompilerError::NotAFunction(name.clone())),
                    }
                } else {
                    // The 'car' was not a symbol, e.g., ((+ 1 2) 3)
                    // This is a higher-order function, which we don't support yet.
                    return Err(CompilerError::NotASymbol);
                }
            }
        }
        Ok(())
    }
}

mod tests {

    use super::*;
    use crate::ExecBuffer;
    use crate::ast::AstNode; // Import AstNode
    use crate::encodings::LispValue; // Import LispValue
    #[test]
    fn test_compiler() {
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
    #[test]
    fn test_bool() {
        let mut compiler = Compiler::new();
        let expr = true;
        let ast_node = AstNode::Bool(expr);
        let result = compiler.compile_function(&ast_node);
        assert!(result.is_ok());
        let code = result.unwrap();
        let exec = ExecBuffer::new(&code).unwrap();

        let func = unsafe { exec.as_function::<unsafe extern "C" fn() -> i64>() };
        let encoded = unsafe { func() };
        let lisp_val = LispValue::from_raw_word(encoded);
        assert_eq!(lisp_val.as_bool(), Some(expr));
    }

    // --- UPDATED: Test for add1 ---
    #[test]
    fn test_add1() {
        let mut compiler = Compiler::new();

        // This is the "Lisp way" AST for `(add1 10)`
        let ast_node = AstNode::Pair {
            car: Box::new(AstNode::Symbol("add1".to_string())),
            cdr: Box::new(AstNode::Pair {
                car: Box::new(AstNode::Integer(10)),
                cdr: Box::new(AstNode::Nil),
            }),
        };

        let result = compiler.compile_function(&ast_node);
        assert!(result.is_ok()); // This should pass now

        let code = result.unwrap();
        let exec = ExecBuffer::new(&code).unwrap();

        let func = unsafe { exec.as_function::<unsafe extern "C" fn() -> i64>() };
        let encoded_result = unsafe { func() };
        let lisp_val = LispValue::from_raw_word(encoded_result);

        // The result should be the encoded value for 11
        assert!(lisp_val.is_integer());
        assert_eq!(lisp_val.as_integer(), Some(11));
    }
}
