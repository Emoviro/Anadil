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
                '"' => tokens.push(self.lex_string()?),
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
                    let line = self.line;
                    let column = self.column;
                    self.advance();

                    if self.current() == Some('/') {
                        self.skip_line_comment();
                    } else {
                        tokens.push(Token::new(TokenKind::Slash, line, column));
                    }
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
                        format!("Gecersiz karakter: {ch}"),
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
            .map_err(|_| LexError::new("Sayi cozumlenemedi", line, column))?;

        Ok(Token::new(TokenKind::Number(value), line, column))
    }

    fn lex_string(&mut self) -> Result<Token, LexError> {
        let line = self.line;
        let column = self.column;
        let mut value = String::new();
        self.advance();

        while let Some(ch) = self.current() {
            match ch {
                '"' => {
                    self.advance();
                    return Ok(Token::new(TokenKind::String(value), line, column));
                }
                '\n' => {
                    return Err(LexError::new(
                        "Metin sabiti satir sonundan once kapatilmali",
                        line,
                        column,
                    ));
                }
                '\\' => {
                    self.advance();
                    let Some(escaped) = self.current() else {
                        return Err(LexError::new(
                            "Metin sabiti icinde eksik kacis dizisi",
                            line,
                            column,
                        ));
                    };

                    let escaped = match escaped {
                        '"' => '"',
                        '\\' => '\\',
                        'n' => '\n',
                        't' => '\t',
                        other => {
                            return Err(LexError::new(
                                format!("Desteklenmeyen kacis dizisi: \\{other}"),
                                self.line,
                                self.column,
                            ));
                        }
                    };
                    value.push(escaped);
                    self.advance();
                }
                _ => {
                    value.push(ch);
                    self.advance();
                }
            }
        }

        Err(LexError::new(
            "Metin sabiti dosya sonundan once kapatilmali",
            line,
            column,
        ))
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
            "say\u{0131}" | "sayÄ±" | "sayÃ„Â±" => TokenKind::Sayi,
            "mant\u{0131}k" | "mantÄ±k" | "mantÃ„Â±k" => TokenKind::Mantik,
            "metin" => TokenKind::Metin,
            "e\u{011f}er" | "eÄŸer" | "eÃ„Å¸er" => TokenKind::Eger,
            "de\u{011f}ilse" | "deÄŸilse" | "deÃ„Å¸ilse" => TokenKind::Degilse,
            "d\u{00f6}ng\u{00fc}" | "dÃ¶ngÃ¼" | "dÃƒÂ¶ngÃƒÂ¼" => TokenKind::Dongu,
            "k\u{0131}r" | "kÄ±r" | "kÃ„Â±r" => TokenKind::Kir,
            "devam" => TokenKind::Devam,
            "d\u{00f6}n" | "dÃ¶n" | "dÃƒÂ¶n" => TokenKind::Don,
            "do\u{011f}ru" | "doÄŸru" | "doÃ„Å¸ru" => TokenKind::Dogru,
            "yanl\u{0131}\u{015f}" | "yanlÄ±ÅŸ" | "yanlÃ„Â±Ã…Å¸" => TokenKind::Yanlis,
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

    fn skip_line_comment(&mut self) {
        while let Some(ch) = self.current() {
            if ch == '\n' {
                break;
            }

            self.advance();
        }
    }
}

fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch.is_alphabetic()
}

fn is_ident_continue(ch: char) -> bool {
    ch == '_' || ch.is_alphanumeric()
}
