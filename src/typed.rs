use crate::ast::{BinaryOp, SourceSpan, Type};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalKind {
    Param,
    Variable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedLocalRef {
    pub id: LocalId,
    pub name: String,
    pub ty: Type,
    pub kind: LocalKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedProgram {
    pub functions: Vec<TypedFunction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedFunction {
    pub function_id: FunctionId,
    pub span: SourceSpan,
    pub name: String,
    pub params: Vec<TypedParam>,
    pub return_type: Option<Type>,
    pub body: TypedBlock,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedParam {
    pub local_id: LocalId,
    pub span: SourceSpan,
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedBlock {
    pub span: SourceSpan,
    pub statements: Vec<TypedStmt>,
    pub definitely_returns: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedStmt {
    pub span: SourceSpan,
    pub kind: TypedStmtKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypedStmtKind {
    VarDecl(TypedVarDecl),
    Assign(TypedAssignStmt),
    If(TypedIfStmt),
    Loop(TypedLoopStmt),
    Break,
    Continue,
    Return(Option<TypedExpr>),
    Expr(TypedExpr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedVarDecl {
    pub local_id: LocalId,
    pub span: SourceSpan,
    pub name: String,
    pub ty: Type,
    pub value: TypedExpr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedAssignStmt {
    pub span: SourceSpan,
    pub target: TypedLocalRef,
    pub value: TypedExpr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedIfStmt {
    pub span: SourceSpan,
    pub condition: TypedExpr,
    pub then_branch: TypedBlock,
    pub else_branch: Option<TypedBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedLoopStmt {
    pub span: SourceSpan,
    pub init: Option<TypedLoopPart>,
    pub condition: Option<TypedExpr>,
    pub step: Option<TypedLoopPart>,
    pub body: TypedBlock,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypedLoopPart {
    VarDecl(TypedVarDecl),
    Assign(TypedAssignStmt),
    Expr(TypedExpr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedExpr {
    pub span: SourceSpan,
    pub ty: TypedExprType,
    pub kind: TypedExprKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypedExprType {
    Value(Type),
    Void,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypedExprKind {
    Number(i64),
    Bool(bool),
    Variable(TypedLocalRef),
    Call {
        target: CallTarget,
        args: Vec<TypedExpr>,
    },
    Binary {
        left: Box<TypedExpr>,
        op: BinaryOp,
        right: Box<TypedExpr>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallTarget {
    Function {
        function_id: FunctionId,
        name: String,
    },
    Builtin(BuiltinFunction),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinFunction {
    Yazdir,
}

impl BuiltinFunction {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "yazdir" => Some(Self::Yazdir),
            _ => None,
        }
    }

    pub fn return_type(self) -> Option<Type> {
        match self {
            BuiltinFunction::Yazdir => None,
        }
    }
}
