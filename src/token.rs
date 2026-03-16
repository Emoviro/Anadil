use crate::ast::SourceSpan;

// Tokens are the shared contract between the lexer and the parser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    // Literals
    Number(i64),
    Ident(String),

    // Keywords
    Sayi,
    Mantik,
    Eger,
    Degilse,
    Dongu,
    Kir,
    Devam,
    Don,
    Dogru,
    Yanlis,

    // Punctuation
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Colon,
    Semicolon,

    // Operators
    Assign,
    Arrow,
    Plus,
    Minus,
    Star,
    Slash,
    EqEq,
    NotEq,
    Less,
    Greater,
    LessEq,
    GreaterEq,

    // Special
    Eof,
}

impl TokenKind {
    pub fn describe(&self) -> String {
        match self {
            TokenKind::Number(value) => format!("sayı sabiti `{value}`"),
            TokenKind::Ident(name) => format!("tanımlayıcı `{name}`"),
            TokenKind::Sayi => "`sayı`".to_string(),
            TokenKind::Mantik => "`mantık`".to_string(),
            TokenKind::Eger => "`eğer`".to_string(),
            TokenKind::Degilse => "`değilse`".to_string(),
            TokenKind::Dongu => "`döngü`".to_string(),
            TokenKind::Kir => "`kır`".to_string(),
            TokenKind::Devam => "`devam`".to_string(),
            TokenKind::Don => "`dön`".to_string(),
            TokenKind::Dogru => "`doğru`".to_string(),
            TokenKind::Yanlis => "`yanlış`".to_string(),
            TokenKind::LParen => "`(`".to_string(),
            TokenKind::RParen => "`)`".to_string(),
            TokenKind::LBrace => "`{`".to_string(),
            TokenKind::RBrace => "`}`".to_string(),
            TokenKind::Comma => "`,`".to_string(),
            TokenKind::Colon => "`:`".to_string(),
            TokenKind::Semicolon => "`;`".to_string(),
            TokenKind::Assign => "`=`".to_string(),
            TokenKind::Arrow => "`->`".to_string(),
            TokenKind::Plus => "`+`".to_string(),
            TokenKind::Minus => "`-`".to_string(),
            TokenKind::Star => "`*`".to_string(),
            TokenKind::Slash => "`/`".to_string(),
            TokenKind::EqEq => "`==`".to_string(),
            TokenKind::NotEq => "`!=`".to_string(),
            TokenKind::Less => "`<`".to_string(),
            TokenKind::Greater => "`>`".to_string(),
            TokenKind::LessEq => "`<=`".to_string(),
            TokenKind::GreaterEq => "`>=`".to_string(),
            TokenKind::Eof => "dosya sonu".to_string(),
        }
    }
}

// Each token keeps its source position for better diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(kind: TokenKind, line: usize, column: usize) -> Self {
        Self { kind, line, column }
    }

    pub fn span(&self) -> SourceSpan {
        SourceSpan::new(self.line, self.column)
    }
}
