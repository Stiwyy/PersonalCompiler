// Remove the duplicate derive attributes
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Number(String),
    String(String),
    Identifier(String),
    Plus,
    Minus,
    Star,
    Slash,
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanEqual,
    GreaterThanEqual,
    Assign,
    Semicolon,
    Dot,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
}

pub fn lex(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = source.chars().peekable();
    
    while let Some(&c) = chars.peek() {
        match c {
            // Skip whitespace
            c if c.is_whitespace() => {
                chars.next();
            },
            
            // Numbers
            c if c.is_digit(10) => {
                let mut number = String::new();
                let mut has_dot = false;
                
                while let Some(&c) = chars.peek() {
                    if c.is_digit(10) {
                        number.push(c);
                        chars.next();
                    } else if c == '.' && !has_dot {
                        has_dot = true;
                        number.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                
                tokens.push(Token::Number(number));
            },
            
            // Strings support both single and double quotes
            '"' | '\'' => {
                let quote_type = c; // Remember which quote type started the string
                chars.next(); // Skip opening quote
                let mut s = String::new();
                
                while let Some(&c) = chars.peek() {
                    if c == quote_type {
                        chars.next(); // Skip closing quote
                        break;
                    } else if c == '\\' {
                        // Handle escape sequences
                        chars.next(); 
                        if let Some(&next_c) = chars.peek() {
                            match next_c {
                                'n' => s.push('\n'),
                                't' => s.push('\t'),
                                'r' => s.push('\r'),
                                '\\' => s.push('\\'),
                                '\'' => s.push('\''),
                                '"' => s.push('"'),
                                _ => s.push(next_c), 
                            }
                            chars.next();
                        }
                    } else {
                        s.push(c);
                        chars.next();
                    }
                }
                
                tokens.push(Token::String(s));
            },
            
            // Identifiers and keywords
            c if c.is_alphabetic() || c == '_' => {
                let mut ident = String::new();
                
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        ident.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                
                // Keywords are just identifiers with special meaning
                tokens.push(Token::Identifier(ident));
            },
            
            // Operators and punctuation
            '+' => {
                chars.next();
                tokens.push(Token::Plus);
            },
            '-' => {
                chars.next();
                tokens.push(Token::Minus);
            },
            '*' => {
                chars.next();
                tokens.push(Token::Star);
            },
            '/' => {
				chars.next();
				if chars.peek() == Some(&'/') {
					chars.next();
					while let Some(&c) = chars.peek() {
						if c == '\n' {
							break;
						}
						chars.next();
					}
				} else if chars.peek() == Some(&'*') {
					chars.next();
					while let Some(&c) = chars.peek() {
						chars.next();
						if c == '*' && chars.peek() == Some(&'/') {
							chars.next();
							break;
						}
					}
					
				} else {
					tokens.push(Token::Slash);
				}
			},
            '=' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::Equal);
                } else {
                    tokens.push(Token::Assign);
                }
            },
            '!' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::NotEqual);
                } else {
                    // Handle single ! if needed
                }
            },
            '<' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::LessThanEqual);
                } else {
                    tokens.push(Token::LessThan);
                }
            },
            '>' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::GreaterThanEqual);
                } else {
                    tokens.push(Token::GreaterThan);
                }
            },
            '.' => {
                chars.next();
                tokens.push(Token::Dot);
            },
            ';' => {
                chars.next();
                tokens.push(Token::Semicolon);
            },
            '(' => {
                chars.next();
                tokens.push(Token::LParen);
            },
            ')' => {
                chars.next();
                tokens.push(Token::RParen);
            },
            '[' => {
                chars.next();
                tokens.push(Token::LBracket);
            },
            ']' => {
                chars.next();
                tokens.push(Token::RBracket);
            },
            ',' => {
                chars.next();
                tokens.push(Token::Comma);
            },
			'{' => {
				chars.next();
				tokens.push(Token::LBrace);
			},
			'}' => {
				chars.next();
				tokens.push(Token::RBrace);
			},
            
            // Skip any other characters (or handle them as errors)
            _ => {
                chars.next();
            }
        }
    }
    
    tokens
}