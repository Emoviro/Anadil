use std::fmt;

// A lightweight source span is enough for precise diagnostics in V1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceSpan {
    pub line: usize,
    pub column: usize,
}

impl SourceSpan {
    pub const fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

impl fmt::Display for SourceSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "satır {}, sütun {}", self.line, self.column)
    }
}

// Program is the top-level container for parsed functions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub functions: Vec<Function>,
}

// Functions are the only valid top-level declarations in V1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub span: SourceSpan,
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub span: SourceSpan,
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Sayi,
    Mantik,
    Metin,
    Dizi,
    Deger,
}

impl Type {
    pub fn name(self) -> &'static str {
        match self {
            Type::Sayi => "sayı",
            Type::Mantik => "mantık",
            Type::Metin => "metin",
            Type::Dizi => "dizi",
            Type::Deger => "değer",
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str((*self).name())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub span: SourceSpan,
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stmt {
    pub span: SourceSpan,
    pub kind: StmtKind,
}

impl Stmt {
    pub fn new(span: SourceSpan, kind: StmtKind) -> Self {
        Self { span, kind }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StmtKind {
    VarDecl(VarDecl),
    Assign(AssignStmt),
    If(IfStmt),
    Loop(LoopStmt),
    Break,
    Continue,
    Return(Option<Expr>),
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarDecl {
    pub span: SourceSpan,
    pub name: String,
    pub ty: Type,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssignStmt {
    pub span: SourceSpan,
    pub target: String,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfStmt {
    pub span: SourceSpan,
    pub condition: Expr,
    pub then_branch: Block,
    pub else_branch: Option<Block>,
}

// Counter loops reuse a small statement subset for init and step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoopStmt {
    pub span: SourceSpan,
    pub init: Option<LoopPart>,
    pub condition: Option<Expr>,
    pub step: Option<LoopPart>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoopPart {
    VarDecl(VarDecl),
    Assign(AssignStmt),
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expr {
    pub span: SourceSpan,
    pub kind: ExprKind,
}

impl Expr {
    pub fn new(span: SourceSpan, kind: ExprKind) -> Self {
        Self { span, kind }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprKind {
    Number(i64),
    Bool(bool),
    String(String),
    Array(Vec<Expr>),
    Variable(String),
    Index {
        target: Box<Expr>,
        index: Box<Expr>,
    },
    Call {
        callee: String,
        args: Vec<Expr>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Negate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
}
