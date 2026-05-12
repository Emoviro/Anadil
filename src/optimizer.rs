use crate::ast::{BinaryOp, UnaryOp};
use crate::typed::{
    TypedBlock, TypedExpr, TypedExprKind, TypedFunction, TypedIfStmt, TypedLoopPart, TypedLoopStmt,
    TypedProgram, TypedStmt, TypedStmtKind,
};

pub fn optimize_typed_program(program: TypedProgram) -> TypedProgram {
    TypedProgram {
        functions: program
            .functions
            .into_iter()
            .map(optimize_function)
            .collect(),
    }
}

fn optimize_function(function: TypedFunction) -> TypedFunction {
    TypedFunction {
        body: optimize_block(function.body),
        ..function
    }
}

fn optimize_block(block: TypedBlock) -> TypedBlock {
    TypedBlock {
        statements: block
            .statements
            .into_iter()
            .map(optimize_statement)
            .collect(),
        ..block
    }
}

fn optimize_statement(statement: TypedStmt) -> TypedStmt {
    let kind = match statement.kind {
        TypedStmtKind::VarDecl(mut decl) => {
            decl.value = optimize_expr(decl.value);
            TypedStmtKind::VarDecl(decl)
        }
        TypedStmtKind::Assign(mut assign) => {
            assign.value = optimize_expr(assign.value);
            TypedStmtKind::Assign(assign)
        }
        TypedStmtKind::If(if_stmt) => TypedStmtKind::If(optimize_if(if_stmt)),
        TypedStmtKind::Loop(loop_stmt) => TypedStmtKind::Loop(Box::new(optimize_loop(*loop_stmt))),
        TypedStmtKind::Return(value) => TypedStmtKind::Return(value.map(optimize_expr)),
        TypedStmtKind::Expr(expr) => TypedStmtKind::Expr(optimize_expr(expr)),
        TypedStmtKind::Break => TypedStmtKind::Break,
        TypedStmtKind::Continue => TypedStmtKind::Continue,
    };

    TypedStmt { kind, ..statement }
}

fn optimize_if(if_stmt: TypedIfStmt) -> TypedIfStmt {
    TypedIfStmt {
        condition: optimize_expr(if_stmt.condition),
        then_branch: optimize_block(if_stmt.then_branch),
        else_branch: if_stmt.else_branch.map(optimize_block),
        ..if_stmt
    }
}

fn optimize_loop(loop_stmt: TypedLoopStmt) -> TypedLoopStmt {
    TypedLoopStmt {
        init: loop_stmt.init.map(optimize_loop_part),
        condition: loop_stmt.condition.map(optimize_expr),
        step: loop_stmt.step.map(optimize_loop_part),
        body: optimize_block(loop_stmt.body),
        ..loop_stmt
    }
}

fn optimize_loop_part(part: TypedLoopPart) -> TypedLoopPart {
    match part {
        TypedLoopPart::VarDecl(mut decl) => {
            decl.value = optimize_expr(decl.value);
            TypedLoopPart::VarDecl(decl)
        }
        TypedLoopPart::Assign(mut assign) => {
            assign.value = optimize_expr(assign.value);
            TypedLoopPart::Assign(assign)
        }
        TypedLoopPart::Expr(expr) => TypedLoopPart::Expr(optimize_expr(expr)),
    }
}

fn optimize_expr(expr: TypedExpr) -> TypedExpr {
    let span = expr.span;
    let ty = expr.ty;
    let kind = match expr.kind {
        TypedExprKind::Unary { op, expr } => {
            let expr = optimize_expr(*expr);
            fold_unary(op, &expr).unwrap_or_else(|| TypedExprKind::Unary {
                op,
                expr: Box::new(expr),
            })
        }
        TypedExprKind::Binary { left, op, right } => {
            let left = optimize_expr(*left);
            let right = optimize_expr(*right);
            fold_binary(&left, op, &right).unwrap_or_else(|| simplify_binary(left, op, right))
        }
        TypedExprKind::Call { target, args } => TypedExprKind::Call {
            target,
            args: args.into_iter().map(optimize_expr).collect(),
        },
        TypedExprKind::Array(elements) => {
            TypedExprKind::Array(elements.into_iter().map(optimize_expr).collect())
        }
        TypedExprKind::Index { target, index } => TypedExprKind::Index {
            target: Box::new(optimize_expr(*target)),
            index: Box::new(optimize_expr(*index)),
        },
        other => other,
    };

    TypedExpr { span, ty, kind }
}

fn fold_unary(op: UnaryOp, expr: &TypedExpr) -> Option<TypedExprKind> {
    match (op, &expr.kind) {
        (UnaryOp::Negate, TypedExprKind::Number(value)) => {
            value.checked_neg().map(TypedExprKind::Number)
        }
        _ => None,
    }
}

fn fold_binary(left: &TypedExpr, op: BinaryOp, right: &TypedExpr) -> Option<TypedExprKind> {
    match (&left.kind, op, &right.kind) {
        (TypedExprKind::Number(left), BinaryOp::Add, TypedExprKind::Number(right)) => {
            left.checked_add(*right).map(TypedExprKind::Number)
        }
        (TypedExprKind::Number(left), BinaryOp::Subtract, TypedExprKind::Number(right)) => {
            left.checked_sub(*right).map(TypedExprKind::Number)
        }
        (TypedExprKind::Number(left), BinaryOp::Multiply, TypedExprKind::Number(right)) => {
            left.checked_mul(*right).map(TypedExprKind::Number)
        }
        (TypedExprKind::Number(_), BinaryOp::Divide, TypedExprKind::Number(0)) => None,
        (TypedExprKind::Number(left), BinaryOp::Divide, TypedExprKind::Number(right)) => {
            left.checked_div(*right).map(TypedExprKind::Number)
        }
        (TypedExprKind::Number(left), BinaryOp::Equal, TypedExprKind::Number(right)) => {
            Some(TypedExprKind::Bool(left == right))
        }
        (TypedExprKind::Number(left), BinaryOp::NotEqual, TypedExprKind::Number(right)) => {
            Some(TypedExprKind::Bool(left != right))
        }
        (TypedExprKind::Number(left), BinaryOp::Less, TypedExprKind::Number(right)) => {
            Some(TypedExprKind::Bool(left < right))
        }
        (TypedExprKind::Number(left), BinaryOp::Greater, TypedExprKind::Number(right)) => {
            Some(TypedExprKind::Bool(left > right))
        }
        (TypedExprKind::Number(left), BinaryOp::LessEqual, TypedExprKind::Number(right)) => {
            Some(TypedExprKind::Bool(left <= right))
        }
        (TypedExprKind::Number(left), BinaryOp::GreaterEqual, TypedExprKind::Number(right)) => {
            Some(TypedExprKind::Bool(left >= right))
        }
        (TypedExprKind::Bool(left), BinaryOp::Equal, TypedExprKind::Bool(right)) => {
            Some(TypedExprKind::Bool(left == right))
        }
        (TypedExprKind::Bool(left), BinaryOp::NotEqual, TypedExprKind::Bool(right)) => {
            Some(TypedExprKind::Bool(left != right))
        }
        (TypedExprKind::String(left), BinaryOp::Equal, TypedExprKind::String(right)) => {
            Some(TypedExprKind::Bool(left == right))
        }
        (TypedExprKind::String(left), BinaryOp::NotEqual, TypedExprKind::String(right)) => {
            Some(TypedExprKind::Bool(left != right))
        }
        _ => None,
    }
}

fn simplify_binary(left: TypedExpr, op: BinaryOp, right: TypedExpr) -> TypedExprKind {
    match op {
        BinaryOp::Add if is_number(&right, 0) => left.kind,
        BinaryOp::Add if is_number(&left, 0) => right.kind,
        BinaryOp::Subtract if is_number(&right, 0) => left.kind,
        BinaryOp::Multiply if is_number(&right, 1) => left.kind,
        BinaryOp::Multiply if is_number(&left, 1) => right.kind,
        BinaryOp::Multiply if is_number(&right, 0) && is_side_effect_free(&left) => {
            TypedExprKind::Number(0)
        }
        BinaryOp::Multiply if is_number(&left, 0) && is_side_effect_free(&right) => {
            TypedExprKind::Number(0)
        }
        BinaryOp::Divide if is_number(&right, 1) => left.kind,
        _ => TypedExprKind::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        },
    }
}

fn is_number(expr: &TypedExpr, expected: i64) -> bool {
    matches!(expr.kind, TypedExprKind::Number(value) if value == expected)
}

fn is_side_effect_free(expr: &TypedExpr) -> bool {
    match &expr.kind {
        TypedExprKind::Number(_)
        | TypedExprKind::Bool(_)
        | TypedExprKind::String(_)
        | TypedExprKind::Variable(_) => true,
        TypedExprKind::Array(elements) => elements.iter().all(is_side_effect_free),
        TypedExprKind::Index { target, index } => {
            is_side_effect_free(target) && is_side_effect_free(index)
        }
        TypedExprKind::Unary { expr, .. } => is_side_effect_free(expr),
        TypedExprKind::Binary { left, right, .. } => {
            is_side_effect_free(left) && is_side_effect_free(right)
        }
        TypedExprKind::Call { .. } => false,
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;
    use crate::optimizer::optimize_typed_program;
    use crate::parser::Parser;
    use crate::sema::Analyzer;
    use crate::typed::{TypedExprKind, TypedStmtKind};

    fn optimized_main_expr(source: &str) -> TypedExprKind {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().expect("source should lex");
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().expect("source should parse");
        let typed = Analyzer::analyze(&program).expect("source should typecheck");
        let optimized = optimize_typed_program(typed);
        let main = optimized
            .functions
            .iter()
            .find(|function| function.name == "Ana")
            .expect("source should contain Ana");
        let statement = &main.body.statements[0];

        match &statement.kind {
            TypedStmtKind::VarDecl(decl) => decl.value.kind.clone(),
            TypedStmtKind::Expr(expr) => expr.kind.clone(),
            other => panic!("unexpected statement kind: {other:?}"),
        }
    }

    #[test]
    fn folds_numeric_constant_expressions() {
        let expr = optimized_main_expr(
            "\
Ana() {
    x: sayı = 2 + 3 * 4;
}
",
        );

        assert_eq!(expr, TypedExprKind::Number(14));
    }

    #[test]
    fn preserves_division_by_zero_runtime_error() {
        let expr = optimized_main_expr(
            "\
Ana() {
    x: sayı = 10 / 0;
}
",
        );

        assert!(matches!(expr, TypedExprKind::Binary { .. }));
    }

    #[test]
    fn simplifies_identity_arithmetic() {
        let expr = optimized_main_expr(
            "\
Ana() {
    x: sayı = 1 * (0 + 42);
}
",
        );

        assert_eq!(expr, TypedExprKind::Number(42));
    }

    #[test]
    fn folds_bool_and_string_comparisons() {
        let bool_expr = optimized_main_expr(
            "\
Ana() {
    doğru == doğru;
}
",
        );
        let string_expr = optimized_main_expr(
            "\
Ana() {
    \"a\" != \"b\";
}
",
        );

        assert_eq!(bool_expr, TypedExprKind::Bool(true));
        assert_eq!(string_expr, TypedExprKind::Bool(true));
    }

    #[test]
    fn does_not_remove_side_effecting_calls_when_simplifying_zero_multiply() {
        let expr = optimized_main_expr(
            "\
Etiket(x: sayı) -> sayı {
    yazdir(x);
    dön x;
}

Ana() {
    x: sayı = Etiket(7) * 0;
}
",
        );

        assert!(matches!(expr, TypedExprKind::Binary { .. }));
    }
}
