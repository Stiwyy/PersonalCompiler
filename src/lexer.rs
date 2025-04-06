#[derive(Debug, Clone)]
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
}

pub fn lex(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            '0'..='9' => {
                let mut number = 0;
                while let Some(d) = chars.peek().and_then(|d| d.to_digit(10)) {
                    number = number * 10 + d as i32;
                    chars.next();
                }
                tokens.push(Token::Number(number));
            }
            '+' => {
                chars.next();
                tokens.push(Token::Plus);
            }
            '-' => {
                chars.next();
                tokens.push(Token::Minus);
            }
            '*' => {
                chars.next();
                tokens.push(Token::Star);
            }
            '/' => {
                chars.next();
                tokens.push(Token::Slash);
            }
            '(' => {
                chars.next();
                tokens.push(Token::LParen);
            }
            ')' => {
                chars.next();
                tokens.push(Token::RParen);
            }
            ';' => {
                chars.next();
                tokens.push(Token::Semicolon);
            }
            c if c.is_alphabetic() => {
                let mut ident = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphanumeric() {
                        ident.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Ident(ident));
            }
            c if c.is_whitespace() => {
                chars.next();
            }
            _ => {
                panic!("Unrecognized character: {}", c);
            }
        }
    }

    tokens
}
