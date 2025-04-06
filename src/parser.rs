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

    // Returns the current token without advancing the position
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    // Consumes the current token and advances to the next one
    fn eat(&mut self) -> Option<Token> {
        // Gets the current token and clones it
        let tok = self.tokens.get(self.pos).cloned();
        self.pos += 1;
        tok
    }

    // Parsing a console.print expression (console.print("test");)
    pub fn parse_console_print_expr(&mut self) -> Option<Expr> {
        match self.eat() {
            Some(Token::Ident(ref s)) if s == "console" => {
                match self.eat() {
                    Some(Token::Dot) => {
                        match self.eat() {
                            Some(Token::Print) => {
                                match self.eat() {
                                    Some(Token::LParen) => {
                                        // Parses the expression inside print()
                                        let expr = self.parse_expr()?;
                                        match self.eat() {
                                            Some(Token::RParen) => {
                                                match self.eat() {
                                                    // Returns the parsed expression if a semicolon is found
                                                    Some(Token::Semicolon) => Some(Expr::Print(Box::new(expr))),
                                                    _ => None, // Missing semicolon
                                                }
                                            }
                                            _ => None, // Missing closing parenthesis
                                        }
                                    }
                                    _ => None, // Missing opening parenthesis
                                }
                            }
                            _ => None, // Not a print keyword
                        }
                    }
                    _ => None, // Missing dot after console
                }
            }
            _ => None, // Not a console.print expression
        }
    }

    // Parsing an exit(...) expression
    pub fn parse_exit_expr(&mut self) -> Option<Expr> {
        match self.eat() {
            Some(Token::Ident(ref s)) if s == "exit" => {
                match self.eat() {
                    Some(Token::LParen) => {
                        // Parses the expression inside exit()
                        let expr = self.parse_expr()?;
                        match self.eat() {
                            Some(Token::RParen) => {
                                match self.eat() {
                                    // Returns the parsed expression if a semicolon is found
                                    Some(Token::Semicolon) => Some(expr),
                                    _ => None  // No semicolon -> invalid expression
                                }
                            }
                            _ => None,  // No closing parenthesis -> invalid expression
                        }
                    }
                    _ => None,  // No opening parenthesis -> invalid expression
                }
            }
            _ => None,  // Not an exit expression
        }
    }

    // Parsing a general expression
    fn parse_expr(&mut self) -> Option<Expr> {
        self.parse_term()
    }

    // Parsing terms (multiplication, division, addition, subtraction)
    fn parse_term(&mut self) -> Option<Expr> {
        // Parse the first factor
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
                    // Creates a binary operand
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
                    // Creates a binary operand
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
            // If it's an opening parenthesis -> parses the expression inside
            Token::LParen => {
                let expr = self.parse_expr()?;
                match self.eat()? {
                    // Returns the parsed expression inside the parentheses
                    Token::RParen => Some(expr),
                    // Missing closing parenthesis -> invalid expression
                    _ => None,
                }
            }
            _ => None,
        }
    }
}
