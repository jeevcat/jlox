use anyhow::anyhow;
use log::error;

use crate::scanner::{Token, TokenType};

pub fn make_error(token: &Token, message: &str) -> anyhow::Error {
    match token.token_type {
        TokenType::Eof => (anyhow!("{} at end", message)),
        _ => (anyhow!("{} at '{}'", message, token.lexeme)),
    }
}

pub fn report_error(token: &Token, message: &str) {
    match token.token_type {
        TokenType::Eof => (error!("{} at end", message)),
        _ => {
            (error!(
                "[line {}, col {}] {} at '{}'",
                token.line, token.col, message, token.lexeme
            ))
        }
    }
}
