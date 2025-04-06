#[derive(Debug, Clone)]
//Token for each valid character
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
    Println, // Token for println
}

//Break code down into Tokens
pub fn lex(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    //Goes through each char.
    while let Some(&c) = chars.peek() {
        match c {
            //Char is a number between 0 and 9
            '0'..='9' => {
                let mut number = 0;
                //Checks if its a number with more than one digit and if so then build the entire number.
                while let Some(d) = chars.peek().and_then(|d| d.to_digit(10)) {
                    number = number * 10 + d as i32;
                    chars.next();
                }
                //adds number as a number token to tokens
                tokens.push(Token::Number(number));
            }
            //if char is a + then add it as a plus token to tokens
            '+' => {
                chars.next();
                tokens.push(Token::Plus);
            }
            //if char is a - then add it as a minus token to tokens
            '-' => {
                chars.next();
                tokens.push(Token::Minus);
            }
            //if char is a * then add it as a star token to tokens
            '*' => {
                chars.next();
                tokens.push(Token::Star);
            }
            //if char is a / then add it as a slash token to tokens
            '/' => {
                chars.next();
                tokens.push(Token::Slash);
            }
            //if char is a ( then add it as a LParen token to tokens
            '(' => {
                chars.next();
                tokens.push(Token::LParen);
            }
            //if char is a ) then add it as a RParen token to tokens
            ')' => {
                chars.next();
                tokens.push(Token::RParen);
            }
            //if char is a ; then add it as a semicolon token to tokens
            ';' => {
                chars.next();
                tokens.push(Token::Semicolon);
            }
            //if char is a letter
            c if c.is_alphabetic() => {
                let mut ident = String::new();
                //adds char to ident until its no longer a letter (alphanumeric)
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphanumeric() {
                        ident.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                //adds string to tokens as a ident token
                tokens.push(Token::Ident(ident));
            }

            //igonres whitespace
            c if c.is_whitespace() => {
                chars.next();
            }
            //if the char is unknown then stop with an error message
            _ => {
                panic!("Unrecognized character: {}", c);
            }
            //
            'p' if chars.clone().take(6).collect::<String>() == "println" => {
                chars.nth(5);  // Consume the 'println' keyword
                tokens.push(Token::Println);
            },
        }
    }

    tokens
}
