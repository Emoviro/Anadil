use crate::error::LexError;
use crate::token::{Token, TokenKind};

// The lexer turns raw source text into a flat token stream.
pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();

        while let Some(ch) = self.current() {
            match ch {
                ' ' | '\t' | '\r' => self.advance(),
                '\n' => self.advance_newline(),
                '0'..='9' => tokens.push(self.lex_number()?),
                ch if is_ident_start(ch) => tokens.push(self.lex_ident_or_keyword()),
                '(' => {
                    tokens.push(self.make_token(TokenKind::LParen));
                    self.advance();
                }
                ')' => {
                    tokens.push(self.make_token(TokenKind::RParen));
                    self.advance();
                }
                '{' => {
                    tokens.push(self.make_token(TokenKind::LBrace));
                    self.advance();
                }
                '}' => {
                    tokens.push(self.make_token(TokenKind::RBrace));
                    self.advance();
                }
                ',' => {
                    tokens.push(self.make_token(TokenKind::Comma));
                    self.advance();
                }
                ':' => {
                    tokens.push(self.make_token(TokenKind::Colon));
                    self.advance();
                }
                ';' => {
                    tokens.push(self.make_token(TokenKind::Semicolon));
                    self.advance();
                }
                '+' => {
                    tokens.push(self.make_token(TokenKind::Plus));
                    self.advance();
                }
                '*' => {
                    tokens.push(self.make_token(TokenKind::Star));
                    self.advance();
                }
                '/' => {
                    tokens.push(self.make_token(TokenKind::Slash));
                    self.advance();
                }
                '=' => {
                    let line = self.line;
                    let column = self.column;
                    self.advance();

                    if self.current() == Some('=') {
                        self.advance();
                        tokens.push(Token::new(TokenKind::EqEq, line, column));
                    } else {
                        tokens.push(Token::new(TokenKind::Assign, line, column));
                    }
                }
                '!' => {
                    let line = self.line;
                    let column = self.column;
                    self.advance();

                    if self.current() == Some('=') {
                        self.advance();
                        tokens.push(Token::new(TokenKind::NotEq, line, column));
                    } else {
                        return Err(LexError::new("Beklenmeyen karakter: !", line, column));
                    }
                }
                '<' => {
                    let line = self.line;
                    let column = self.column;
                    self.advance();

                    if self.current() == Some('=') {
                        self.advance();
                        tokens.push(Token::new(TokenKind::LessEq, line, column));
                    } else {
                        tokens.push(Token::new(TokenKind::Less, line, column));
                    }
                }
                '>' => {
                    let line = self.line;
                    let column = self.column;
                    self.advance();

                    if self.current() == Some('=') {
                        self.advance();
                        tokens.push(Token::new(TokenKind::GreaterEq, line, column));
                    } else {
                        tokens.push(Token::new(TokenKind::Greater, line, column));
                    }
                }
                '-' => {
                    let line = self.line;
                    let column = self.column;
                    self.advance();

                    if self.current() == Some('>') {
                        self.advance();
                        tokens.push(Token::new(TokenKind::Arrow, line, column));
                    } else {
                        tokens.push(Token::new(TokenKind::Minus, line, column));
                    }
                }
                _ => {
                    return Err(LexError::new(
                        format!("Geçersiz karakter: {ch}"),
                        self.line,
                        self.column,
                    ));
                }
            }
        }

        tokens.push(Token::new(TokenKind::Eof, self.line, self.column));
        Ok(tokens)
    }

    fn lex_number(&mut self) -> Result<Token, LexError> {
        let line = self.line;
        let column = self.column;
        let mut number = String::new();

        while let Some(ch) = self.current() {
            if ch.is_ascii_digit() {
                number.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        let value = number
            .parse::<i64>()
            .map_err(|_| LexError::new("Sayı çözümlenemedi", line, column))?;

        Ok(Token::new(TokenKind::Number(value), line, column))
    }

    fn lex_ident_or_keyword(&mut self) -> Token {
        let line = self.line;
        let column = self.column;
        let mut ident = String::new();

        while let Some(ch) = self.current() {
            if is_ident_continue(ch) {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        let kind = match ident.as_str() {
            "sayı" => TokenKind::Sayi,
            "mantık" => TokenKind::Mantik,
            "eğer" => TokenKind::Eger,
            "değilse" => TokenKind::Degilse,
            "döngü" => TokenKind::Dongu,
            "kır" => TokenKind::Kir,
            "devam" => TokenKind::Devam,
            "dön" => TokenKind::Don,
            "doğru" => TokenKind::Dogru,
            "yanlış" => TokenKind::Yanlis,
            _ => TokenKind::Ident(ident),
        };

        Token::new(kind, line, column)
    }

    fn make_token(&self, kind: TokenKind) -> Token {
        Token::new(kind, self.line, self.column)
    }

    fn current(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn advance(&mut self) {
        if self.current().is_some() {
            self.pos += 1;
            self.column += 1;
        }
    }

    fn advance_newline(&mut self) {
        self.pos += 1;
        self.line += 1;
        self.column = 1;
    }
}

fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch.is_alphabetic()
}

fn is_ident_continue(ch: char) -> bool {
    ch == '_' || ch.is_alphanumeric()
}
