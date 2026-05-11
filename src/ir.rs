use std::fmt;

use crate::ast::{BinaryOp, Type, UnaryOp};
use crate::typed::{
    BuiltinFunction, CallTarget, FunctionId, LocalId, TypedBlock, TypedExpr, TypedExprKind,
    TypedExprType, TypedFunction, TypedLoopPart, TypedProgram, TypedStmt, TypedStmtKind,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrProgram {
    pub functions: Vec<IrFunction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrFunction {
    pub id: FunctionId,
    pub name: String,
    pub params: Vec<IrLocal>,
    pub return_type: Option<Type>,
    pub locals: Vec<IrLocal>,
    pub body: Vec<IrInstruction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrLocal {
    pub id: LocalId,
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrInstruction {
    Declare {
        local: IrLocal,
        value: IrExpr,
    },
    Assign {
        target: IrLocal,
        value: IrExpr,
    },
    Expr(IrExpr),
    If {
        condition: IrExpr,
        then_body: Vec<IrInstruction>,
        else_body: Vec<IrInstruction>,
    },
    Loop {
        init: Vec<IrInstruction>,
        condition: Option<IrExpr>,
        step: Vec<IrInstruction>,
        body: Vec<IrInstruction>,
    },
    Break,
    Continue,
    Return(Option<IrExpr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrExpr {
    Number(i64),
    Bool(bool),
    String(String),
    Local(LocalId, String),
    Call {
        target: IrCallTarget,
        args: Vec<IrExpr>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<IrExpr>,
    },
    Binary {
        left: Box<IrExpr>,
        op: BinaryOp,
        right: Box<IrExpr>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrCallTarget {
    Function { id: FunctionId, name: String },
    Builtin(BuiltinFunction),
    Runtime(IrRuntimeCall),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrRuntimeCall {
    Yazdir(Type),
    MetinUzunluk,
    MetinEsit,
    MetinEsitDegil,
}

pub fn lower_typed_program(program: &TypedProgram) -> IrProgram {
    IrProgram {
        functions: program.functions.iter().map(lower_function).collect(),
    }
}

fn lower_function(function: &TypedFunction) -> IrFunction {
    IrFunction {
        id: function.function_id,
        name: function.name.clone(),
        params: function
            .params
            .iter()
            .map(|param| IrLocal {
                id: param.local_id,
                name: param.name.clone(),
                ty: param.ty,
            })
            .collect(),
        return_type: function.return_type,
        locals: collect_locals(function),
        body: lower_block(&function.body),
    }
}

fn collect_locals(function: &TypedFunction) -> Vec<IrLocal> {
    let mut locals: Vec<IrLocal> = function
        .params
        .iter()
        .map(|param| IrLocal {
            id: param.local_id,
            name: param.name.clone(),
            ty: param.ty,
        })
        .collect();
    collect_block_locals(&function.body, &mut locals);
    locals.sort_by_key(|local| local.id.0);
    locals
}

fn collect_block_locals(block: &TypedBlock, locals: &mut Vec<IrLocal>) {
    for statement in &block.statements {
        match &statement.kind {
            TypedStmtKind::VarDecl(decl) => locals.push(IrLocal {
                id: decl.local_id,
                name: decl.name.clone(),
                ty: decl.ty,
            }),
            TypedStmtKind::If(if_stmt) => {
                collect_block_locals(&if_stmt.then_branch, locals);
                if let Some(else_branch) = &if_stmt.else_branch {
                    collect_block_locals(else_branch, locals);
                }
            }
            TypedStmtKind::Loop(loop_stmt) => {
                if let Some(part) = &loop_stmt.init {
                    collect_loop_part_locals(part, locals);
                }
                if let Some(part) = &loop_stmt.step {
                    collect_loop_part_locals(part, locals);
                }
                collect_block_locals(&loop_stmt.body, locals);
            }
            _ => {}
        }
    }
}

fn collect_loop_part_locals(part: &TypedLoopPart, locals: &mut Vec<IrLocal>) {
    if let TypedLoopPart::VarDecl(decl) = part {
        locals.push(IrLocal {
            id: decl.local_id,
            name: decl.name.clone(),
            ty: decl.ty,
        });
    }
}

fn lower_block(block: &TypedBlock) -> Vec<IrInstruction> {
    block.statements.iter().map(lower_statement).collect()
}

fn lower_statement(statement: &TypedStmt) -> IrInstruction {
    match &statement.kind {
        TypedStmtKind::VarDecl(decl) => IrInstruction::Declare {
            local: IrLocal {
                id: decl.local_id,
                name: decl.name.clone(),
                ty: decl.ty,
            },
            value: lower_expr(&decl.value),
        },
        TypedStmtKind::Assign(assign) => IrInstruction::Assign {
            target: IrLocal {
                id: assign.target.id,
                name: assign.target.name.clone(),
                ty: assign.target.ty,
            },
            value: lower_expr(&assign.value),
        },
        TypedStmtKind::Expr(expr) => IrInstruction::Expr(lower_expr(expr)),
        TypedStmtKind::If(if_stmt) => IrInstruction::If {
            condition: lower_expr(&if_stmt.condition),
            then_body: lower_block(&if_stmt.then_branch),
            else_body: if_stmt
                .else_branch
                .as_ref()
                .map(lower_block)
                .unwrap_or_default(),
        },
        TypedStmtKind::Loop(loop_stmt) => IrInstruction::Loop {
            init: loop_stmt
                .init
                .as_ref()
                .map(lower_loop_part)
                .into_iter()
                .collect(),
            condition: loop_stmt.condition.as_ref().map(lower_expr),
            step: loop_stmt
                .step
                .as_ref()
                .map(lower_loop_part)
                .into_iter()
                .collect(),
            body: lower_block(&loop_stmt.body),
        },
        TypedStmtKind::Break => IrInstruction::Break,
        TypedStmtKind::Continue => IrInstruction::Continue,
        TypedStmtKind::Return(value) => IrInstruction::Return(value.as_ref().map(lower_expr)),
    }
}

fn lower_loop_part(part: &TypedLoopPart) -> IrInstruction {
    match part {
        TypedLoopPart::VarDecl(decl) => IrInstruction::Declare {
            local: IrLocal {
                id: decl.local_id,
                name: decl.name.clone(),
                ty: decl.ty,
            },
            value: lower_expr(&decl.value),
        },
        TypedLoopPart::Assign(assign) => IrInstruction::Assign {
            target: IrLocal {
                id: assign.target.id,
                name: assign.target.name.clone(),
                ty: assign.target.ty,
            },
            value: lower_expr(&assign.value),
        },
        TypedLoopPart::Expr(expr) => IrInstruction::Expr(lower_expr(expr)),
    }
}

fn lower_expr(expr: &TypedExpr) -> IrExpr {
    match &expr.kind {
        TypedExprKind::Number(value) => IrExpr::Number(*value),
        TypedExprKind::Bool(value) => IrExpr::Bool(*value),
        TypedExprKind::String(value) => IrExpr::String(value.clone()),
        TypedExprKind::Variable(local) => IrExpr::Local(local.id, local.name.clone()),
        TypedExprKind::Call { target, args } => lower_call_expr(target, args),
        TypedExprKind::Unary { op, expr } => IrExpr::Unary {
            op: *op,
            expr: Box::new(lower_expr(expr)),
        },
        TypedExprKind::Binary { left, op, right } => lower_binary_expr(left, *op, right),
    }
}

fn lower_call_expr(target: &CallTarget, args: &[TypedExpr]) -> IrExpr {
    if let CallTarget::Builtin(builtin) = target {
        match builtin {
            BuiltinFunction::Yazdir => {
                if let Some(arg) = args.first() {
                    return IrExpr::Call {
                        target: IrCallTarget::Runtime(IrRuntimeCall::Yazdir(value_type(arg))),
                        args: args.iter().map(lower_expr).collect(),
                    };
                }
            }
            BuiltinFunction::Uzunluk => {
                return IrExpr::Call {
                    target: IrCallTarget::Runtime(IrRuntimeCall::MetinUzunluk),
                    args: args.iter().map(lower_expr).collect(),
                };
            }
        }
    }

    IrExpr::Call {
        target: lower_call_target(target),
        args: args.iter().map(lower_expr).collect(),
    }
}

fn lower_binary_expr(left: &TypedExpr, op: BinaryOp, right: &TypedExpr) -> IrExpr {
    if is_metin_equality(left, op, right) {
        let runtime_call = match op {
            BinaryOp::Equal => IrRuntimeCall::MetinEsit,
            BinaryOp::NotEqual => IrRuntimeCall::MetinEsitDegil,
            _ => unreachable!(),
        };
        return IrExpr::Call {
            target: IrCallTarget::Runtime(runtime_call),
            args: vec![lower_expr(left), lower_expr(right)],
        };
    }

    IrExpr::Binary {
        left: Box::new(lower_expr(left)),
        op,
        right: Box::new(lower_expr(right)),
    }
}

fn lower_call_target(target: &CallTarget) -> IrCallTarget {
    match target {
        CallTarget::Function { function_id, name } => IrCallTarget::Function {
            id: *function_id,
            name: name.clone(),
        },
        CallTarget::Builtin(builtin) => IrCallTarget::Builtin(*builtin),
    }
}

fn value_type(expr: &TypedExpr) -> Type {
    match expr.ty {
        TypedExprType::Value(ty) => ty,
        TypedExprType::Void => panic!("void expression cannot be lowered as runtime value"),
    }
}

fn is_metin_equality(left: &TypedExpr, op: BinaryOp, right: &TypedExpr) -> bool {
    matches!(op, BinaryOp::Equal | BinaryOp::NotEqual)
        && matches!(left.ty, TypedExprType::Value(Type::Metin))
        && matches!(right.ty, TypedExprType::Value(Type::Metin))
}

pub fn format_ir(program: &IrProgram) -> String {
    let mut output = String::new();
    for (index, function) in program.functions.iter().enumerate() {
        if index > 0 {
            output.push('\n');
        }
        format_function(function, &mut output);
    }
    output
}

fn format_function(function: &IrFunction, output: &mut String) {
    output.push_str("fn ");
    output.push_str(&function.name);
    output.push('(');
    for (index, param) in function.params.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        output.push_str(&format_local(param));
    }
    output.push(')');
    if let Some(return_type) = function.return_type {
        output.push_str(" -> ");
        output.push_str(&return_type.to_string());
    }
    output.push_str(" {\n");

    if !function.locals.is_empty() {
        output.push_str("  locals:\n");
        for local in &function.locals {
            output.push_str("    ");
            output.push_str(&format_local(local));
            output.push('\n');
        }
    }

    output.push_str("  body:\n");
    format_instructions(&function.body, 2, output);
    output.push_str("}\n");
}

fn format_instructions(instructions: &[IrInstruction], indent: usize, output: &mut String) {
    for instruction in instructions {
        format_instruction(instruction, indent, output);
    }
}

fn format_instruction(instruction: &IrInstruction, indent: usize, output: &mut String) {
    push_indent(output, indent);
    match instruction {
        IrInstruction::Declare { local, value } => {
            output.push_str("decl ");
            output.push_str(&format_local(local));
            output.push_str(" = ");
            output.push_str(&value.to_string());
            output.push('\n');
        }
        IrInstruction::Assign { target, value } => {
            output.push_str("set ");
            output.push_str(&format_local_ref(target));
            output.push_str(" = ");
            output.push_str(&value.to_string());
            output.push('\n');
        }
        IrInstruction::Expr(expr) => {
            output.push_str("expr ");
            output.push_str(&expr.to_string());
            output.push('\n');
        }
        IrInstruction::If {
            condition,
            then_body,
            else_body,
        } => {
            output.push_str("if ");
            output.push_str(&condition.to_string());
            output.push_str(" {\n");
            format_instructions(then_body, indent + 1, output);
            push_indent(output, indent);
            if else_body.is_empty() {
                output.push_str("}\n");
            } else {
                output.push_str("} else {\n");
                format_instructions(else_body, indent + 1, output);
                push_indent(output, indent);
                output.push_str("}\n");
            }
        }
        IrInstruction::Loop {
            init,
            condition,
            step,
            body,
        } => {
            output.push_str("loop");
            if !init.is_empty() || condition.is_some() || !step.is_empty() {
                output.push_str(" (");
                output.push_str(&format_inline_instructions(init));
                output.push_str("; ");
                if let Some(condition) = condition {
                    output.push_str(&condition.to_string());
                }
                output.push_str("; ");
                output.push_str(&format_inline_instructions(step));
                output.push(')');
            }
            output.push_str(" {\n");
            format_instructions(body, indent + 1, output);
            push_indent(output, indent);
            output.push_str("}\n");
        }
        IrInstruction::Break => output.push_str("break\n"),
        IrInstruction::Continue => output.push_str("continue\n"),
        IrInstruction::Return(Some(expr)) => {
            output.push_str("return ");
            output.push_str(&expr.to_string());
            output.push('\n');
        }
        IrInstruction::Return(None) => output.push_str("return\n"),
    }
}

fn format_inline_instructions(instructions: &[IrInstruction]) -> String {
    instructions
        .iter()
        .map(|instruction| match instruction {
            IrInstruction::Declare { local, value } => {
                format!("decl {} = {value}", format_local(local))
            }
            IrInstruction::Assign { target, value } => {
                format!("set {} = {value}", format_local_ref(target))
            }
            IrInstruction::Expr(expr) => format!("expr {expr}"),
            IrInstruction::Break => "break".to_string(),
            IrInstruction::Continue => "continue".to_string(),
            IrInstruction::Return(Some(expr)) => format!("return {expr}"),
            IrInstruction::Return(None) => "return".to_string(),
            IrInstruction::If { .. } | IrInstruction::Loop { .. } => "<nested>".to_string(),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_local(local: &IrLocal) -> String {
    format!("{}#{}: {}", local.name, local.id.0, local.ty)
}

fn format_local_ref(local: &IrLocal) -> String {
    format!("{}#{}", local.name, local.id.0)
}

fn push_indent(output: &mut String, indent: usize) {
    for _ in 0..indent {
        output.push_str("  ");
    }
}

impl fmt::Display for IrExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IrExpr::Number(value) => write!(f, "{value}"),
            IrExpr::Bool(true) => f.write_str("dogru"),
            IrExpr::Bool(false) => f.write_str("yanlis"),
            IrExpr::String(value) => write!(f, "\"{}\"", value.escape_default()),
            IrExpr::Local(id, name) => write!(f, "{name}#{}", id.0),
            IrExpr::Call { target, args } => {
                write!(f, "{target}(")?;
                for (index, arg) in args.iter().enumerate() {
                    if index > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{arg}")?;
                }
                f.write_str(")")
            }
            IrExpr::Unary { op, expr } => write!(f, "({op} {expr})"),
            IrExpr::Binary { left, op, right } => write!(f, "({left} {op} {right})"),
        }
    }
}

impl fmt::Display for IrCallTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IrCallTarget::Function { id, name } => write!(f, "{name}@{}", id.0),
            IrCallTarget::Builtin(BuiltinFunction::Yazdir) => f.write_str("builtin.yazdir"),
            IrCallTarget::Builtin(BuiltinFunction::Uzunluk) => f.write_str("builtin.uzunluk"),
            IrCallTarget::Runtime(call) => write!(f, "runtime.{call}"),
        }
    }
}

impl fmt::Display for IrRuntimeCall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IrRuntimeCall::Yazdir(Type::Sayi) => f.write_str("yazdir_sayi"),
            IrRuntimeCall::Yazdir(Type::Mantik) => f.write_str("yazdir_mantik"),
            IrRuntimeCall::Yazdir(Type::Metin) => f.write_str("yazdir_metin"),
            IrRuntimeCall::MetinUzunluk => f.write_str("metin_uzunluk"),
            IrRuntimeCall::MetinEsit => f.write_str("metin_esit"),
            IrRuntimeCall::MetinEsitDegil => f.write_str("metin_esit_degil"),
        }
    }
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOp::Negate => f.write_str("-"),
        }
    }
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = match self {
            BinaryOp::Add => "+",
            BinaryOp::Subtract => "-",
            BinaryOp::Multiply => "*",
            BinaryOp::Divide => "/",
            BinaryOp::Equal => "==",
            BinaryOp::NotEqual => "!=",
            BinaryOp::Less => "<",
            BinaryOp::Greater => ">",
            BinaryOp::LessEqual => "<=",
            BinaryOp::GreaterEqual => ">=",
        };
        f.write_str(op)
    }
}

#[cfg(test)]
mod tests {
    use crate::ir::{format_ir, lower_typed_program};
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::sema::Analyzer;

    fn lower(source: &str) -> String {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().expect("source should lex");
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().expect("source should parse");
        let typed = Analyzer::analyze(&program).expect("source should typecheck");
        let ir = lower_typed_program(&typed);
        format_ir(&ir)
    }

    #[test]
    fn lowers_function_call_and_return() {
        let ir = lower(
            "\
Topla(a: sayı, b: sayı) -> sayı {
    dön a + b;
}

Ana() {
    yazdir(Topla(10, 20));
}
",
        );

        assert!(ir.contains("fn Topla(a#0: sayı, b#1: sayı) -> sayı"));
        assert!(ir.contains("return (a#0 + b#1)"));
        assert!(ir.contains("expr runtime.yazdir_sayi(Topla@0(10, 20))"));
    }

    #[test]
    fn lowers_loop_control_flow() {
        let ir = lower(
            "\
Ana() {
    döngü (i: sayı = 0; i < 3; i = i + 1) {
        eğer (i == 1) {
            devam;
        }
        yazdir(i);
    }
}
",
        );

        assert!(ir.contains("loop (decl i#0: sayı = 0; (i#0 < 3); set i#0 = (i#0 + 1))"));
        assert!(ir.contains("if (i#0 == 1)"));
        assert!(ir.contains("continue"));
    }

    #[test]
    fn lowers_runtime_string_operations_explicitly() {
        let ir = lower(
            "\
Ana() {
    a: metin = \"Merhaba\";
    yazdir(a);
    yazdir(uzunluk(a));
    yazdir(a == \"Merhaba\");
    yazdir(a != \"Dunya\");
}
",
        );

        assert!(ir.contains("expr runtime.yazdir_metin(a#0)"));
        assert!(ir.contains("runtime.yazdir_sayi(runtime.metin_uzunluk(a#0))"));
        assert!(ir.contains("runtime.yazdir_mantik(runtime.metin_esit(a#0, \"Merhaba\"))"));
        assert!(ir.contains("runtime.yazdir_mantik(runtime.metin_esit_degil(a#0, \"Dunya\"))"));
    }
}
