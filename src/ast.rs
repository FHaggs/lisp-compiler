// The ASTNode enum represents a value that can be one of several types.
// We're starting with just integers.
#[derive(Debug, Clone, Copy)] // Useful traits for printing and copying
pub enum AstNode {
    Integer(i64),
    Bool(bool),
    Char(char),
    Nil,
    // Later, you'll add more variants here, e.g.:
    // Pair(*mut Pair),
    // Symbol(u64),
}
