use anyhow::anyhow;

use crate::scanner::{Token, TokenType};

pub fn error(token: &Token, message: &str) -> anyhow::Error {
    match token.token_type {
        TokenType::Eof => (anyhow!("{} at end", message)),
        _ => (anyhow!("{} at '{}'", message, token.lexeme)),
    }
}
