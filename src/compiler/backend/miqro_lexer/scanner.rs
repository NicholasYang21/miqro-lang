use std::fmt::{Debug, Formatter, write};
use std::str::Chars;

use unicode_xid::UnicodeXID;

use super::unescape::unescape;

impl Lexer<'_> {
    pub fn new(code: &str) -> Lexer {
        Lexer {
            text: code.into(),
            src: code.chars(),
            line: 1,
            column: 0,
            current: '\0',
        }
    }

    /// Generate next token from the source code.
    pub fn next_token(&mut self) -> Token {
        if self.eof() { return self.make_token(TokenType::Eof, ""); }

        let op_first = [
            '+', '-', '*', '/', '%', '^', '|', '>', '<', '&', '!',
            '=', ',', ';', '.', ':'
        ];
        
        let operators = 
            ["+", "-", "*", "/", "%", "^", "|", ">>", "<<", "&", "!", 
             "+=", "-=", "*=", "/=", "%=", "^=", "|=", ">>=", "<<=", "&=",
             ">", "<", "||", "&&", "==", "!=", ">=", "<=",
             ",", ";", ".", "->", "::"];
        
        let first = self.next().unwrap();
        match first {
            '\0' => self.make_token(TokenType::Eof, ""),
            c if c.is_whitespace() => { 
                self.eat_until(|c| !c.is_whitespace());
                self.next_token()
            }
            
            c if c == '/' && self.lookahead() == '/' => {
                self.next();
                self.eat_until(|c| c == '\n');
                self.next_token()
            }
            
            c if c == '/' && self.lookahead() == '*' => {
                self.next();
                self.next();
                while !(self.eof() || self.curr() == '*' && self.lookahead() == '/') {
                    self.next();
                }
                if !self.eof() {
                    self.next();
                    self.next();
                }
                self.next_token()
            }
            
            '\'' => { 
                let mut content = String::new();
                let (ln, col) = (self.line, self.column);
                while self.lookahead() != '\'' && !self.eof() {
                    content.push(self.next().unwrap());
                }
                
                if !self.eof() { self.next().unwrap(); }
                
                let text = unescape(&content);
                if let Err(e) = text {
                    return self.make_token(TokenType::Error, &e.to_string());
                }

                let text = text.unwrap();
                
                Token {
                    ty: TokenType::CharLiteral,
                    text,
                    line: ln,
                    column: col,
                }
            }
            
            '\"' => {
                let mut content = String::new();
                let (ln, col) = (self.line, self.column);
                while self.lookahead() != '\"' && !self.eof() {
                    content.push(self.next().unwrap());
                }

                if !self.eof() { self.next().unwrap(); }
                
                let text = unescape(&content);
                if let Err(e) = text {
                    return self.make_token(TokenType::Error, &e.to_string());
                }

                let text = text.unwrap();

                Token {
                    ty: TokenType::StringLiteral,
                    text,
                    line: ln,
                    column: col,
                }
            }
            
            c if c.is_xid_start() => {
                let mut id = String::new();
                
                id.push(c);
                
                loop {
                    let c = self.lookahead();

                    if c.is_xid_continue() {
                        id.push(c);
                        self.next();
                    } else {
                        break;
                    }
                }
                
                // Check if the identifier is a keyword.
                let keywords = 
                    ["let", "func", "if", "else", "while", "for", "return"];
                if keywords.contains(&id.as_str()) {
                    return self.make_token(TokenType::Keyword, &id);
                }
                
                if id == "true" || id == "false" {
                    return self.make_token(TokenType::BoolLiteral, &id);
                }
                
                self.make_token(TokenType::Id, &id)
            }
            
            c @ '0'..='9' => {
                if c == '0' {
                    match self.lookahead() {
                        'b' => {
                            let val = self.number("0b");
                            self.make_token(TokenType::IntLiteral, &val)
                        }

                        'o' => {
                            let val = self.number("0o");
                            self.make_token(TokenType::IntLiteral, &val)
                        }

                        'x' => {
                            let val = self.number("0x");
                            self.make_token(TokenType::IntLiteral, &val)
                        }

                        '.' => {
                            let val = self.float("0.");
                            self.make_token(TokenType::FloatLiteral, &val)
                        }

                        n @ '0'..='9' => {
                            let mut lit = String::from("0"); lit.push(n);
                            let val = self.number(&lit);
                            self.make_token(TokenType::IntLiteral, &val)
                        }

                        _ => {
                            // If it is not a valid number, return an error token.
                            self.make_token(TokenType::Error, "Invalid number literal suffix")
                        }
                    }
                } else {
                    if self.lookahead() == '.' {
                        let lit = self.float(&c.to_string());
                        return self.make_token(TokenType::FloatLiteral, &lit);
                    } 
                    let lit = self.number(&c.to_string());
                    self.make_token(TokenType::IntLiteral, &lit)
                }
            }
            
            c if op_first.contains(&c) => {
                let mut op = String::new();
                op.push(c);
                
                while operators.contains(&format!("{}{}", op, self.lookahead()).as_str()) {
                    op.push(self.lookahead());
                    self.next();
                }
                
                self.make_token(TokenType::Op, &op)
            }
            
            '(' => self.make_token(TokenType::LParen, "("),
            ')' => self.make_token(TokenType::RParen, ")"),
            '[' => self.make_token(TokenType::LBracket, "["),
            ']' => self.make_token(TokenType::RBracket, "]"),
            '{' => self.make_token(TokenType::LBrace, "{"),
            '}' => self.make_token(TokenType::RBrace, "}"),
            
            _ => self.make_token(TokenType::Error, "Invalid character"),
        }
    }

    pub fn eof(&self) -> bool {
        self.src.as_str().is_empty()
    }
    
    fn number(&mut self, lit: &str) -> String {
        let mut content = String::from(lit);
        
        match lit {
            "0b" => {
                self.next();
                while let c @ '0'..='1' = self.lookahead() {
                    content.push(c);
                    self.next();
                }
            }
            
            "0o" => {
                self.next();
                while let c @ '0'..='7' = self.lookahead() {
                    content.push(c);
                    self.next();
                }
            }
            
            "0x" => {
                self.next();
                while self.lookahead().is_ascii_hexdigit() {
                    self.next();
                    content.push(self.curr());
                }
            }
            
            _ => {
                while let _c @ '0'..='9' = self.lookahead() {
                    content.push(self.next().unwrap());
                }
            }
        }
        
        content
    }
    
    fn float(&mut self, lit: &str) -> String {
        self.next();
        let mut content = String::from(lit);
        
        // Read the integer part.
        while let c @ '0'..='9' = self.lookahead() {
            content.push(c);
            self.next();
        }
        
        content
    }
    
    fn eat_until(&mut self, mut f: impl FnMut(char) -> bool) {
        while !f(self.lookahead()) && !self.eof() {
            self.next();
        }
    }

    /// Get next char without modifying the source code.
    fn lookahead(&self) -> char {
        self.src.clone().next().unwrap_or('\0')
    }

    fn curr(&self) -> char {
        self.current
    }

    fn make_token(&self, ty: TokenType, text: &str) -> Token {
        Token::new(ty, text, self.line, self.column)
    }
}

/// A scanner to tokenize the source code into meaningful tokens.
pub struct Lexer<'a> {
    pub text: String,
    pub line: usize,
    pub column: usize,
    src: Chars<'a>,
    current: char,
}

impl Iterator for Lexer<'_> {
    type Item = char;
    fn next(&mut self) -> Option<Self::Item> {
        let c = self.src.next();

        self.current = c.unwrap();

        self.column += 1;
        if c == Some('\n') {
            self.line += 1;
            self.column = 0;
        }
        c
    }
}

/// The minimal lexeme of the code.
pub struct Token {
    pub ty: TokenType,
    pub text: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(ty: TokenType, text: &str, line: usize, column: usize) -> Token {
        Token { ty, text: text.into(), line, column }
    }
}

/// The type of token.
#[derive(Eq, PartialEq)]
pub enum TokenType {
    // identifier
    Id,
    // literals
    IntLiteral,
    BoolLiteral,
    StringLiteral,
    CharLiteral,
    FloatLiteral,
    // Keywords
    Keyword,
    // Operators
    // + - * / % ^ | >> << & !
    // += -= *= /= %= ^= |= >>= <<= &=
    // > < || && == != >= <=
    // , ; . ->
    Op,
    // Punctuations
    LParen,        // (
    RParen,        // )
    LBracket,      // [
    RBracket,      // ]
    LBrace,        // {
    RBrace,        // }
    // Special tokens
    Error,
    Eof,
}

impl Debug for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write(f, format_args!("Lexeme: (Type: {:?}, Content: {}, At: (L: {}, C, {}))", 
                              self.ty, self.text, self.line, self.column))
    }
}

impl Debug for TokenType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::Id => write!(f, "<identifier>"),
            TokenType::IntLiteral => write!(f, "<literal: int>"),
            TokenType::BoolLiteral => write!(f, "<literal: bool>"),
            TokenType::StringLiteral => write!(f, "<literal: string>"),
            TokenType::CharLiteral => write!(f, "<literal: char>"),
            TokenType::FloatLiteral => write!(f, "<literal: float>"),
            TokenType::Keyword => write!(f, "keyword"),
            TokenType::Eof => write!(f, "EOF"),
            TokenType::Op => write!(f, "<operator>"),
            TokenType::LParen => write!(f, "<punctuation>"),
            TokenType::RParen => write!(f, "<punctuation>"),
            TokenType::LBracket => write!(f, "<punctuation>"),
            TokenType::RBracket => write!(f, "<punctuation>"),
            TokenType::LBrace => write!(f, "<punctuation>"),
            TokenType::RBrace => write!(f, "<punctuation>"),
            TokenType::Error => write!(f, "<error msg>"),
        }
    }
}