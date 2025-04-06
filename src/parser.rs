//Import Token from lexer and Expr and BinOp from the AST module
use crate::lexer::Token;
use crate::ast::{Expr, BinOp};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    //Constructor for Parser
    pub fn new(tokens: Vec<Token>) -> Self {
        //Initialise parser with tokens and position at 0
        Self { tokens, pos: 0 }
    }

    //returns the current token without changing position
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    //consumes the current token and moves to the next one
    fn eat(&mut self) -> Option<Token> {
        //get current token and clone it
        let tok = self.tokens.get(self.pos).cloned();
        self.pos += 1;
        tok
    }

    //parses an exit(...) expression
    pub fn parse_exit_expr(&mut self) -> Option<Expr> {
        match self.eat() {
            Some(Token::Ident(ref s)) if s == "exit" => {
                match self.eat() {
                    Some(Token::LParen) => {
                        //parse the expression inside exit()
                        let expr = self.parse_expr()?;
                        match self.eat() {
                            Some(Token::RParen) => {
                                match self.eat() {
                                    //return parsed expr if semicolon found
                                    Some(Token::Semicolon) => Some(expr),
                                    _ => None  //no semicolon-> invalid expression
                                }
                            }
                            _ => None,  //no closing parenthesis -> invalid expression
                        }
                    }
                    _ => None,  //no opening parenthesis -> invalid expression
                }
            }
            _ => None,  //not "exit" -> invalid expression
        }
    }

    //parses an expression (just calls parse_term for now)
    fn parse_expr(&mut self) -> Option<Expr> {
        self.parse_term()
    }

    //parses terms (multiplication, division, addition, subtraction)
    fn parse_term(&mut self) -> Option<Expr> {
        //parse the first factor
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
                    //creates a binary operation node
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

    //parses factors (multiplication, division)
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
                    //creates a binary operation node
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

    //parses a primary value
    fn parse_primary(&mut self) -> Option<Expr> {
        match self.eat()? {
            Token::Number(n) => Some(Expr::Number(n)),
            //if its an opening parenthesis -> parse the expression inside
            Token::LParen => {
                let expr = self.parse_expr()?;
                match self.eat()? {
                    //return the parsed expression inside the parentheses
                    Token::RParen => Some(expr),
                    //no closing parenthesis, invalid expression
                    _ => None,
                }
            }
            _ => None,
        }
    }
}
