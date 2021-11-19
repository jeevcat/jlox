use anyhow::{anyhow, Result};
use std::cell::Cell;

use crate::{
    expr::{Expr, Literal},
    scanner::{Token, TokenType},
    stmt::Stmt,
};

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    current: Cell<usize>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens,
            current: Cell::new(0),
        }
    }

    pub fn parse(&self) -> Result<Vec<Stmt>> {
        if self.tokens.is_empty() {
            return Err(anyhow!("No tokens provided"));
        }

        let mut statements = vec![];
        while !self.is_at_end() {
            statements.push(self.statement()?);
        }
        Ok(statements)
    }

    fn advance(&self) -> &Token {
        let prev = self.peek();
        if !self.is_at_end() {
            self.current.set(self.current.get() + 1);
        }
        prev
    }

    fn consume_matching(&self, token_types: &[TokenType]) -> Option<&Token> {
        for token_type in token_types {
            if self.check(token_type) {
                return Some(self.advance());
            }
        }
        None
    }

    fn consume(&self, token_type: &TokenType, message: &str) -> Result<&Token> {
        if self.check(token_type) {
            return Ok(self.advance());
        }
        Err(self.error(self.peek(), message))
    }

    fn error(&self, token: &Token, message: &str) -> anyhow::Error {
        match token.token_type {
            TokenType::Eof => (anyhow!("{} at end", message)),
            _ => (anyhow!("{} at '{}'", message, self.peek().lexeme)),
        }
    }

    fn check(&self, token_type: &TokenType) -> bool {
        &self.peek().token_type == token_type
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().token_type, TokenType::Eof)
    }

    fn peek(&self) -> &Token {
        // advance() won't allow going past end, so this is safe
        &self.tokens[self.current.get()]
    }

    fn statement(&self) -> Result<Stmt> {
        if self.consume_matching(&[TokenType::Print]).is_some() {
            return self.print_statement();
        }
        self.expression_statement()
    }

    fn print_statement(&self) -> Result<Stmt> {
        let value = self.expression()?;
        self.consume(&TokenType::Semicolon, "Expect ';' after value")?;
        Ok(Stmt::Print(value))
    }

    fn expression_statement(&self) -> Result<Stmt> {
        let value = self.expression()?;
        self.consume(&TokenType::Semicolon, "Expect ';' after expression")?;
        Ok(Stmt::Expression(value))
    }

    fn expression(&self) -> Result<Expr> {
        self.equality()
    }

    fn equality(&self) -> Result<Expr> {
        let mut expr = self.comparison()?;
        while let Some(operator) =
            self.consume_matching(&[TokenType::BangEqual, TokenType::EqualEqual])
        {
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: operator.clone(),
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn comparison(&self) -> Result<Expr> {
        let mut expr = self.term()?;
        while let Some(operator) = self.consume_matching(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: operator.to_owned(),
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn term(&self) -> Result<Expr> {
        let mut expr = self.factor()?;
        while let Some(operator) = self.consume_matching(&[TokenType::Minus, TokenType::Plus]) {
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: operator.to_owned(),
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn factor(&self) -> Result<Expr> {
        let mut expr = self.unary()?;
        while let Some(operator) = self.consume_matching(&[TokenType::Slash, TokenType::Star]) {
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: operator.to_owned(),
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn unary(&self) -> Result<Expr> {
        if let Some(operator) = self.consume_matching(&[TokenType::Bang, TokenType::Minus]) {
            let right = self.unary()?;
            return Ok(Expr::Unary {
                operator: operator.to_owned(),
                right: Box::new(right),
            });
        }
        self.primary()
    }

    fn primary(&self) -> Result<Expr> {
        match &self.advance().token_type {
            TokenType::LeftParen => {
                let expr = self.expression()?;
                self.consume(&TokenType::RightParen, "Expect ')' after expression")?;
                Ok(Expr::Grouping(Box::new(expr)))
            }
            TokenType::String(s) => Ok(Expr::Literal(Literal::String(s))),
            TokenType::Number(n) => Ok(Expr::Literal(Literal::Number(*n))),
            TokenType::False => Ok(Expr::Literal(Literal::False)),
            TokenType::Nil => Ok(Expr::Literal(Literal::Nil)),
            TokenType::True => Ok(Expr::Literal(Literal::True)),
            _ => Err(self.error(self.peek(), "Expect expression")),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{parser::Parser, scanner::scan_tokens, stmt::Stmt};

    #[test]
    fn parse() {
        let input = "print (1 + 2 * -3 - 4);";
        let tokens = scan_tokens(input).unwrap();
        let parser = Parser::new(tokens);
        let statements = parser.parse().unwrap();
        assert!(matches!(statements[0], Stmt::Print(_)));
    }
}
