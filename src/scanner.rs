use std::fmt;

use anyhow::{anyhow, Result};
use phf::phf_map;

static KEYWORDS: phf::Map<&'static str, TokenType> = phf_map! {
    "and" => TokenType::And,
    "class" => TokenType::Class,
    "else" => TokenType::Else,
    "false" => TokenType::False,
    "for" => TokenType::For,
    "fun" => TokenType::Fun,
    "if" => TokenType::If,
    "nil" => TokenType::Nil,
    "or" => TokenType::Or,
    "print" => TokenType::Print,
    "return" => TokenType::Return,
    "super" => TokenType::Super,
    "this" => TokenType::This,
    "true" => TokenType::True,
    "var" => TokenType::Var,
    "while" => TokenType::While,
};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String(String),
    Number(f64),

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

pub struct Token<'a> {
    pub token_type: TokenType,
    pub lexeme: &'a str,
    pub line: usize,
    pub col: u32,
}

impl<'a> fmt::Debug for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Token {{ ty: {:?}, lexeme: \"{}\", line: {:?}, col: {:?}}}",
            self.token_type, self.lexeme, self.line, self.col
        )
    }
}

pub fn scan_tokens(input: &str) -> Result<Vec<Token>> {
    let mut scanner = Scanner {
        source: input,
        tokens: vec![],
        start: 0,
        current: 0,
        line: 1,
        col: 0,
    };

    scanner.scan_tokens()?;

    Ok(scanner.tokens)
}

struct Scanner<'a> {
    source: &'a str,
    tokens: Vec<Token<'a>>,
    start: usize,
    current: usize,
    line: usize,
    col: u32,
}

impl<'a> Scanner<'a> {
    fn scan_tokens(&mut self) -> Result<()> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }

        self.tokens.push(Token {
            token_type: TokenType::Eof,
            lexeme: "",
            line: self.line,
            col: self.col,
        });

        Ok(())
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.col += 1;

        char::from(self.source.as_bytes()[self.current - 1])
    }

    fn scan_token(&mut self) -> Result<()> {
        let c = self.advance();

        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            '[' => self.add_token(TokenType::LeftBracket),
            ']' => self.add_token(TokenType::RightBracket),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),
            '!' => {
                let matches_eq = self.matches('=');
                self.add_token(if matches_eq {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                })
            }
            '=' => {
                let matches_eq = self.matches('=');
                self.add_token(if matches_eq {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                })
            }
            '<' => {
                let matches_eq = self.matches('=');
                self.add_token(if matches_eq {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                })
            }
            '>' => {
                let matches_eq = self.matches('=');
                self.add_token(if matches_eq {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                })
            }
            '/' => {
                if self.matches('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash)
                }
            }
            ' ' | '\r' | '\t' => {}
            '\n' => {
                self.line += 1;
                self.col = 0
            }
            '"' => self.string()?,
            _ => {
                if Scanner::is_decimal_digit(c) {
                    self.number()
                } else if Scanner::is_alpha(c) {
                    self.identifier()
                } else {
                    return Err(anyhow!("scanner can't handle {}", c));
                }
            }
        }
        Ok(())
    }

    fn is_alpha(c: char) -> bool {
        c.is_alphabetic()
    }

    fn is_decimal_digit(c: char) -> bool {
        c.is_digit(10)
    }

    fn is_alphanumeric(c: char) -> bool {
        Scanner::is_alpha(c) || Scanner::is_decimal_digit(c)
    }

    fn identifier(&mut self) {
        while Scanner::is_alphanumeric(self.peek()) {
            self.advance();
        }

        let literal_val = &self.source[self.start..self.current];

        match KEYWORDS.get(literal_val) {
            Some(kw_token_type) => self.add_token(kw_token_type.to_owned()),
            None => self.add_token(TokenType::Identifier),
        };
    }

    fn number(&mut self) {
        while Scanner::is_decimal_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' && Scanner::is_decimal_digit(self.peek_next()) {
            self.advance();
        }

        while Scanner::is_decimal_digit(self.peek()) {
            self.advance();
        }

        let val: f64 = self.source[self.start..self.current].parse().unwrap();

        self.add_token(TokenType::Number(val));
    }

    fn string(&mut self) -> Result<()> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1
            }
            self.advance();
        }

        if self.is_at_end() {
            return Err(anyhow!("Unterminated string at line {}", self.line));
        }

        assert!(self.peek() == '"');

        self.advance();

        self.add_token(TokenType::String(
            self.source[self.start + 1..self.current - 1].to_string(),
        ));
        Ok(())
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            char::from(self.source.as_bytes()[self.current + 1])
        }
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            char::from(self.source.as_bytes()[self.current])
        }
    }

    fn matches(&mut self, c: char) -> bool {
        if self.is_at_end() {
            return true;
        }

        if char::from(self.source.as_bytes()[self.current]) != c {
            return false;
        }

        self.current += 1;
        self.col += 1;
        true
    }

    fn add_token(&mut self, token_type: TokenType) {
        let text = &self.source[self.start..self.current];

        self.tokens.push(Token {
            token_type,
            lexeme: text,
            line: self.line,
            col: self.col,
        })
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}

#[cfg(test)]
mod tests {
    use super::scan_tokens;

    #[test]
    fn it_works() {
        let tokens = scan_tokens("  ({}) ").unwrap();
        assert_eq!(tokens[0].lexeme, "(");
    }
}
