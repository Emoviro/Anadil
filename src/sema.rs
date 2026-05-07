use std::collections::HashMap;
use std::fmt;

use crate::ast::{
    AssignStmt, BinaryOp, Block, Expr, ExprKind, Function, LoopPart, Program, SourceSpan, Stmt,
    StmtKind, Type, VarDecl,
};
use crate::typed::{
    BuiltinFunction, CallTarget, FunctionId, LocalId, LocalKind, TypedAssignStmt, TypedBlock,
    TypedExpr, TypedExprKind, TypedExprType, TypedFunction, TypedIfStmt, TypedLocalRef,
    TypedLoopPart, TypedLoopStmt, TypedParam, TypedProgram, TypedStmt, TypedStmtKind, TypedVarDecl,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticError {
    pub message: String,
    pub span: Option<SourceSpan>,
}

impl SemanticError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span: None,
        }
    }

    fn at(span: SourceSpan, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span: Some(span),
        }
    }
}

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.span {
            Some(span) => write!(
                f,
                "{} (satır {}, sütun {})",
                self.message, span.line, span.column
            ),
            None => f.write_str(&self.message),
        }
    }
}

impl std::error::Error for SemanticError {}

#[derive(Debug, Clone)]
struct FunctionSignature {
    id: FunctionId,
    params: Vec<Type>,
    return_type: Option<Type>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExprType {
    Value(Type),
    Void,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FlowStatus {
    FallsThrough,
    Terminates,
    Returns,
}

#[derive(Debug)]
struct ScopeStack {
    scopes: Vec<HashMap<String, LocalSymbol>>,
}

#[derive(Debug, Clone, Copy)]
struct LocalSymbol {
    id: LocalId,
    ty: Type,
    kind: LocalKind,
}

impl ScopeStack {
    fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    fn declare(&mut self, name: &str, symbol: LocalSymbol) -> bool {
        let current = self
            .scopes
            .last_mut()
            .expect("scope stack must always contain a root scope");

        if current.contains_key(name) {
            false
        } else {
            current.insert(name.to_string(), symbol);
            true
        }
    }

    fn lookup(&self, name: &str) -> Option<LocalSymbol> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
    }
}

#[derive(Debug)]
struct FunctionContext {
    return_type: Option<Type>,
    loop_depth: usize,
    next_local_id: usize,
}

impl FunctionContext {
    fn allocate_local(&mut self) -> LocalId {
        let id = LocalId(self.next_local_id);
        self.next_local_id += 1;
        id
    }
}

// The analyzer validates the parsed AST and prepares it for later compiler stages.
pub struct Analyzer {
    functions: HashMap<String, FunctionSignature>,
}

impl Analyzer {
    fn typed_local_ref(&self, name: &str, symbol: LocalSymbol) -> TypedLocalRef {
        TypedLocalRef {
            id: symbol.id,
            name: name.to_string(),
            ty: symbol.ty,
            kind: symbol.kind,
        }
    }

    pub fn analyze(program: &Program) -> Result<TypedProgram, SemanticError> {
        let mut analyzer = Self {
            functions: HashMap::new(),
        };

        analyzer.collect_function_signatures(program)?;
        analyzer.validate_entry_point()?;

        for function in &program.functions {
            analyzer.analyze_function(function)?;
        }

        analyzer.build_typed_program(program)
    }

    fn collect_function_signatures(&mut self, program: &Program) -> Result<(), SemanticError> {
        for (index, function) in program.functions.iter().enumerate() {
            if BuiltinFunction::from_name(&function.name).is_some() {
                return Err(SemanticError::at(
                    function.span,
                    format!("`{}` adı yerleşik fonksiyon için ayrılmış", function.name),
                ));
            }

            let signature = FunctionSignature {
                id: FunctionId(index),
                params: function.params.iter().map(|param| param.ty).collect(),
                return_type: function.return_type,
            };

            if self
                .functions
                .insert(function.name.clone(), signature)
                .is_some()
            {
                return Err(SemanticError::at(
                    function.span,
                    format!(
                        "`{}` fonksiyonu birden fazla kez tanımlanmış",
                        function.name
                    ),
                ));
            }
        }

        Ok(())
    }

    fn validate_entry_point(&self) -> Result<(), SemanticError> {
        let Some(entry) = self.functions.get("Ana") else {
            return Err(SemanticError::new(
                "Program giriş noktası için parametresiz bir `Ana()` fonksiyonu tanımlanmalı",
            ));
        };

        if !entry.params.is_empty() {
            return Err(SemanticError::new("`Ana()` fonksiyonu parametre alamaz"));
        }

        if entry.return_type.is_some() {
            return Err(SemanticError::new(
                "`Ana()` fonksiyonu dönüş tipi belirtemez",
            ));
        }

        Ok(())
    }

    fn combine_branch_flows(
        &self,
        then_flow: FlowStatus,
        else_flow: Option<FlowStatus>,
    ) -> FlowStatus {
        let Some(else_flow) = else_flow else {
            return FlowStatus::FallsThrough;
        };

        match (then_flow, else_flow) {
            (FlowStatus::Returns, FlowStatus::Returns) => FlowStatus::Returns,
            (FlowStatus::FallsThrough, _) | (_, FlowStatus::FallsThrough) => {
                FlowStatus::FallsThrough
            }
            _ => FlowStatus::Terminates,
        }
    }

    fn analyze_function(&self, function: &Function) -> Result<(), SemanticError> {
        let mut scopes = ScopeStack::new();
        let mut context = FunctionContext {
            return_type: function.return_type,
            loop_depth: 0,
            next_local_id: 0,
        };

        for param in &function.params {
            let symbol = LocalSymbol {
                id: context.allocate_local(),
                ty: param.ty,
                kind: LocalKind::Param,
            };
            if !scopes.declare(&param.name, symbol) {
                return Err(SemanticError::at(
                    param.span,
                    format!("`{}` parametresi birden fazla kez tanımlanmış", param.name),
                ));
            }
        }

        let body_flow = self.analyze_block(&function.body, &mut scopes, &mut context, false)?;

        if function.return_type.is_some() && body_flow != FlowStatus::Returns {
            return Err(SemanticError::at(
                function.span,
                "Dönüş tipi belirtilen fonksiyonda tüm kontrol yolları bir değer döndürmeli",
            ));
        }

        Ok(())
    }

    fn analyze_block(
        &self,
        block: &Block,
        scopes: &mut ScopeStack,
        context: &mut FunctionContext,
        create_scope: bool,
    ) -> Result<FlowStatus, SemanticError> {
        if create_scope {
            scopes.push_scope();
        }

        let result = (|| {
            let mut flow = FlowStatus::FallsThrough;

            for statement in &block.statements {
                if flow != FlowStatus::FallsThrough {
                    return Err(SemanticError::at(
                        statement.span,
                        "Bu ifade önceki kontrol akışı nedeniyle erişilemez",
                    ));
                }

                flow = self.analyze_statement(statement, scopes, context)?;
            }

            Ok(flow)
        })();

        if create_scope {
            scopes.pop_scope();
        }

        result
    }

    fn analyze_statement(
        &self,
        statement: &Stmt,
        scopes: &mut ScopeStack,
        context: &mut FunctionContext,
    ) -> Result<FlowStatus, SemanticError> {
        match &statement.kind {
            StmtKind::VarDecl(decl) => {
                self.analyze_var_decl(decl, scopes, context)?;
                Ok(FlowStatus::FallsThrough)
            }
            StmtKind::Assign(assign) => {
                self.analyze_assignment(assign, scopes, context)?;
                Ok(FlowStatus::FallsThrough)
            }
            StmtKind::If(if_stmt) => {
                let condition_type = self.expect_value_expr(&if_stmt.condition, scopes, context)?;

                if condition_type != Type::Mantik {
                    return Err(SemanticError::at(
                        if_stmt.condition.span,
                        format!(
                            "`eğer` koşulu `mantık` olmalı, bulunan `{}`",
                            condition_type
                        ),
                    ));
                }

                let then_flow = self.analyze_block(&if_stmt.then_branch, scopes, context, true)?;

                let else_flow = if let Some(else_branch) = &if_stmt.else_branch {
                    self.analyze_block(else_branch, scopes, context, true)?
                } else {
                    FlowStatus::FallsThrough
                };

                Ok(self.combine_branch_flows(then_flow, Some(else_flow)))
            }
            StmtKind::Loop(loop_stmt) => {
                scopes.push_scope();

                let result = (|| {
                    if let Some(init) = &loop_stmt.init {
                        self.analyze_loop_part(init, scopes, context)?;
                    }

                    if let Some(condition) = &loop_stmt.condition {
                        let condition_type = self.expect_value_expr(condition, scopes, context)?;

                        if condition_type != Type::Mantik {
                            return Err(SemanticError::at(
                                condition.span,
                                format!(
                                    "`döngü` koşulu `mantık` olmalı, bulunan `{}`",
                                    condition_type
                                ),
                            ));
                        }
                    }

                    if let Some(step) = &loop_stmt.step {
                        self.analyze_loop_part(step, scopes, context)?;
                    }

                    context.loop_depth += 1;
                    let body_result = self.analyze_block(&loop_stmt.body, scopes, context, true);
                    context.loop_depth -= 1;
                    body_result?;

                    // Conservative rule: loops are not assumed to return on every path.
                    Ok(FlowStatus::FallsThrough)
                })();

                scopes.pop_scope();
                result
            }
            StmtKind::Break => {
                if context.loop_depth == 0 {
                    Err(SemanticError::at(
                        statement.span,
                        "`kır` ifadesi yalnızca döngü içinde kullanılabilir",
                    ))
                } else {
                    Ok(FlowStatus::Terminates)
                }
            }
            StmtKind::Continue => {
                if context.loop_depth == 0 {
                    Err(SemanticError::at(
                        statement.span,
                        "`devam` ifadesi yalnızca döngü içinde kullanılabilir",
                    ))
                } else {
                    Ok(FlowStatus::Terminates)
                }
            }
            StmtKind::Return(value) => {
                self.analyze_return(statement.span, value.as_ref(), scopes, context)?;
                Ok(FlowStatus::Returns)
            }
            StmtKind::Expr(expr) => {
                self.infer_expr_type(expr, scopes, context)?;
                Ok(FlowStatus::FallsThrough)
            }
        }
    }

    fn analyze_loop_part(
        &self,
        part: &LoopPart,
        scopes: &mut ScopeStack,
        context: &mut FunctionContext,
    ) -> Result<(), SemanticError> {
        match part {
            LoopPart::VarDecl(decl) => self.analyze_var_decl(decl, scopes, context),
            LoopPart::Assign(assign) => self.analyze_assignment(assign, scopes, context),
            LoopPart::Expr(expr) => {
                self.infer_expr_type(expr, scopes, context)?;
                Ok(())
            }
        }
    }

    fn analyze_var_decl(
        &self,
        decl: &VarDecl,
        scopes: &mut ScopeStack,
        context: &mut FunctionContext,
    ) -> Result<(), SemanticError> {
        let value_type = self.expect_value_expr(&decl.value, scopes, context)?;

        if value_type != decl.ty {
            return Err(SemanticError::at(
                decl.span,
                format!(
                    "`{}` değişkeni `{}` tipinde, ama atanan ifade `{}` tipinde",
                    decl.name, decl.ty, value_type
                ),
            ));
        }

        let symbol = LocalSymbol {
            id: context.allocate_local(),
            ty: decl.ty,
            kind: LocalKind::Variable,
        };
        if !scopes.declare(&decl.name, symbol) {
            return Err(SemanticError::at(
                decl.span,
                format!("`{}` aynı scope içinde yeniden tanımlanmış", decl.name),
            ));
        }

        Ok(())
    }

    fn analyze_assignment(
        &self,
        assign: &AssignStmt,
        scopes: &mut ScopeStack,
        context: &mut FunctionContext,
    ) -> Result<(), SemanticError> {
        let Some(target) = scopes.lookup(&assign.target) else {
            return Err(SemanticError::at(
                assign.span,
                format!("Tanımsız değişkene atama yapılıyor: `{}`", assign.target),
            ));
        };

        let value_type = self.expect_value_expr(&assign.value, scopes, context)?;

        if value_type != target.ty {
            return Err(SemanticError::at(
                assign.span,
                format!(
                    "`{}` değişkeni `{}` tipinde, ama atanan ifade `{}` tipinde",
                    assign.target, target.ty, value_type
                ),
            ));
        }

        Ok(())
    }

    fn analyze_return(
        &self,
        return_span: SourceSpan,
        value: Option<&Expr>,
        scopes: &ScopeStack,
        context: &mut FunctionContext,
    ) -> Result<(), SemanticError> {
        match (context.return_type, value) {
            (None, None) => Ok(()),
            (None, Some(_)) => Err(SemanticError::at(
                return_span,
                "Dönüşsüz fonksiyon değer döndüremez",
            )),
            (Some(expected_type), None) => Err(SemanticError::at(
                return_span,
                format!(
                    "Bu fonksiyon `{}` döndürmeli, `dön` ifadesi ise değer içermiyor",
                    expected_type
                ),
            )),
            (Some(expected_type), Some(expr)) => {
                let actual_type = self.expect_value_expr(expr, scopes, context)?;

                if actual_type != expected_type {
                    return Err(SemanticError::at(
                        return_span,
                        format!(
                            "`dön` ifadesi `{}` tipinde olmalı, bulunan `{}`",
                            expected_type, actual_type
                        ),
                    ));
                }

                Ok(())
            }
        }
    }

    fn expect_value_expr(
        &self,
        expr: &Expr,
        scopes: &ScopeStack,
        context: &FunctionContext,
    ) -> Result<Type, SemanticError> {
        match self.infer_expr_type(expr, scopes, context)? {
            ExprType::Value(ty) => Ok(ty),
            ExprType::Void => Err(SemanticError::at(
                expr.span,
                "Bu ifade değer üretmeli, ancak dönüşsüz",
            )),
        }
    }

    fn infer_expr_type(
        &self,
        expr: &Expr,
        scopes: &ScopeStack,
        context: &FunctionContext,
    ) -> Result<ExprType, SemanticError> {
        match &expr.kind {
            ExprKind::Number(_) => Ok(ExprType::Value(Type::Sayi)),
            ExprKind::Bool(_) => Ok(ExprType::Value(Type::Mantik)),
            ExprKind::Variable(name) => {
                let Some(symbol) = scopes.lookup(name) else {
                    return Err(SemanticError::at(
                        expr.span,
                        format!("Tanımsız değişken kullanılıyor: `{name}`"),
                    ));
                };

                Ok(ExprType::Value(symbol.ty))
            }
            ExprKind::Call { callee, args } => {
                self.analyze_call_expr(expr.span, callee, args, scopes, context)
            }
            ExprKind::Unary { op: _, expr: inner } => {
                let inner_type = self.expect_value_expr(inner, scopes, context)?;
                if inner_type != Type::Sayi {
                    return Err(SemanticError::at(
                        expr.span,
                        format!(
                            "`-` işlemi yalnızca `sayı` operandı kabul eder, bulunan `{}`",
                            inner_type
                        ),
                    ));
                }

                Ok(ExprType::Value(Type::Sayi))
            }
            ExprKind::Binary { left, op, right } => {
                let left_type = self.expect_value_expr(left, scopes, context)?;
                let right_type = self.expect_value_expr(right, scopes, context)?;

                match op {
                    BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide => {
                        self.expect_numeric_operands(expr.span, *op, left_type, right_type)?;
                        Ok(ExprType::Value(Type::Sayi))
                    }
                    BinaryOp::Less
                    | BinaryOp::Greater
                    | BinaryOp::LessEqual
                    | BinaryOp::GreaterEqual => {
                        self.expect_numeric_operands(expr.span, *op, left_type, right_type)?;
                        Ok(ExprType::Value(Type::Mantik))
                    }
                    BinaryOp::Equal | BinaryOp::NotEqual => {
                        if left_type != right_type {
                            return Err(SemanticError::at(
                                expr.span,
                                format!(
                                    "`{}` ve `{}` tipleri `{}` işleminde karşılaştırılamaz",
                                    left_type,
                                    right_type,
                                    self.binary_op_name(*op)
                                ),
                            ));
                        }

                        Ok(ExprType::Value(Type::Mantik))
                    }
                }
            }
        }
    }

    fn analyze_call_expr(
        &self,
        call_span: SourceSpan,
        callee: &str,
        args: &[Expr],
        scopes: &ScopeStack,
        context: &FunctionContext,
    ) -> Result<ExprType, SemanticError> {
        let mut arg_types = Vec::with_capacity(args.len());
        for arg in args {
            arg_types.push(self.expect_value_expr(arg, scopes, context)?);
        }

        if let Some(signature) = self.functions.get(callee) {
            if arg_types.len() != signature.params.len() {
                return Err(SemanticError::at(
                    call_span,
                    format!(
                        "`{}` fonksiyonu {} argüman bekliyor, {} argüman verildi",
                        callee,
                        signature.params.len(),
                        arg_types.len()
                    ),
                ));
            }

            for (index, (arg_type, param_type)) in
                arg_types.iter().zip(signature.params.iter()).enumerate()
            {
                if arg_type != param_type {
                    return Err(SemanticError::at(
                        args[index].span,
                        format!(
                            "`{}` çağrısındaki {}. argüman `{}` olmalı, bulunan `{}`",
                            callee,
                            index + 1,
                            param_type,
                            arg_type
                        ),
                    ));
                }
            }

            return Ok(match signature.return_type {
                Some(return_type) => ExprType::Value(return_type),
                None => ExprType::Void,
            });
        }

        if callee == "yazdir" {
            if arg_types.len() != 1 {
                return Err(SemanticError::at(
                    call_span,
                    "`yazdir` yerleşik fonksiyonu tam olarak 1 argüman bekler",
                ));
            }

            return Ok(ExprType::Void);
        }

        Err(SemanticError::at(
            call_span,
            format!("Tanımsız fonksiyon çağrısı: `{callee}`"),
        ))
    }

    fn expect_numeric_operands(
        &self,
        span: SourceSpan,
        op: BinaryOp,
        left: Type,
        right: Type,
    ) -> Result<(), SemanticError> {
        if left == Type::Sayi && right == Type::Sayi {
            Ok(())
        } else {
            Err(SemanticError::at(
                span,
                format!(
                    "`{}` işlemi yalnızca `sayı` operandları kabul eder, bulunan `{}` ve `{}`",
                    self.binary_op_name(op),
                    left,
                    right
                ),
            ))
        }
    }

    fn binary_op_name(&self, op: BinaryOp) -> &'static str {
        match op {
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
        }
    }

    fn build_typed_program(&self, program: &Program) -> Result<TypedProgram, SemanticError> {
        let mut functions = Vec::with_capacity(program.functions.len());

        for function in &program.functions {
            functions.push(self.build_typed_function(function)?);
        }

        Ok(TypedProgram { functions })
    }

    fn build_typed_function(&self, function: &Function) -> Result<TypedFunction, SemanticError> {
        let mut scopes = ScopeStack::new();
        let mut context = FunctionContext {
            return_type: function.return_type,
            loop_depth: 0,
            next_local_id: 0,
        };

        let mut params = Vec::with_capacity(function.params.len());
        for param in &function.params {
            let local_id = context.allocate_local();
            scopes.declare(
                &param.name,
                LocalSymbol {
                    id: local_id,
                    ty: param.ty,
                    kind: LocalKind::Param,
                },
            );
            params.push(TypedParam {
                local_id,
                span: param.span,
                name: param.name.clone(),
                ty: param.ty,
            });
        }

        let body = self.build_typed_block(&function.body, &mut scopes, &mut context, false)?;
        let function_id = self
            .functions
            .get(&function.name)
            .expect("function signatures are collected before typed build")
            .id;

        Ok(TypedFunction {
            function_id,
            span: function.span,
            name: function.name.clone(),
            params,
            return_type: function.return_type,
            body,
        })
    }

    fn build_typed_block(
        &self,
        block: &Block,
        scopes: &mut ScopeStack,
        context: &mut FunctionContext,
        create_scope: bool,
    ) -> Result<TypedBlock, SemanticError> {
        if create_scope {
            scopes.push_scope();
        }

        let result = (|| {
            let mut flow = FlowStatus::FallsThrough;
            let mut statements = Vec::with_capacity(block.statements.len());

            for statement in &block.statements {
                if flow != FlowStatus::FallsThrough {
                    return Err(SemanticError::at(
                        statement.span,
                        "Bu ifade önceki kontrol akışı nedeniyle erişilemez",
                    ));
                }

                let (typed_stmt, stmt_flow) =
                    self.build_typed_statement(statement, scopes, context)?;
                flow = stmt_flow;
                statements.push(typed_stmt);
            }

            Ok(TypedBlock {
                span: block.span,
                statements,
                definitely_returns: flow == FlowStatus::Returns,
            })
        })();

        if create_scope {
            scopes.pop_scope();
        }

        result
    }

    fn build_typed_statement(
        &self,
        statement: &Stmt,
        scopes: &mut ScopeStack,
        context: &mut FunctionContext,
    ) -> Result<(TypedStmt, FlowStatus), SemanticError> {
        match &statement.kind {
            StmtKind::VarDecl(decl) => Ok((
                TypedStmt {
                    span: statement.span,
                    kind: TypedStmtKind::VarDecl(self.build_typed_var_decl(decl, scopes, context)?),
                },
                FlowStatus::FallsThrough,
            )),
            StmtKind::Assign(assign) => Ok((
                TypedStmt {
                    span: statement.span,
                    kind: TypedStmtKind::Assign(
                        self.build_typed_assignment(assign, scopes, context)?,
                    ),
                },
                FlowStatus::FallsThrough,
            )),
            StmtKind::If(if_stmt) => {
                let condition =
                    self.expect_typed_value_expr(&if_stmt.condition, scopes, context)?;
                let condition_type = match condition.ty {
                    TypedExprType::Value(ty) => ty,
                    TypedExprType::Void => unreachable!(),
                };

                if condition_type != Type::Mantik {
                    return Err(SemanticError::at(
                        if_stmt.condition.span,
                        format!(
                            "`eğer` koşulu `mantık` olmalı, bulunan `{}`",
                            condition_type
                        ),
                    ));
                }

                let then_branch =
                    self.build_typed_block(&if_stmt.then_branch, scopes, context, true)?;
                let else_branch = if let Some(else_branch) = &if_stmt.else_branch {
                    Some(self.build_typed_block(else_branch, scopes, context, true)?)
                } else {
                    None
                };

                let then_flow = if then_branch.definitely_returns {
                    FlowStatus::Returns
                } else {
                    FlowStatus::FallsThrough
                };
                let else_flow = else_branch.as_ref().map(|branch| {
                    if branch.definitely_returns {
                        FlowStatus::Returns
                    } else {
                        FlowStatus::FallsThrough
                    }
                });
                let flow = self.combine_branch_flows(then_flow, else_flow);

                Ok((
                    TypedStmt {
                        span: statement.span,
                        kind: TypedStmtKind::If(TypedIfStmt {
                            span: if_stmt.span,
                            condition,
                            then_branch,
                            else_branch,
                        }),
                    },
                    flow,
                ))
            }
            StmtKind::Loop(loop_stmt) => {
                scopes.push_scope();

                let result = (|| {
                    let init = if let Some(init) = &loop_stmt.init {
                        Some(self.build_typed_loop_part(init, scopes, context)?)
                    } else {
                        None
                    };

                    let condition = if let Some(condition) = &loop_stmt.condition {
                        let typed_condition =
                            self.expect_typed_value_expr(condition, scopes, context)?;
                        let condition_type = match typed_condition.ty {
                            TypedExprType::Value(ty) => ty,
                            TypedExprType::Void => unreachable!(),
                        };

                        if condition_type != Type::Mantik {
                            return Err(SemanticError::at(
                                condition.span,
                                format!(
                                    "`döngü` koşulu `mantık` olmalı, bulunan `{}`",
                                    condition_type
                                ),
                            ));
                        }

                        Some(typed_condition)
                    } else {
                        None
                    };

                    let step = if let Some(step) = &loop_stmt.step {
                        Some(self.build_typed_loop_part(step, scopes, context)?)
                    } else {
                        None
                    };

                    context.loop_depth += 1;
                    let body = self.build_typed_block(&loop_stmt.body, scopes, context, true);
                    context.loop_depth -= 1;

                    Ok((
                        TypedStmt {
                            span: statement.span,
                            kind: TypedStmtKind::Loop(Box::new(TypedLoopStmt {
                                span: loop_stmt.span,
                                init,
                                condition,
                                step,
                                body: body?,
                            })),
                        },
                        FlowStatus::FallsThrough,
                    ))
                })();

                scopes.pop_scope();
                result
            }
            StmtKind::Break => Ok((
                TypedStmt {
                    span: statement.span,
                    kind: TypedStmtKind::Break,
                },
                FlowStatus::Terminates,
            )),
            StmtKind::Continue => Ok((
                TypedStmt {
                    span: statement.span,
                    kind: TypedStmtKind::Continue,
                },
                FlowStatus::Terminates,
            )),
            StmtKind::Return(value) => Ok((
                TypedStmt {
                    span: statement.span,
                    kind: TypedStmtKind::Return(self.build_typed_return(
                        statement.span,
                        value.as_ref(),
                        scopes,
                        context,
                    )?),
                },
                FlowStatus::Returns,
            )),
            StmtKind::Expr(expr) => Ok((
                TypedStmt {
                    span: statement.span,
                    kind: TypedStmtKind::Expr(self.build_typed_expr(expr, scopes, context)?),
                },
                FlowStatus::FallsThrough,
            )),
        }
    }

    fn build_typed_loop_part(
        &self,
        part: &LoopPart,
        scopes: &mut ScopeStack,
        context: &mut FunctionContext,
    ) -> Result<TypedLoopPart, SemanticError> {
        match part {
            LoopPart::VarDecl(decl) => Ok(TypedLoopPart::VarDecl(
                self.build_typed_var_decl(decl, scopes, context)?,
            )),
            LoopPart::Assign(assign) => Ok(TypedLoopPart::Assign(
                self.build_typed_assignment(assign, scopes, context)?,
            )),
            LoopPart::Expr(expr) => Ok(TypedLoopPart::Expr(
                self.build_typed_expr(expr, scopes, context)?,
            )),
        }
    }

    fn build_typed_var_decl(
        &self,
        decl: &VarDecl,
        scopes: &mut ScopeStack,
        context: &mut FunctionContext,
    ) -> Result<TypedVarDecl, SemanticError> {
        let value = self.expect_typed_value_expr(&decl.value, scopes, context)?;
        let value_type = match value.ty {
            TypedExprType::Value(ty) => ty,
            TypedExprType::Void => unreachable!(),
        };

        if value_type != decl.ty {
            return Err(SemanticError::at(
                decl.span,
                format!(
                    "`{}` değişkeni `{}` tipinde, ama atanan ifade `{}` tipinde",
                    decl.name, decl.ty, value_type
                ),
            ));
        }

        let local_id = context.allocate_local();
        scopes.declare(
            &decl.name,
            LocalSymbol {
                id: local_id,
                ty: decl.ty,
                kind: LocalKind::Variable,
            },
        );

        Ok(TypedVarDecl {
            local_id,
            span: decl.span,
            name: decl.name.clone(),
            ty: decl.ty,
            value,
        })
    }

    fn build_typed_assignment(
        &self,
        assign: &AssignStmt,
        scopes: &mut ScopeStack,
        context: &mut FunctionContext,
    ) -> Result<TypedAssignStmt, SemanticError> {
        let target = scopes.lookup(&assign.target).ok_or_else(|| {
            SemanticError::at(
                assign.span,
                format!("Tanımsız değişkene atama yapılıyor: `{}`", assign.target),
            )
        })?;

        let value = self.expect_typed_value_expr(&assign.value, scopes, context)?;
        let value_type = match value.ty {
            TypedExprType::Value(ty) => ty,
            TypedExprType::Void => unreachable!(),
        };

        if value_type != target.ty {
            return Err(SemanticError::at(
                assign.span,
                format!(
                    "`{}` değişkeni `{}` tipinde, ama atanan ifade `{}` tipinde",
                    assign.target, target.ty, value_type
                ),
            ));
        }

        Ok(TypedAssignStmt {
            span: assign.span,
            target: self.typed_local_ref(&assign.target, target),
            value,
        })
    }

    fn build_typed_return(
        &self,
        return_span: SourceSpan,
        value: Option<&Expr>,
        scopes: &ScopeStack,
        context: &mut FunctionContext,
    ) -> Result<Option<TypedExpr>, SemanticError> {
        match (context.return_type, value) {
            (None, None) => Ok(None),
            (None, Some(_)) => Err(SemanticError::at(
                return_span,
                "Dönüşsüz fonksiyon değer döndüremez",
            )),
            (Some(expected_type), None) => Err(SemanticError::at(
                return_span,
                format!(
                    "Bu fonksiyon `{}` döndürmeli, `dön` ifadesi ise değer içermiyor",
                    expected_type
                ),
            )),
            (Some(expected_type), Some(expr)) => {
                let typed_expr = self.expect_typed_value_expr(expr, scopes, context)?;
                let actual_type = match typed_expr.ty {
                    TypedExprType::Value(ty) => ty,
                    TypedExprType::Void => unreachable!(),
                };

                if actual_type != expected_type {
                    return Err(SemanticError::at(
                        return_span,
                        format!(
                            "`dön` ifadesi `{}` tipinde olmalı, bulunan `{}`",
                            expected_type, actual_type
                        ),
                    ));
                }

                Ok(Some(typed_expr))
            }
        }
    }

    fn expect_typed_value_expr(
        &self,
        expr: &Expr,
        scopes: &ScopeStack,
        context: &FunctionContext,
    ) -> Result<TypedExpr, SemanticError> {
        let typed_expr = self.build_typed_expr(expr, scopes, context)?;

        match typed_expr.ty {
            TypedExprType::Value(_) => Ok(typed_expr),
            TypedExprType::Void => Err(SemanticError::at(
                expr.span,
                "Bu ifade değer üretmeli, ancak dönüşsüz",
            )),
        }
    }

    fn build_typed_expr(
        &self,
        expr: &Expr,
        scopes: &ScopeStack,
        context: &FunctionContext,
    ) -> Result<TypedExpr, SemanticError> {
        match &expr.kind {
            ExprKind::Number(value) => Ok(TypedExpr {
                span: expr.span,
                ty: TypedExprType::Value(Type::Sayi),
                kind: TypedExprKind::Number(*value),
            }),
            ExprKind::Bool(value) => Ok(TypedExpr {
                span: expr.span,
                ty: TypedExprType::Value(Type::Mantik),
                kind: TypedExprKind::Bool(*value),
            }),
            ExprKind::Variable(name) => {
                let symbol = scopes.lookup(name).ok_or_else(|| {
                    SemanticError::at(
                        expr.span,
                        format!("Tanımsız değişken kullanılıyor: `{name}`"),
                    )
                })?;

                Ok(TypedExpr {
                    span: expr.span,
                    ty: TypedExprType::Value(symbol.ty),
                    kind: TypedExprKind::Variable(self.typed_local_ref(name, symbol)),
                })
            }
            ExprKind::Call { callee, args } => {
                self.build_typed_call_expr(expr.span, callee, args, scopes, context)
            }
            ExprKind::Unary { op, expr: inner } => {
                let inner = self.expect_typed_value_expr(inner, scopes, context)?;
                let inner_type = match inner.ty {
                    TypedExprType::Value(ty) => ty,
                    TypedExprType::Void => unreachable!(),
                };

                if inner_type != Type::Sayi {
                    return Err(SemanticError::at(
                        expr.span,
                        format!(
                            "`-` işlemi yalnızca `sayı` operandı kabul eder, bulunan `{}`",
                            inner_type
                        ),
                    ));
                }

                Ok(TypedExpr {
                    span: expr.span,
                    ty: TypedExprType::Value(Type::Sayi),
                    kind: TypedExprKind::Unary {
                        op: *op,
                        expr: Box::new(inner),
                    },
                })
            }
            ExprKind::Binary { left, op, right } => {
                let left = self.expect_typed_value_expr(left, scopes, context)?;
                let right = self.expect_typed_value_expr(right, scopes, context)?;

                let left_type = match left.ty {
                    TypedExprType::Value(ty) => ty,
                    TypedExprType::Void => unreachable!(),
                };
                let right_type = match right.ty {
                    TypedExprType::Value(ty) => ty,
                    TypedExprType::Void => unreachable!(),
                };

                let result_type = match op {
                    BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide => {
                        self.expect_numeric_operands(expr.span, *op, left_type, right_type)?;
                        TypedExprType::Value(Type::Sayi)
                    }
                    BinaryOp::Less
                    | BinaryOp::Greater
                    | BinaryOp::LessEqual
                    | BinaryOp::GreaterEqual => {
                        self.expect_numeric_operands(expr.span, *op, left_type, right_type)?;
                        TypedExprType::Value(Type::Mantik)
                    }
                    BinaryOp::Equal | BinaryOp::NotEqual => {
                        if left_type != right_type {
                            return Err(SemanticError::at(
                                expr.span,
                                format!(
                                    "`{}` ve `{}` tipleri `{}` işleminde karşılaştırılamaz",
                                    left_type,
                                    right_type,
                                    self.binary_op_name(*op)
                                ),
                            ));
                        }

                        TypedExprType::Value(Type::Mantik)
                    }
                };

                Ok(TypedExpr {
                    span: expr.span,
                    ty: result_type,
                    kind: TypedExprKind::Binary {
                        left: Box::new(left),
                        op: *op,
                        right: Box::new(right),
                    },
                })
            }
        }
    }

    fn build_typed_call_expr(
        &self,
        call_span: SourceSpan,
        callee: &str,
        args: &[Expr],
        scopes: &ScopeStack,
        context: &FunctionContext,
    ) -> Result<TypedExpr, SemanticError> {
        let mut typed_args = Vec::with_capacity(args.len());
        for arg in args {
            typed_args.push(self.expect_typed_value_expr(arg, scopes, context)?);
        }

        if let Some(signature) = self.functions.get(callee) {
            if typed_args.len() != signature.params.len() {
                return Err(SemanticError::at(
                    call_span,
                    format!(
                        "`{}` fonksiyonu {} argüman bekliyor, {} argüman verildi",
                        callee,
                        signature.params.len(),
                        typed_args.len()
                    ),
                ));
            }

            for (index, (arg, param_type)) in
                typed_args.iter().zip(signature.params.iter()).enumerate()
            {
                let arg_type = match arg.ty {
                    TypedExprType::Value(ty) => ty,
                    TypedExprType::Void => unreachable!(),
                };

                if arg_type != *param_type {
                    return Err(SemanticError::at(
                        arg.span,
                        format!(
                            "`{}` çağrısındaki {}. argüman `{}` olmalı, bulunan `{}`",
                            callee,
                            index + 1,
                            param_type,
                            arg_type
                        ),
                    ));
                }
            }

            return Ok(TypedExpr {
                span: call_span,
                ty: match signature.return_type {
                    Some(ty) => TypedExprType::Value(ty),
                    None => TypedExprType::Void,
                },
                kind: TypedExprKind::Call {
                    target: CallTarget::Function {
                        function_id: signature.id,
                        name: callee.to_string(),
                    },
                    args: typed_args,
                },
            });
        }

        if let Some(builtin) = BuiltinFunction::from_name(callee) {
            match builtin {
                BuiltinFunction::Yazdir => {
                    if typed_args.len() != 1 {
                        return Err(SemanticError::at(
                            call_span,
                            "`yazdir` yerleşik fonksiyonu tam olarak 1 argüman bekler",
                        ));
                    }
                }
            }

            return Ok(TypedExpr {
                span: call_span,
                ty: match builtin.return_type() {
                    Some(ty) => TypedExprType::Value(ty),
                    None => TypedExprType::Void,
                },
                kind: TypedExprKind::Call {
                    target: CallTarget::Builtin(builtin),
                    args: typed_args,
                },
            });
        }

        Err(SemanticError::at(
            call_span,
            format!("Tanımsız fonksiyon çağrısı: `{callee}`"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::Analyzer;
    use crate::ast::Type;
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::typed::{BuiltinFunction, CallTarget, TypedExprKind, TypedExprType, TypedStmtKind};

    fn analyze(source: &str) -> Result<crate::typed::TypedProgram, String> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().map_err(|error| error.to_string())?;
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().map_err(|error| error.to_string())?;
        Analyzer::analyze(&program).map_err(|error| error.to_string())
    }

    #[test]
    fn builds_typed_call_targets() {
        let source = r#"
Topla(a: sayı, b: sayı) -> sayı {
    dön a + b;
}

Ana() {
    sonuc: sayı = Topla(1, 2);
    yazdir(sonuc);
}
"#;

        let program = analyze(source).expect("semantic analysis should succeed");
        let main_function = program
            .functions
            .iter()
            .find(|function| function.name == "Ana")
            .expect("Ana function should exist");

        let TypedStmtKind::VarDecl(var_decl) = &main_function.body.statements[0].kind else {
            panic!("first statement should be a typed variable declaration");
        };

        let TypedExprKind::Call { target, .. } = &var_decl.value.kind else {
            panic!("variable initializer should be a typed function call");
        };
        assert_eq!(
            target,
            &CallTarget::Function {
                function_id: crate::typed::FunctionId(0),
                name: "Topla".to_string(),
            }
        );

        let TypedStmtKind::Expr(expr) = &main_function.body.statements[1].kind else {
            panic!("second statement should be a typed expression");
        };

        let TypedExprKind::Call { target, .. } = &expr.kind else {
            panic!("expression statement should be a builtin call");
        };
        assert_eq!(target, &CallTarget::Builtin(BuiltinFunction::Yazdir));
    }

    #[test]
    fn types_function_call_result() {
        let source = r#"
Topla(a: sayı, b: sayı) -> sayı {
    dön a + b;
}

Ana() {
    sonuc: sayı = Topla(1, 2);
    yazdir(sonuc);
}
"#;

        let program = analyze(source).expect("semantic analysis should succeed");
        let main_function = program
            .functions
            .iter()
            .find(|function| function.name == "Ana")
            .expect("Ana function should exist");

        let TypedStmtKind::VarDecl(var_decl) = &main_function.body.statements[0].kind else {
            panic!("first statement should be a typed variable declaration");
        };

        assert_eq!(var_decl.ty, Type::Sayi);
        assert_eq!(var_decl.value.ty, TypedExprType::Value(Type::Sayi));
    }

    #[test]
    fn rejects_unreachable_statement_after_return() {
        let source = "\
Topla(a: say\u{131}, b: say\u{131}) -> say\u{131} {\n\
    d\u{f6}n a + b;\n\
    yazdir(a);\n\
}\n\
\n\
Ana() {\n\
    yazdir(1);\n\
}\n";

        let error = analyze(source).expect_err("semantic analysis should fail");
        assert!(error.contains("ifade"));
        assert!(error.contains("satır 3, sütun 1"));
    }

    #[test]
    fn rejects_unreachable_statement_after_continue() {
        let source = "\
Ana() {\n\
    d\u{f6}ng\u{fc} (do\u{11f}ru) {\n\
        devam;\n\
        yazdir(1);\n\
    }\n\
}\n";

        let error = analyze(source).expect_err("semantic analysis should fail");
        assert!(error.contains("ifade"));
        assert!(error.contains("satır 4, sütun 1"));
    }

    #[test]
    fn rejects_entry_point_return_type() {
        let source = r#"
Ana() -> sayı {
    dön 1;
}
"#;

        let error = analyze(source).expect_err("semantic analysis should fail");
        assert!(error.contains("dönüş tipi"));
    }

    #[test]
    fn rejects_reserved_builtin_name_as_function() {
        let source = r#"
yazdir(x: sayı) {
}

Ana() {
}
"#;

        let error = analyze(source).expect_err("semantic analysis should fail");
        assert!(error.contains("yerleşik"));
    }

    #[test]
    fn rejects_using_void_builtin_as_value() {
        let source = r#"
Ana() {
    sonuc: sayı = yazdir(1);
}
"#;

        let error = analyze(source).expect_err("semantic analysis should fail");
        assert!(error.contains("değer üretmeli"));
    }

    #[test]
    fn accepts_valid_program() {
        let source = r#"
Topla(a: sayı, b: sayı) -> sayı {
    dön a + b;
}

Ana() {
    sonuc: sayı = Topla(10, 20);
    döngü (i: sayı = 0; i < 3; i = i + 1) {
        eğer (i == 1) {
            devam;
        }
        yazdir(sonuc);
    }
}
"#;

        assert!(analyze(source).is_ok());
    }

    #[test]
    fn accepts_if_else_when_both_branches_return() {
        let source = r#"
Karar(x: mantık) -> sayı {
    eğer (x) {
        dön 1;
    } değilse {
        dön 0;
    }
}

Ana() {
    sonuc: sayı = Karar(doğru);
    yazdir(sonuc);
}
"#;

        assert!(analyze(source).is_ok());
    }

    #[test]
    fn rejects_missing_entry_point() {
        let source = r#"
Topla(a: sayı, b: sayı) -> sayı {
    dön a + b;
}
"#;

        let error = analyze(source).expect_err("semantic analysis should fail");
        assert!(error.contains("Ana()"));
    }

    #[test]
    fn reports_line_and_column_for_type_mismatch() {
        let source = r#"
Ana() {
    x: sayı = doğru;
}
"#;

        let error = analyze(source).expect_err("semantic analysis should fail");
        assert!(error.contains("satır 3, sütun 5"));
    }

    #[test]
    fn reports_line_and_column_for_break_outside_loop() {
        let source = r#"
Ana() {
    kır;
}
"#;

        let error = analyze(source).expect_err("semantic analysis should fail");
        assert!(error.contains("satır 3, sütun 5"));
    }

    #[test]
    fn rejects_wrong_argument_count() {
        let source = r#"
Topla(a: sayı, b: sayı) -> sayı {
    dön a + b;
}

Ana() {
    sonuc: sayı = Topla(10);
}
"#;

        let error = analyze(source).expect_err("semantic analysis should fail");
        assert!(error.contains("2 argüman bekliyor"));
    }

    #[test]
    fn rejects_return_value_in_void_function() {
        let source = r#"
Ana() {
    dön 10;
}
"#;

        let error = analyze(source).expect_err("semantic analysis should fail");
        assert!(error.contains("satır 3, sütun 5"));
    }

    #[test]
    fn rejects_non_void_function_when_some_paths_do_not_return() {
        let source = r#"
Karar(x: mantık) -> sayı {
    eğer (x) {
        dön 1;
    }
}

Ana() {
    yazdir(1);
}
"#;

        let error = analyze(source).expect_err("semantic analysis should fail");
        assert!(error.contains("tüm kontrol yolları"));
    }

    #[test]
    fn accepts_unary_minus_for_numbers() {
        let source = r#"
Ana() {
    x: sayı = -10;
    yazdir(10 + -x);
}
"#;

        assert!(analyze(source).is_ok());
    }

    #[test]
    fn rejects_unary_minus_for_bool() {
        let source = r#"
Ana() {
    yazdir(-doğru);
}
"#;

        let error = analyze(source).expect_err("semantic analysis should fail");
        assert!(error.contains("yalnızca `sayı`"));
    }
}
