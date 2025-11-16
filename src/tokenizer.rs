use std::iter::Peekable;
use std::str::Chars;

/// The "dumb" tokens your parser will receive.
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    LParen, // (
    RParen, // )
    Integer(i64),
    Symbol(String),
    Char(char), // <-- ADD THIS
}

/// The Tokenizer struct, which is itself an iterator.
pub struct Tokenizer<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token; // This iterator returns Tokens

    fn next(&mut self) -> Option<Self::Item> {
        // 1. Skip all whitespace
        self.skip_whitespace();

        // 2. Look at the next character
        let ch = self.chars.next()?;

        // 3. Decide what kind of token to make
        match ch {
            '(' => Some(Token::LParen),
            ')' => Some(Token::RParen),

            '0'..='9' => Some(self.tokenize_number(ch)),

            '#' => {
                if self.chars.peek() == Some(&'\\') {
                    self.chars.next(); // Consume the '\'
                    Some(self.tokenize_char())
                } else {
                    // It's just a symbol that starts with #
                    Some(self.tokenize_symbol(ch))
                }
            }
            _ => Some(self.tokenize_symbol(ch)),
        }
    }
}

impl<'a> Tokenizer<'a> {
    /// Creates a new tokenizer for a given string.
    pub fn new(input: &'a str) -> Self {
        Tokenizer {
            chars: input.chars().peekable(),
        }
    }

    /// Consumes and returns a number token.
    fn tokenize_number(&mut self, first_char: char) -> Token {
        let mut s = String::new();
        s.push(first_char);

        while let Some(&ch) = self.chars.peek() {
            if ch.is_digit(10) {
                s.push(self.chars.next().unwrap());
            } else {
                break;
            }
        }

        let num = s.parse::<i64>().expect("Tokenizer failed to parse number");
        Token::Integer(num)
    }

    /// Consumes and returns a symbol token.
    fn tokenize_symbol(&mut self, first_char: char) -> Token {
        let mut s = String::new();
        s.push(first_char);

        while let Some(&ch) = self.chars.peek() {
            if ch.is_whitespace() || ch == '(' || ch == ')' {
                break;
            } else {
                s.push(self.chars.next().unwrap());
            }
        }
        Token::Symbol(s)
    }

    /// Skips over any whitespace
    fn skip_whitespace(&mut self) {
        while let Some(&ch) = self.chars.peek() {
            if ch.is_whitespace() {
                self.chars.next();
            } else {
                break;
            }
        }
    }

    fn tokenize_char(&mut self) -> Token {
        // We've already consumed the #\
        // Read the rest of the symbol (e.g., "a", "space", "newline")
        let mut s = String::new();
        while let Some(&ch) = self.chars.peek() {
            if ch.is_whitespace() || ch == '(' || ch == ')' {
                break; // Delimiter found, stop
            } else {
                s.push(self.chars.next().unwrap()); // Consume
            }
        }

        if s.len() == 1 {
            // It's a single char, e.g., #\a
            Token::Char(s.chars().next().unwrap())
        } else {
            // It's a named char, e.g., #\space
            match s.as_str() {
                "space" => Token::Char(' '),
                "newline" => Token::Char('\n'),
                "tab" => Token::Char('\t'),
                _ => panic!("Unknown character name: #\\{}", s),
            }
        }
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_tokenize_char() {
        let mut tokenizer = Tokenizer::new("#\\a");
        assert_eq!(tokenizer.next(), Some(Token::Char('a')));

        let mut tokenizer = Tokenizer::new("#\\space");
        assert_eq!(tokenizer.next(), Some(Token::Char(' ')));

        let mut tokenizer = Tokenizer::new("#\\newline");
        assert_eq!(tokenizer.next(), Some(Token::Char('\n')));

        let mut tokenizer = Tokenizer::new("#\\tab");
        assert_eq!(tokenizer.next(), Some(Token::Char('\t')));
    }
}
