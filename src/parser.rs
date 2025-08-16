use crate::ast::{Expr, BinOp};
use crate::lexer::Token;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    pub fn is_finished(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn parse_let_declaration(&mut self) -> Option<Expr> {
        // Check for let keyword
        if self.is_finished() || !matches!(&self.tokens[self.pos], Token::Identifier(id) if id == "let") {
            return None;
        }
        self.pos += 1;
        
        // Get variable name
        if self.is_finished() || !matches!(&self.tokens[self.pos], Token::Identifier(_)) {
            return None;
        }
        let name = match &self.tokens[self.pos] {
            Token::Identifier(id) => id.clone(),
            _ => unreachable!()
        };
        self.pos += 1;
        
        // Expect '='
        if self.is_finished() || self.tokens[self.pos] != Token::Assign {
            return None;
        }
        self.pos += 1;
        
        // Parse the value 
        let value = match self.parse_expression() {
            Some(expr) => expr,
            None => return None,
        };
        
        // Expect ';'
        if self.is_finished() || self.tokens[self.pos] != Token::Semicolon {
            return None;
        }
        self.pos += 1;
        
        Some(Expr::Let {
            name,
            value: Box::new(value),
        })
    }

    pub fn parse_assignment(&mut self) -> Option<Expr> {
        // Only proceed if the current token is an identifier
        if self.is_finished() || !matches!(&self.tokens[self.pos], Token::Identifier(_)) {
            return None;
        }
        
        // Save the current position to backtrack if this isnt an assignment
        let start_pos = self.pos;
        
        // Get variable name
        let name = match &self.tokens[self.pos] {
            Token::Identifier(id) => id.clone(),
            _ => unreachable!()
        };
        self.pos += 1;
        
        // Check for '='
        if self.is_finished() || self.tokens[self.pos] != Token::Assign {
            // Not an assignment, backtrack
            self.pos = start_pos;
            return None;
        }
        self.pos += 1;
        
        // Parse the value
        let value = match self.parse_expression() {
            Some(expr) => expr,
            None => {
                self.pos = start_pos;
                return None;
            },
        };
        
        // Expect ';'
        if self.is_finished() || self.tokens[self.pos] != Token::Semicolon {
            self.pos = start_pos;
            return None;
        }
        self.pos += 1;
        
        Some(Expr::Assign {
            name,
            value: Box::new(value),
        })
    }

    // Parse a constant declaration: const name = value;
    pub fn parse_const_declaration(&mut self) -> Option<Expr> {
        // Check for 'const' keyword
        if self.is_finished() || !matches!(&self.tokens[self.pos], Token::Identifier(id) if id == "const") {
            return None;
        }
        self.pos += 1;
        
        // Get variable name
        if self.is_finished() || !matches!(&self.tokens[self.pos], Token::Identifier(_)) {
            return None;
        }
        let name = match &self.tokens[self.pos] {
            Token::Identifier(id) => id.clone(),
            _ => unreachable!()
        };
        self.pos += 1;
        
        // Expect '='
        if self.is_finished() || self.tokens[self.pos] != Token::Assign {
            return None;
        }
        self.pos += 1;
        
        // Parse the value (can be any expression)
        let value = match self.parse_expression() {
            Some(expr) => expr,
            None => return None,
        };
        
        // Expect ';'
        if self.is_finished() || self.tokens[self.pos] != Token::Semicolon {
            return None;
        }
        self.pos += 1;
        
        Some(Expr::Const {
            name,
            value: Box::new(value),
        })
    }

    // Parse a console.print statement: console.print(expr);
    pub fn parse_console_print_expr(&mut self) -> Option<Expr> {
        // Check for 'console'
        if self.is_finished() || !matches!(&self.tokens[self.pos], Token::Identifier(id) if id == "console") {
            return None;
        }
        self.pos += 1;
        
        // Expect '.'
        if self.is_finished() || self.tokens[self.pos] != Token::Dot {
            return None;
        }
        self.pos += 1;
        
        // Expect 'print'
        if self.is_finished() || !matches!(&self.tokens[self.pos], Token::Identifier(id) if id == "print") {
            return None;
        }
        self.pos += 1;
        
        // Expect '('
        if self.is_finished() || self.tokens[self.pos] != Token::LParen {
            return None;
        }
        self.pos += 1;
        
        // Parse the expression to print
        let expr = match self.parse_expression() {
            Some(expr) => expr,
            None => return None,
        };
        
        // Expect ')'
        if self.is_finished() || self.tokens[self.pos] != Token::RParen {
            return None;
        }
        self.pos += 1;
        
        // Expect ';'
        if self.is_finished() || self.tokens[self.pos] != Token::Semicolon {
            return None;
        }
        self.pos += 1;
        
        Some(Expr::Print(Box::new(expr)))
    }

    // Parse an exit statement: exit(expr);
    pub fn parse_exit_expr(&mut self) -> Option<Expr> {
        // Check for 'exit'
        if self.is_finished() || !matches!(&self.tokens[self.pos], Token::Identifier(id) if id == "exit") {
            return None;
        }
        self.pos += 1;
        
        // Expect '('
        if self.is_finished() || self.tokens[self.pos] != Token::LParen {
            return None;
        }
        self.pos += 1;
        
        // Parse the exit code
        let expr = match self.parse_expression() {
            Some(expr) => expr,
            None => return None,
        };
        
        // Expect ')'
        if self.is_finished() || self.tokens[self.pos] != Token::RParen {
            return None;
        }
        self.pos += 1;
        
        // Expect ';'
        if self.is_finished() || self.tokens[self.pos] != Token::Semicolon {
            return None;
        }
        self.pos += 1;
        
        Some(Expr::Exit(Box::new(expr)))
    }

    // Parse an expression
    pub fn parse_expression(&mut self) -> Option<Expr> {
        self.parse_additive_expr()
    }

    // Parse additive expressions: term (+|-) term
    fn parse_additive_expr(&mut self) -> Option<Expr> {
        let mut left = self.parse_multiplicative_expr()?;
        
        while !self.is_finished() {
            match &self.tokens[self.pos] {
                Token::Plus => {
                    self.pos += 1;
                    if let Some(right) = self.parse_multiplicative_expr() {
                        left = Expr::BinaryOp {
                            op: BinOp::Add,
                            left: Box::new(left),
                            right: Box::new(right),
                        };
                    } else {
                        return None;
                    }
                },
                Token::Minus => {
                    self.pos += 1;
                    if let Some(right) = self.parse_multiplicative_expr() {
                        left = Expr::BinaryOp {
                            op: BinOp::Sub,
                            left: Box::new(left),
                            right: Box::new(right),
                        };
                    } else {
                        return None;
                    }
                },
                Token::Equal => {
                    self.pos += 1;
                    if let Some(right) = self.parse_multiplicative_expr() {
                        left = Expr::BinaryOp {
                            op: BinOp::Equal,
                            left: Box::new(left),
                            right: Box::new(right),
                        };
                    } else {
                        return None;
                    }
                },
                Token::NotEqual => {
                    self.pos += 1;
                    if let Some(right) = self.parse_multiplicative_expr() {
                        left = Expr::BinaryOp {
                            op: BinOp::NotEqual,
                            left: Box::new(left),
                            right: Box::new(right),
                        };
                    } else {
                        return None;
                    }
                },
                Token::LessThan => {
                    self.pos += 1;
                    if let Some(right) = self.parse_multiplicative_expr() {
                        left = Expr::BinaryOp {
                            op: BinOp::Lt,
                            left: Box::new(left),
                            right: Box::new(right),
                        };
                    } else {
                        return None;
                    }
                },
                Token::GreaterThan => {
                    self.pos += 1;
                    if let Some(right) = self.parse_multiplicative_expr() {
                        left = Expr::BinaryOp {
                            op: BinOp::Gt,
                            left: Box::new(left),
                            right: Box::new(right),
                        };
                    } else {
                        return None;
                    }
                },
                Token::LessThanEqual => {
                    self.pos += 1;
                    if let Some(right) = self.parse_multiplicative_expr() {
                        left = Expr::BinaryOp {
                            op: BinOp::Lte,
                            left: Box::new(left),
                            right: Box::new(right),
                        };
                    } else {
                        return None;
                    }
                },
                Token::GreaterThanEqual => {
                    self.pos += 1;
                    if let Some(right) = self.parse_multiplicative_expr() {
                        left = Expr::BinaryOp {
                            op: BinOp::Gte,
                            left: Box::new(left),
                            right: Box::new(right),
                        };
                    } else {
                        return None;
                    }
                },
                _ => break,
            }
        }
        
        Some(left)
    }

    // Parse multiplicative expressions: factor (*|/) factor
    fn parse_multiplicative_expr(&mut self) -> Option<Expr> {
        let mut left = self.parse_primary()?;
        
        while !self.is_finished() {
            match &self.tokens[self.pos] {
                Token::Star => {
                    self.pos += 1;
                    if let Some(right) = self.parse_primary() {
                        left = Expr::BinaryOp {
                            op: BinOp::Mul,
                            left: Box::new(left),
                            right: Box::new(right),
                        };
                    } else {
                        return None;
                    }
                },
                Token::Slash => {
                    self.pos += 1;
                    if let Some(right) = self.parse_primary() {
                        left = Expr::BinaryOp {
                            op: BinOp::Div,
                            left: Box::new(left),
                            right: Box::new(right),
                        };
                    } else {
                        return None;
                    }
                },
                _ => break,
            }
        }
        
        Some(left)
    }

    // Parse primary expressions: literal, variable, or parenthesized expression
    fn parse_primary(&mut self) -> Option<Expr> {
        if self.is_finished() {
            return None;
        }
        
        // Try parsing a float literal first (since it's more specific than integer)
        if let Some(expr) = self.parse_float_literal() {
            return Some(expr);
        }
        
        // Try parsing all other literal types
        if let Some(expr) = self.parse_number_literal() {
            return Some(expr);
        }
        
        if let Some(expr) = self.parse_string_literal() {
            return Some(expr);
        }
        
        if let Some(expr) = self.parse_bool_literal() {
            return Some(expr);
        }
        
        if let Some(expr) = self.parse_null_literal() {
            return Some(expr);
        }
        
        if let Some(expr) = self.parse_array_literal() {
            return Some(expr);
        }
        
        if let Some(expr) = self.parse_variable() {
            return Some(expr);
        }
        
        // Parenthesized expression
        if !self.is_finished() && self.tokens[self.pos] == Token::LParen {
            self.pos += 1;
            let expr = self.parse_expression()?;
            
            if self.is_finished() || self.tokens[self.pos] != Token::RParen {
                return None;
            }
            self.pos += 1;
            
            return Some(expr);
        }
        
        None
    }

    // Parse a number literal
    pub fn parse_number_literal(&mut self) -> Option<Expr> {
        if self.is_finished() {
            return None;
        }
        
        match &self.tokens[self.pos] {
            Token::Number(n) if !n.contains('.') => {
                self.pos += 1;
                match n.parse::<i32>() {
                    Ok(val) => Some(Expr::Number(val)),
                    Err(_) => None
                }
            },
            _ => None
        }
    }

    // Parse a float literal
    pub fn parse_float_literal(&mut self) -> Option<Expr> {
        if self.is_finished() {
            return None;
        }
        
        match &self.tokens[self.pos] {
            Token::Number(n) if n.contains('.') => {
                self.pos += 1;
                match n.parse::<f64>() {
                    Ok(val) => Some(Expr::Float(val)),
                    Err(_) => None
                }
            },
            _ => None
        }
    }

    // Parse a string literal
    pub fn parse_string_literal(&mut self) -> Option<Expr> {
        if self.is_finished() {
            return None;
        }
        
        match &self.tokens[self.pos] {
            Token::String(s) => {
                self.pos += 1;
                Some(Expr::StringLiteral(s.clone()))
            },
            _ => None
        }
    }

    // Parse a boolean literal
    pub fn parse_bool_literal(&mut self) -> Option<Expr> {
        if self.is_finished() {
            return None;
        }
        
        match &self.tokens[self.pos] {
            Token::Identifier(id) if id == "true" => {
                self.pos += 1;
                Some(Expr::Boolean(true))
            },
            Token::Identifier(id) if id == "false" => {
                self.pos += 1;
                Some(Expr::Boolean(false))
            },
            _ => None
        }
    }

    // Parse a null literal
    pub fn parse_null_literal(&mut self) -> Option<Expr> {
        if self.is_finished() {
            return None;
        }
        
        match &self.tokens[self.pos] {
            Token::Identifier(id) if id == "null" => {
                self.pos += 1;
                Some(Expr::Null)
            },
            _ => None
        }
    }

    // Parse an array literal
    pub fn parse_array_literal(&mut self) -> Option<Expr> {
        if self.is_finished() || self.tokens[self.pos] != Token::LBracket {
            return None;
        }
        
        self.pos += 1; // Consume '['
        let mut elements = Vec::new();
        
        // Handle empty array
        if !self.is_finished() && self.tokens[self.pos] == Token::RBracket {
            self.pos += 1; // Consume ']'
            return Some(Expr::Array(elements));
        }
        
        // Parse elements
        loop {
            if self.is_finished() {
                return None; // Unexpected end of input
            }
            
            // Parse element
            if let Some(element) = self.parse_expression() {
                elements.push(Box::new(element));
            } else {
                return None; // Expected an expression
            }
            
            // Check for comma or closing bracket
            if self.is_finished() {
                return None; // Unexpected end of input
            }
            
            if self.tokens[self.pos] == Token::RBracket {
                self.pos += 1; // Consume ']'
                break;
            }
            
            if self.tokens[self.pos] != Token::Comma {
                return None; // Expected ',' or ']'
            }
            
            self.pos += 1; // Consume ','
        }
        
        Some(Expr::Array(elements))
    }

    // Parse a variable reference
    pub fn parse_variable(&mut self) -> Option<Expr> {
        if self.is_finished() {
            return None;
        }
        
        match &self.tokens[self.pos] {
            Token::Identifier(id) => {
                self.pos += 1;
                Some(Expr::Variable(id.clone()))
            },
            _ => None
        }
    }

	// If-Statements
	pub fn parse_if_statement(&mut self) -> Option<Expr> {
		// Check for "if"
		if self.is_finished() || !matches!(&self.tokens[self.pos], Token::Identifier(id) if id == "if") {
			return None;
		}
		self.pos += 1;
		
		// expect '('
		if self.is_finished() || self.tokens[self.pos] != Token::LParen {
			return None;
		}
		self.pos += 1;

		// Parse condition
		let condition = match self.parse_expression() {
			Some(expr) => expr,
			None => return None,
		};
		
		// expect ')'
		if self.is_finished() || self.tokens[self.pos] != Token::RParen {
			return None;
		}
		self.pos += 1;

		// expect '{'
		if self.is_finished() || self.tokens[self.pos] != Token::LBrace {
			return None;
		}
		self.pos += 1;

		// Parse then-block
		let mut then_statements = Vec::new();
		while !self.is_finished() && self.tokens[self.pos] != Token::RBrace {
			if let Some(stmt) = self.parse_statement() {
				then_statements.push(Box::new(stmt));
			} else {
				return None; // Invalid statement in block
			}
		}

		// expect '}'
		if self.is_finished() || self.tokens[self.pos] != Token::RBrace {
			return None;
		}
		self.pos += 1;

		// Parse else-block
		let mut else_statements = None;
		if !self.is_finished() && matches!(&self.tokens[self.pos], Token::Identifier(id) if id == "else") {
			self.pos += 1;

			// expect '{'
			if self.is_finished() || self.tokens[self.pos] != Token::LBrace {
				return None;
			}
			self.pos += 1;
			
			let mut statements = Vec::new();
			while !self.is_finished() && self.tokens[self.pos] != Token::RBrace {
				if let Some(stmt) = self.parse_statement() {
					statements.push(Box::new(stmt));
				} else {
					return None; // Invalid statement in block
				}
			}

			// expect '}'
			if self.is_finished() || self.tokens[self.pos] != Token::RBrace {
				return None;
			}
			self.pos += 1;
			
			else_statements = Some(statements);
		}
		
		Some(Expr::If {
			condition: Box::new(condition),
			then_branch: then_statements,
			else_branch: else_statements,
		})
	}

	// This helper method parses a single statement
	pub fn parse_statement(&mut self) -> Option<Expr> {
		if let Some(stmt) = self.parse_const_declaration() {
			return Some(stmt);
		}
		if let Some(stmt) = self.parse_let_declaration() {
			return Some(stmt);
		}
		if let Some(stmt) = self.parse_assignment() {
			return Some(stmt);
		}
		if let Some(stmt) = self.parse_console_print_expr() {
			return Some(stmt);
		}
		if let Some(stmt) = self.parse_exit_expr() {
			return Some(stmt);
		}
		// Also if-statements could be nested
		if let Some(stmt) = self.parse_if_statement() {
			return Some(stmt);
		}
		None
	}
}