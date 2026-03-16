use std::fmt;

// Program is the top-level container for parsed functions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub functions: Vec<Function>,
}

// Functions are the only valid top-level declarations in V1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Sayi,
    Mantik,
}

impl Type {
    pub fn name(self) -> &'static str {
        match self {
            Type::Sayi => "sayı",
            Type::Mantik => "mantık",
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
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
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
    pub name: String,
    pub ty: Type,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssignStmt {
    pub target: String,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfStmt {
    pub condition: Expr,
    pub then_branch: Block,
    pub else_branch: Option<Block>,
}

// Counter loops reuse a small statement subset for init and step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoopStmt {
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
pub enum Expr {
    Number(i64),
    Bool(bool),
    Variable(String),
    Call {
        callee: String,
        args: Vec<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
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
