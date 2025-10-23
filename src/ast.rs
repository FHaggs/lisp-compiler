#[derive(Debug, Clone, Copy)]
pub enum AstNode {
    Integer(i64),
    Bool(bool),
    Char(char),
    Nil,
    // Pair(*mut Pair),
    // Symbol(u64),
}
