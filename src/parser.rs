use anyhow::{anyhow, Result};
use log::error;
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
            if let Some(statement) = self.declaration() {
                statements.push(statement);
            }
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

    fn synchronize(&self) {
        let mut prev = self.advance();

        while !self.is_at_end() {
            if matches!(prev.token_type, TokenType::Semicolon) {
                return;
            }

            match self.peek().token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => {}
            }

            prev = self.advance();
        }
    }

    // TODO: should this be returning an option?
    fn declaration(&self) -> Option<Stmt> {
        let result = if self.consume_matching(&[TokenType::Var]).is_some() {
            self.variable_declaration()
        } else {
            self.statement()
        };
        match result {
            Ok(s) => Some(s),
            Err(e) => {
                error!("{}", e);
                self.synchronize();
                None
            }
        }
    }

    fn variable_declaration(&self) -> Result<Stmt> {
        let name = self.consume(&TokenType::Identifier, "Expect variable name")?;

        let initializer = if self.consume_matching(&[TokenType::Equal]).is_some() {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(
            &TokenType::Semicolon,
            "Expect ';' after variable declaration",
        )?;
        Ok(Stmt::Var {
            name: name.clone(),
            initializer,
        })
    }

    fn statement(&self) -> Result<Stmt> {
        // Similar to using consume_matching(), but using match. Need to make sure we call advance manually though.
        match self.peek().token_type {
            TokenType::If => {
                self.advance();
                self.if_statement()
            }
            TokenType::While => {
                self.advance();
                self.while_statement()
            }
            TokenType::Print => {
                self.advance();
                self.print_statement()
            }
            TokenType::LeftBrace => {
                self.advance();
                Ok(Stmt::Block(self.block()?))
            }
            _ => self.expression_statement(),
        }
    }

    fn if_statement(&self) -> Result<Stmt> {
        self.consume(&TokenType::LeftParen, "Expect '(' after 'if'")?;
        let condition = self.expression()?;
        self.consume(&TokenType::RightParen, "Expect ')' after if condition")?;
        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.consume_matching(&[TokenType::Else]).is_some() {
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn while_statement(&self) -> Result<Stmt> {
        self.consume(&TokenType::LeftParen, "Expect '(' after 'while'")?;
        let condition = self.expression()?;
        self.consume(&TokenType::RightParen, "Expect ')' after if condition")?;
        let body = Box::new(self.statement()?);
        Ok(Stmt::While { condition, body })
    }

    fn print_statement(&self) -> Result<Stmt> {
        let value = self.expression()?;
        self.consume(&TokenType::Semicolon, "Expect ';' after value")?;
        Ok(Stmt::Print(value))
    }

    fn block(&self) -> Result<Vec<Stmt>> {
        let mut statements = vec![];

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            if let Some(declaration) = self.declaration() {
                statements.push(declaration);
            }
        }

        self.consume(&TokenType::RightBrace, "Expect '}' after block")?;
        Ok(statements)
    }

    fn expression_statement(&self) -> Result<Stmt> {
        let value = self.expression()?;
        self.consume(&TokenType::Semicolon, "Expect ';' after expression")?;
        Ok(Stmt::Expression(value))
    }

    fn expression(&self) -> Result<Expr> {
        self.assignment()
    }

    fn assignment(&self) -> Result<Expr> {
        let expr = self.or()?;

        if let Some(equals) = self.consume_matching(&[TokenType::Equal]) {
            let value = self.assignment()?;

            match expr {
                Expr::Variable { name } => {
                    return Ok(Expr::Assign {
                        name,
                        value: Box::new(value),
                    })
                }
                _ => {
                    self.error(equals, "Invalid assignment target");
                }
            }
        }
        Ok(expr)
    }

    fn or(&self) -> Result<Expr> {
        let mut expr = self.and()?;
        while let Some(operator) = self.consume_matching(&[TokenType::Or]) {
            let right = Box::new(self.and()?);
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: operator.clone(),
                right,
            };
        }
        Ok(expr)
    }

    fn and(&self) -> Result<Expr> {
        let mut expr = self.equality()?;
        while let Some(operator) = self.consume_matching(&[TokenType::Or]) {
            let right = Box::new(self.equality()?);
            expr = Expr::Logical {
                left: Box::new(expr),
                operator: operator.clone(),
                right,
            };
        }
        Ok(expr)
    }

    fn equality(&self) -> Result<Expr> {
        let mut expr = self.comparison()?;
        while let Some(operator) =
            self.consume_matching(&[TokenType::BangEqual, TokenType::EqualEqual])
        {
            let right = Box::new(self.comparison()?);
            expr = Expr::Logical {
                left: Box::new(expr),
                operator: operator.clone(),
                right,
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
            let right = Box::new(self.term()?);
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: operator.clone(),
                right,
            };
        }
        Ok(expr)
    }

    fn term(&self) -> Result<Expr> {
        let mut expr = self.factor()?;
        while let Some(operator) = self.consume_matching(&[TokenType::Minus, TokenType::Plus]) {
            let right = Box::new(self.factor()?);
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: operator.clone(),
                right,
            };
        }
        Ok(expr)
    }

    fn factor(&self) -> Result<Expr> {
        let mut expr = self.unary()?;
        while let Some(operator) = self.consume_matching(&[TokenType::Slash, TokenType::Star]) {
            let right = Box::new(self.unary()?);
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: operator.clone(),
                right,
            };
        }
        Ok(expr)
    }

    fn unary(&self) -> Result<Expr> {
        if let Some(operator) = self.consume_matching(&[TokenType::Bang, TokenType::Minus]) {
            let right = Box::new(self.unary()?);
            return Ok(Expr::Unary {
                operator: operator.clone(),
                right,
            });
        }
        self.primary()
    }

    fn primary(&self) -> Result<Expr> {
        let token = self.advance();
        match &token.token_type {
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
            TokenType::Identifier => Ok(Expr::Variable {
                name: token.clone(),
            }),
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
