// [File: reader.rs]

use crate::ast::AstNode;
use crate::tokenizer::{Token, Tokenizer}; // Make sure Token is imported
use std::iter::Peekable;

/// The Parser (or "Reader")
pub struct Parser<'a> {
    tokens: Peekable<Tokenizer<'a>>,
}

impl<'a> Parser<'a> {
    /// Creates a new parser for a given string.
    pub fn new(input: &'a str) -> Self {
        Parser {
            tokens: Tokenizer::new(input).peekable(),
        }
    }

    /// The main public API. It parses a single "form" (S-expression).
    pub fn read_form(&mut self) -> Result<AstNode, String> {
        let token = self
            .tokens
            .next()
            .ok_or_else(|| "Unexpected end of input".to_string())?;

        match token {
            Token::LParen => self.read_list_tail(),
            Token::RParen => Err("Unexpected ')'".to_string()),

            Token::Char(c) => Ok(AstNode::Char(c)),

            Token::Integer(i) => Ok(AstNode::Integer(i)),
            Token::Symbol(s) => self.parse_symbol(s),
        }
    }

    /// This helper is called right after we consume a '('.
    fn read_list_tail(&mut self) -> Result<AstNode, String> {
        let token = self
            .tokens
            .peek()
            .ok_or_else(|| "Unmatched '('".to_string())?;

        match token {
            Token::RParen => {
                self.tokens.next(); // Consume the ')'
                Ok(AstNode::Nil)
            }
            _ => {
                let car = self.read_form()?;
                let cdr = self.read_list_tail()?;
                Ok(AstNode::Pair {
                    car: Box::new(car),
                    cdr: Box::new(cdr),
                })
            }
        }
    }

    /// A helper to convert a symbol token into the correct AstNode.
    fn parse_symbol(&self, s: String) -> Result<AstNode, String> {
        match s.as_str() {
            "true" => Ok(AstNode::Bool(true)),
            "false" => Ok(AstNode::Bool(false)),
            "nil" => Ok(AstNode::Nil),
            _ => Ok(AstNode::Symbol(s)),
        }
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_reader() {
        let mut reader = Parser::new("(1 2 3)");
        assert_eq!(
            reader.read_form(),
            Ok(AstNode::Pair {
                car: Box::new(AstNode::Integer(1)),
                cdr: Box::new(AstNode::Pair {
                    car: Box::new(AstNode::Integer(2)),
                    cdr: Box::new(AstNode::Pair {
                        car: Box::new(AstNode::Integer(3)),
                        cdr: Box::new(AstNode::Nil),
                    }),
                }),
            })
        );
    }
}
