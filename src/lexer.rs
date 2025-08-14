#[derive(Debug, Clone)]
// Token for each valid character
pub enum Token {
    Number(i32),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    Ident(String),
    Semicolon,
    Println,
    Print,
    Dot,
    StringLiteral(String),
    Const,
    Equals,
}

// Break code down into Tokens
pub fn lex(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    // Goes through each char.
    while let Some(&c) = chars.peek() {
        match c {
            // Char is a number between 0 and 9
            '0'..='9' => {
                let mut number = 0;
                // Checks if it's a number with more than one digit and if so then builds the entire number.
                while let Some(d) = chars.peek().and_then(|d| d.to_digit(10)) {
                    number = number * 10 + d as i32;
                    chars.next();
                }
                // Adds number as a number token to tokens
                tokens.push(Token::Number(number));
            }
            // if char is a + then add it as a plus token to tokens
            '+' => {
                chars.next();
                tokens.push(Token::Plus);
            }
            // if char is a - then add it as a minus token to tokens
            '-' => {
                chars.next();
                tokens.push(Token::Minus);
            }
            // if char is a * then add it as a star token to tokens
            '*' => {
                chars.next();
                tokens.push(Token::Star);
            }
            // if char is a / then add it as a slash token to tokens
            '/' => {
                chars.next();
                tokens.push(Token::Slash);
            }
            // if char is a ( then add it as a LParen token to tokens
            '(' => {
                chars.next();
                tokens.push(Token::LParen);
            }
            // if char is a ) then add it as a RParen token to tokens
            ')' => {
                chars.next();
                tokens.push(Token::RParen);
            }
            // if char is a ; then add it as a semicolon token to tokens
            ';' => {
                chars.next();
                tokens.push(Token::Semicolon);
            }
            // if char is a . then add it as a Dot token to tokens
            '.' => {
                chars.next();
                tokens.push(Token::Dot);
            }
            // if char is a " then parse a string literal
            '"' => {
                chars.next(); // consume opening quote
                let mut s = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch == '"' {
                        break;
                    }
                    s.push(ch);
                    chars.next();
                }
                // consume closing quote
                if let Some('"') = chars.next() {
                    tokens.push(Token::StringLiteral(s));
                } else {
                    panic!("Unterminated string literal");
                }
            }
            '=' => {
                chars.next();
                tokens.push(Token::Equals)
            }

            // if char is a letter
            c if c.is_alphabetic() => {
                let mut ident = String::new();
                // adds char to ident until it's no longer a letter (alphanumeric)
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphanumeric() {
                        ident.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                // Check for keywords "println" or "print"
                if ident == "println" {
                    tokens.push(Token::Println);
                } else if ident == "print" {
                    tokens.push(Token::Print);
                } else if ident == "const" {
                    tokens.push(Token::Const)
                } else {
                    tokens.push(Token::Ident(ident));
                }
            }
            // ignores whitespace
            c if c.is_whitespace() => {
                chars.next();
            }
            // if the char is unknown then stop with an error message
            _ => {
                panic!("Unrecognized character: {}", c);
            }
        }
    }

    tokens
}
