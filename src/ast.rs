#[derive(Debug, Clone, PartialEq)]
pub enum AstNode {
    Integer(i64),
    Bool(bool),
    Char(char),
    Nil,
    Pair {
        car: Box<AstNode>,
        cdr: Box<AstNode>,
    },
    Symbol(String),
}
