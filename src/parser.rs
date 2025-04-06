use crate::lexer::Token;
use crate::ast::{Expr, BinOp};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    // Constructor for Parser
    pub fn new(tokens: Vec<Token>) -> Self {
        // Initialize the parser with tokens and starting position at 0
        Self { tokens, pos: 0 }
    }

    // Public accessor for current position (for error messages)
    pub fn pos(&self) -> usize {
        self.pos
    }

    // Returns true if all tokens have been consumed
    pub fn is_finished(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    // Returns the current token without advancing the position
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    // Consumes the current token and advances to the next one
    fn eat(&mut self) -> Option<Token> {
        let tok = self.tokens.get(self.pos).cloned();
        self.pos += 1;
        tok
    }

    // Parsing a console.print expression (console.print("test");)
    pub fn parse_console_print_expr(&mut self) -> Option<Expr> {
        // Peek without consuming: check if the next token is Ident("console")
        if let Some(Token::Ident(s)) = self.peek() {
            if s != "console" {
                return None;
            }
        } else {
            return None;
        }
        // Now that we know it's a console.print, consume "console"
        self.eat();

        // Next, expect a Dot token
        if let Some(Token::Dot) = self.peek() {
            self.eat();
        } else {
            return None;
        }

        // Next, expect the Print keyword
        if let Some(Token::Print) = self.peek() {
            self.eat();
        } else {
            return None;
        }

        // Next, expect an opening parenthesis
        if let Some(Token::LParen) = self.peek() {
            self.eat();
        } else {
            return None;
        }

        // Parse the expression inside print()
        let expr = self.parse_expr()?;

        // Expect a closing parenthesis
        if let Some(Token::RParen) = self.peek() {
            self.eat();
        } else {
            return None;
        }

        // Expect a semicolon
        if let Some(Token::Semicolon) = self.peek() {
            self.eat();
            return Some(Expr::Print(Box::new(expr)));
        }

        None
    }

    // Parsing an exit(...) expression
    pub fn parse_exit_expr(&mut self) -> Option<Expr> {
        // Peek without consuming: check if the next token is Ident("exit")
        if let Some(Token::Ident(s)) = self.peek() {
            if s != "exit" {
                return None;
            }
        } else {
            return None;
        }
        // Consume "exit"
        self.eat();

        // Expect an opening parenthesis
        if let Some(Token::LParen) = self.peek() {
            self.eat();
        } else {
            return None;
        }

        // Parse the expression inside exit()
        let expr = self.parse_expr()?;

        // Expect a closing parenthesis
        if let Some(Token::RParen) = self.peek() {
            self.eat();
        } else {
            return None;
        }

        // Expect a semicolon
        if let Some(Token::Semicolon) = self.peek() {
            self.eat();
            return Some(Expr::Exit(Box::new(expr)));
        }

        None
    }


    // Parsing a general expression
    fn parse_expr(&mut self) -> Option<Expr> {
        self.parse_term()
    }

    // Parsing terms (multiplication, division, addition, subtraction)
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

    // Parsing factors (multiplication, division)
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

    // Parsing a primary expression (number, string literal, parenthesis)
    fn parse_primary(&mut self) -> Option<Expr> {
        match self.eat()? {
            Token::Number(n) => Some(Expr::Number(n)),
            Token::StringLiteral(s) => Some(Expr::StringLiteral(s)),
            Token::LParen => {
                let expr = self.parse_expr()?;
                if let Some(Token::RParen) = self.peek() {
                    self.eat();
                    Some(expr)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
