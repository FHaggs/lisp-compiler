use crate::assembler::{Assembler, PartialRegister, Register, SetccConditions};
use crate::ast::AstNode;
use crate::encodings::{
    K_BOOL_MASK, K_BOOL_SHIFT, K_BOOL_TAG, K_CHAR_SHIFT, K_CHAR_TAG, K_INTEGER_MASK,
    K_INTEGER_SHIFT, K_INTEGER_TAG, LispValue, Pair, Symbol,
};
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
    fn compile_call(&mut self, car: &AstNode, cdr: &AstNode) -> Result<(), CompilerError> {
        // A Pair in evaluation position means a function call.
        // We must check that the 'car' is a symbol.
        if let AstNode::Symbol(name) = car {
            match name.as_str() {
                // TODO: This is temporary, we should use a more complex symbol table
                "add1" => {
                    // This is a primitive unary function.
                    // We expect one argument in the `cdr` list.
                    if let AstNode::Pair {
                        car: arg1,
                        cdr: arg_rest,
                    } = cdr
                    {
                        // Check it's (add1 arg1), not (add1 arg1 arg2 ...)
                        if let AstNode::Nil = &**arg_rest {
                            // 1. Compile the argument. Result is in RAX.
                            self.compile_expr(arg1)?;

                            // 2. Emit the 'add1' operation. Adding 1 << 2 due to pointer tagging
                            let encoded_one = LispValue::from_integer(1).as_raw_word();
                            self.asm.add_reg_imm32(Register::Rax, encoded_one as i32);
                            Ok(())
                        } else {
                            return Err(CompilerError::InvalidArguments(
                                "add1 expects 1 argument".to_string(),
                            ));
                        }
                    } else {
                        return Err(CompilerError::InvalidArguments(
                            "add1 expects 1 argument. Needs a pair".to_string(),
                        ));
                    }
                }
                "sub1" => {
                    if let AstNode::Pair { car: arg, cdr: _ } = cdr {
                        self.compile_expr(arg)?;

                        let encoded_one = LispValue::from_integer(1).as_raw_word();
                        self.asm.sub_reg_imm32(Register::Rax, encoded_one as i32);
                        Ok(())
                    } else {
                        Err(CompilerError::InvalidArguments(
                            "sub1 expects 1 argument. Needs a pair".to_string(),
                        ))
                    }
                }
                "integer->char" => {
                    if let AstNode::Pair { car: arg1, cdr: _ } = cdr {
                        self.compile_expr(arg1)?;
                        self.asm
                            .shl_reg_imm8(Register::Rax, (K_CHAR_SHIFT - K_INTEGER_SHIFT) as u8)
                            .or_reg_imm8(Register::Rax, K_CHAR_TAG as u8);
                        Ok(())
                    } else {
                        return Err(CompilerError::InvalidArguments(
                            "integer->char expects 1 argument. Needs a pair".to_string(),
                        ));
                    }
                }
                "nil?" => {
                    if let AstNode::Pair { car: arg1, cdr: _ } = cdr {
                        self.compile_expr(arg1)?;
                        self.compile_compare_imm32(LispValue::nil());
                        Ok(())
                    } else {
                        return Err(CompilerError::InvalidArguments(
                            "nil? expects 1 argument. Needs a pair".to_string(),
                        ));
                    }
                }
                "zero?" => {
                    if let AstNode::Pair { car: arg1, cdr: _ } = cdr {
                        self.compile_expr(arg1)?;
                        self.compile_compare_imm32(LispValue::from_integer(0));
                        Ok(())
                    } else {
                        return Err(CompilerError::InvalidArguments(
                            "zero? expects 1 argument. Needs a pair".to_string(),
                        ));
                    }
                }

                "integer?" => {
                    if let AstNode::Pair { car: arg1, cdr: _ } = cdr {
                        self.compile_expr(arg1)?;
                        self.asm.and_reg_imm8(Register::Rax, K_INTEGER_MASK as u8);
                        self.compile_compare_imm32(LispValue::from_raw_word(K_INTEGER_TAG));
                        Ok(())
                    } else {
                        return Err(CompilerError::InvalidArguments(
                            "integer? expects 1 argument. Needs a pair".to_string(),
                        ));
                    }
                }
                "bool?" => {
                    // TODO: Add test to this func
                    if let AstNode::Pair { car: arg1, cdr: _ } = cdr {
                        self.compile_expr(arg1)?;
                        self.asm.and_reg_imm8(Register::Rax, K_BOOL_MASK as u8);
                        self.compile_compare_imm32(LispValue::from_raw_word(K_BOOL_TAG));
                        Ok(())
                    } else {
                        return Err(CompilerError::InvalidArguments(
                            "integer? expects 1 argument. Needs a pair".to_string(),
                        ));
                    }
                }
                // "sub1" => { ... }
                // "if" => { ... this will be a "special form" ... }
                _ => return Err(CompilerError::NotAFunction(name.clone())),
            }
        } else {
            // The 'car' was not a symbol, e.g., ((+ 1 2) 3)
            // This is a higher-order function, which we don't support yet.
            return Err(CompilerError::NotASymbol);
        }
    }
    fn compile_compare_imm32(&mut self, value: LispValue) {
        self.asm
            .cmp_reg_imm32(Register::Rax, value.as_raw_word() as u32)
            .mov_reg_imm32(Register::Rax, 0)
            .setcc_imm8(SetccConditions::Equal, PartialRegister::Al)
            .shl_reg_imm8(Register::Rax, K_BOOL_SHIFT as u8)
            .or_reg_imm8(Register::Rax, K_BOOL_TAG as u8);
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

            AstNode::Pair { car, cdr } => self.compile_call(car, cdr)?,
        }
        Ok(())
    }
}

mod tests {

    use super::*;
    use crate::ExecBuffer;
    use crate::ast::AstNode; // Import AstNode
    use crate::encodings::LispValue; // Import LispValue
    fn compile_ast(ast_node: AstNode) -> LispValue {
        let mut compiler = Compiler::new();
        let result = compiler.compile_function(&ast_node);
        assert!(result.is_ok());
        let code = result.unwrap();
        let exec = ExecBuffer::new(&code).unwrap();

        let func = unsafe { exec.as_function::<unsafe extern "C" fn() -> i64>() };
        let encoded_result = unsafe { func() };
        LispValue::from_raw_word(encoded_result)
    }
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

    #[test]
    fn test_sub1() {
        let mut compiler = Compiler::new();

        // This is the "Lisp way" AST for `(add1 10)`
        let ast_node = AstNode::Pair {
            car: Box::new(AstNode::Symbol("sub1".to_string())),
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
        assert_eq!(lisp_val.as_integer(), Some(9));
    }
    #[test]
    fn test_nested_adds() {
        let mut compiler = Compiler::new();
        let val = 10;
        let expected = val + 2;
        // Test (add1 (add1 5))
        let ast_node = AstNode::Pair {
            car: Box::new(AstNode::Symbol("add1".to_string())),
            cdr: Box::new(AstNode::Pair {
                car: Box::new(AstNode::Pair {
                    car: Box::new(AstNode::Symbol("add1".to_string())),
                    cdr: Box::new(AstNode::Pair {
                        car: Box::new(AstNode::Integer(val)),
                        cdr: Box::new(AstNode::Nil),
                    }),
                }),
                cdr: Box::new(AstNode::Nil),
            }),
        };

        let result = compiler.compile_function(&ast_node);
        assert!(result.is_ok());

        let code = result.unwrap();
        let exec = ExecBuffer::new(&code).unwrap();

        let func = unsafe { exec.as_function::<unsafe extern "C" fn() -> i64>() };
        let encoded_result = unsafe { func() };
        let lisp_val = LispValue::from_raw_word(encoded_result);

        // The result should be the encoded value for 11
        assert!(lisp_val.is_integer());
        assert_eq!(lisp_val.as_integer(), Some(expected));
    }
    #[test]
    fn test_int2char() {
        let ast_node = AstNode::Pair {
            car: Box::new(AstNode::Symbol("integer->char".to_string())),
            cdr: Box::new(AstNode::Pair {
                car: Box::new(AstNode::Integer(64)),
                cdr: Box::new(AstNode::Nil),
            }),
        };
        let lisp_val = compile_ast(ast_node);
        let char = lisp_val.as_char();
        // 64 in the asscii table is @
        assert_eq!(char, Some('@'));
    }
    #[test]
    fn test_is_nill() {
        let ast_node = AstNode::Pair {
            car: Box::new(AstNode::Symbol("nil?".to_string())),
            cdr: Box::new(AstNode::Pair {
                car: Box::new(AstNode::Nil),
                cdr: Box::new(AstNode::Nil),
            }),
        };
        let lisp_val = compile_ast(ast_node);
        let bool = lisp_val.as_bool();
        assert!(bool.is_some());
        assert_eq!(bool.unwrap(), true);
    }

    #[test]
    fn test_is_zero() {
        let ast_node = AstNode::Pair {
            car: Box::new(AstNode::Symbol("zero?".to_string())),
            cdr: Box::new(AstNode::Pair {
                car: Box::new(AstNode::Integer(0)),
                cdr: Box::new(AstNode::Nil),
            }),
        };
        let lisp_val = compile_ast(ast_node);
        let bool = lisp_val.as_bool();
        assert!(bool.is_some());
        assert_eq!(bool.unwrap(), true);
    }

    #[test]
    fn test_is_int() {
        let ast_node = AstNode::Pair {
            car: Box::new(AstNode::Symbol("integer?".to_string())),
            cdr: Box::new(AstNode::Pair {
                car: Box::new(AstNode::Integer(19283)),
                cdr: Box::new(AstNode::Nil),
            }),
        };
        let lisp_val = compile_ast(ast_node);
        let bool = lisp_val.as_bool();
        assert!(bool.is_some());
        assert_eq!(bool.unwrap(), true);
    }
    #[test]
    fn test_not_integer() {
        let ast_node = AstNode::Pair {
            car: Box::new(AstNode::Symbol("integer?".to_string())),
            cdr: Box::new(AstNode::Pair {
                car: Box::new(AstNode::Char('a')),
                cdr: Box::new(AstNode::Nil),
            }),
        };
        let lisp_val = compile_ast(ast_node);
        for b in lisp_val.as_raw_word().to_ne_bytes() {
            print!("{:02X} ", b);
        }
        let bool = lisp_val.as_bool();
        assert!(bool.is_some());
        assert_eq!(bool.unwrap(), false);
    }
}
