use crate::lexer::Token;
use crate::ast::{Expr, BinOp};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn eat(&mut self) -> Option<Token> {
        let tok = self.tokens.get(self.pos).cloned();
        self.pos += 1;
        tok
    }

    pub fn parse_exit_expr(&mut self) -> Option<Expr> {
        match self.eat() {
            Some(Token::Ident(ref s)) if s == "exit" => {
                match self.eat() {
                    Some(Token::LParen) => {
                        let expr = self.parse_expr()?;
                        match self.eat() {
                            Some(Token::RParen) => {
                                match self.eat() {
                                    Some(Token::Semicolon) => Some(expr),
                                    _ => None
                                }
                            }
                            _ => None,
                        }
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn parse_expr(&mut self) -> Option<Expr> {
        self.parse_term()
    }

    fn parse_term(&mut self) -> Option<Expr> {
        let mut node = self.parse_factor()?;

        while let Some(op) = self.peek() {
            match op {
                Token::Plus | Token::Minus => {
                    let op = match self.eat()? {
                        Token::Plus => BinOp::Add,
                        Token::Minus => BinOp::Sub,
                        _ => unreachable!(),
                    };
                    let right = self.parse_factor()?;
                    node = Expr::BinaryOp {
                        op,
                        left: Box::new(node),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        Some(node)
    }

    fn parse_factor(&mut self) -> Option<Expr> {
        let mut node = self.parse_primary()?;

        while let Some(op) = self.peek() {
            match op {
                Token::Star | Token::Slash => {
                    let op = match self.eat()? {
                        Token::Star => BinOp::Mul,
                        Token::Slash => BinOp::Div,
                        _ => unreachable!(),
                    };
                    let right = self.parse_primary()?;
                    node = Expr::BinaryOp {
                        op,
                        left: Box::new(node),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        Some(node)
    }

    fn parse_primary(&mut self) -> Option<Expr> {
        match self.eat()? {
            Token::Number(n) => Some(Expr::Number(n)),
            Token::LParen => {
                let expr = self.parse_expr()?;
                match self.eat()? {
                    Token::RParen => Some(expr),
                    _ => None,
                }
            }
            _ => None,
        }
    }
}
