use std::cell::Cell;

use anyhow::{anyhow, Result};
use log::error;

use crate::{
    ast::{
        expr::{Expr, Literal},
        stmt::{FunctionDecl, Stmt},
    },
    error::make_error,
    scanner::{Token, TokenType},
};

pub struct Parser {
    tokens: Vec<Token>,
    current: Cell<usize>,
}

impl Parser {
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
        Err(make_error(self.peek(), message))
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
        // Similar to using consume_matching(), but using match. Need to make sure we
        // call advance manually though.
        let result = match self.peek().token_type {
            TokenType::Var => {
                self.advance();
                self.variable_declaration()
            }
            TokenType::Fun => {
                self.advance();
                self.function_declaration("function")
            }
            _ => self.statement(),
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
        Ok(Stmt::VarDecl {
            name: name.clone(),
            initializer,
        })
    }

    fn function_declaration(&self, kind: &str) -> Result<Stmt> {
        let name = self
            .consume(&TokenType::Identifier, &format!("Expect {} name", kind))?
            .clone();
        self.consume(
            &TokenType::LeftParen,
            &format!("Expect '(' after {} name", kind),
        )?;

        let mut params = vec![];
        if !self.check(&TokenType::RightParen) {
            let mut first = true;
            while first || self.consume_matching(&[TokenType::Comma]).is_some() {
                if params.len() >= 255 {
                    make_error(self.peek(), "Can't have more than 255 parameters");
                }
                params.push(
                    self.consume(&TokenType::Identifier, "Expect parameter name")?
                        .clone(),
                );
                first = false;
            }
        }
        self.consume(&TokenType::RightParen, "Expect ')' after parameters")?;
        self.consume(
            &TokenType::LeftBrace,
            &format!("Expect '{{' before {} body", kind),
        )?;
        let body = self.block()?;

        Ok(Stmt::FunctionDecl(FunctionDecl { name, params, body }))
    }

    fn statement(&self) -> Result<Stmt> {
        // Similar to using consume_matching(), but using match. Need to make sure we
        // call advance manually though.
        match self.peek().token_type {
            TokenType::For => {
                self.advance();
                self.for_statement()
            }
            TokenType::If => {
                self.advance();
                self.if_statement()
            }
            TokenType::LeftBrace => {
                self.advance();
                Ok(Stmt::Block(self.block()?))
            }
            TokenType::Print => {
                self.advance();
                self.print_statement()
            }
            TokenType::Return => {
                self.advance();
                self.return_statement()
            }
            TokenType::While => {
                self.advance();
                self.while_statement()
            }
            _ => self.expression_statement(),
        }
    }

    fn for_statement(&self) -> Result<Stmt> {
        self.consume(&TokenType::LeftParen, "Expect '(' after 'for'")?;

        let initializer = match self.peek().token_type {
            TokenType::Semicolon => {
                self.advance();
                None
            }
            TokenType::Var => {
                self.advance();
                Some(self.variable_declaration()?)
            }
            _ => Some(self.expression_statement()?),
        };

        let condition = match self.peek().token_type {
            // If coniditon is omitted, we jam in `true` to make an infinite loop
            TokenType::Semicolon => Expr::Literal(Literal::True),
            _ => self.expression()?,
        };
        self.consume(&TokenType::Semicolon, "Expect ';' after loop condition")?;

        let increment = match self.peek().token_type {
            TokenType::RightParen => None,
            _ => Some(self.expression()?),
        };
        self.consume(&TokenType::RightParen, "Expect ')' after for clauses")?;

        let mut body = self.statement()?;

        // Desugar to while loop
        if let Some(increment) = increment {
            // Replace the body with a little block that contains the original body followed
            // by an expression statement that evaluates the increment
            body = Stmt::Block(vec![body, Stmt::Expression(increment)]);
        }

        body = Stmt::While {
            condition,
            body: Box::new(body),
        };

        if let Some(initializer) = initializer {
            // Replace the whole statement with a block that runs the initializer and then
            // executes the loop
            body = Stmt::Block(vec![initializer, body]);
        }

        Ok(body)
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

    fn return_statement(&self) -> Result<Stmt> {
        let value = if !self.check(&TokenType::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(&TokenType::Semicolon, "Expect ';' after return value")?;
        Ok(Stmt::Return {
            value,
            keyword: self.peek().clone(),
        })
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
                    make_error(equals, "Invalid assignment target");
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
                operator: operator.token_type.clone(),
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
                operator: operator.token_type.clone(),
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
                operator: operator.token_type.clone(),
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
                operator: operator.token_type.clone(),
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
                operator: operator.token_type.clone(),
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
                operator: operator.token_type.clone(),
                right,
            };
        }
        Ok(expr)
    }

    fn unary(&self) -> Result<Expr> {
        if let Some(operator) = self.consume_matching(&[TokenType::Bang, TokenType::Minus]) {
            let right = Box::new(self.unary()?);
            return Ok(Expr::Unary {
                operator: operator.token_type.clone(),
                right,
            });
        }
        self.call()
    }

    fn call(&self) -> Result<Expr> {
        let mut expr = self.primary()?;

        loop {
            if self.consume_matching(&[TokenType::LeftParen]).is_some() {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&self, callee: Expr) -> Result<Expr> {
        let mut arguments = vec![];
        if !self.check(&TokenType::RightParen) {
            let mut first = true;
            while first || self.consume_matching(&[TokenType::Comma]).is_some() {
                if arguments.len() >= 255 {
                    make_error(self.peek(), "Can't have more thant 255 arguments");
                }
                arguments.push(self.expression()?);
                first = false;
            }
        }

        let _paren = self.consume(&TokenType::RightParen, "Expect ')' after arguments")?;

        Ok(Expr::Call {
            callee: Box::new(callee),
            //paren,
            arguments,
        })
    }

    fn primary(&self) -> Result<Expr> {
        let token = self.advance();
        match &token.token_type {
            TokenType::LeftParen => {
                let expr = self.expression()?;
                self.consume(&TokenType::RightParen, "Expect ')' after expression")?;
                Ok(Expr::Grouping(Box::new(expr)))
            }
            TokenType::String(s) => Ok(Expr::Literal(Literal::String(s.clone()))),
            TokenType::Number(n) => Ok(Expr::Literal(Literal::Number(*n))),
            TokenType::False => Ok(Expr::Literal(Literal::False)),
            TokenType::Nil => Ok(Expr::Literal(Literal::Nil)),
            TokenType::True => Ok(Expr::Literal(Literal::True)),
            TokenType::Identifier => Ok(Expr::Variable {
                name: token.clone(),
            }),
            _ => Err(make_error(self.peek(), "Expect expression")),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{ast::stmt::Stmt, parser::Parser, scanner::scan_tokens};

    #[test]
    fn parse() {
        let input = "print (1 + 2 * -3 - 4);";
        let tokens = scan_tokens(input).unwrap();
        let parser = Parser::new(tokens);
        let statements = parser.parse().unwrap();
        assert!(matches!(statements[0], Stmt::Print(_)));
    }
}
