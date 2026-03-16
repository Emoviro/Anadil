use std::collections::HashMap;
use std::fmt;

use crate::ast::{
    AssignStmt, BinaryOp, Block, Expr, ExprKind, Function, LoopPart, Program, SourceSpan, Stmt,
    StmtKind, Type, VarDecl,
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
    params: Vec<Type>,
    return_type: Option<Type>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExprType {
    Value(Type),
    Void,
}

#[derive(Debug)]
struct ScopeStack {
    scopes: Vec<HashMap<String, Type>>,
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

    fn declare(&mut self, name: &str, ty: Type) -> bool {
        let current = self
            .scopes
            .last_mut()
            .expect("scope stack must always contain a root scope");

        if current.contains_key(name) {
            false
        } else {
            current.insert(name.to_string(), ty);
            true
        }
    }

    fn lookup(&self, name: &str) -> Option<Type> {
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
    saw_value_return: bool,
}

// The analyzer validates the parsed AST and prepares it for later compiler stages.
pub struct Analyzer {
    functions: HashMap<String, FunctionSignature>,
}

impl Analyzer {
    pub fn analyze(program: &Program) -> Result<(), SemanticError> {
        let mut analyzer = Self {
            functions: HashMap::new(),
        };

        analyzer.collect_function_signatures(program)?;
        analyzer.validate_entry_point()?;

        for function in &program.functions {
            analyzer.analyze_function(function)?;
        }

        Ok(())
    }

    fn collect_function_signatures(&mut self, program: &Program) -> Result<(), SemanticError> {
        for function in &program.functions {
            let signature = FunctionSignature {
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

        Ok(())
    }

    fn analyze_function(&self, function: &Function) -> Result<(), SemanticError> {
        let mut scopes = ScopeStack::new();
        let mut context = FunctionContext {
            return_type: function.return_type,
            loop_depth: 0,
            saw_value_return: false,
        };

        for param in &function.params {
            if !scopes.declare(&param.name, param.ty) {
                return Err(SemanticError::at(
                    param.span,
                    format!("`{}` parametresi birden fazla kez tanımlanmış", param.name),
                ));
            }
        }

        self.analyze_block(&function.body, &mut scopes, &mut context, false)?;

        if context.return_type.is_some() && !context.saw_value_return {
            return Err(SemanticError::at(
                function.span,
                "Dönüş tipi belirtilen fonksiyonda en az bir `dön` ifadesi bulunmalı",
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
    ) -> Result<(), SemanticError> {
        if create_scope {
            scopes.push_scope();
        }

        for statement in &block.statements {
            self.analyze_statement(statement, scopes, context)?;
        }

        if create_scope {
            scopes.pop_scope();
        }

        Ok(())
    }

    fn analyze_statement(
        &self,
        statement: &Stmt,
        scopes: &mut ScopeStack,
        context: &mut FunctionContext,
    ) -> Result<(), SemanticError> {
        match &statement.kind {
            StmtKind::VarDecl(decl) => self.analyze_var_decl(decl, scopes, context),
            StmtKind::Assign(assign) => self.analyze_assignment(assign, scopes, context),
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

                self.analyze_block(&if_stmt.then_branch, scopes, context, true)?;

                if let Some(else_branch) = &if_stmt.else_branch {
                    self.analyze_block(else_branch, scopes, context, true)?;
                }

                Ok(())
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
                    body_result
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
                    Ok(())
                }
            }
            StmtKind::Continue => {
                if context.loop_depth == 0 {
                    Err(SemanticError::at(
                        statement.span,
                        "`devam` ifadesi yalnızca döngü içinde kullanılabilir",
                    ))
                } else {
                    Ok(())
                }
            }
            StmtKind::Return(value) => {
                self.analyze_return(statement.span, value.as_ref(), scopes, context)
            }
            StmtKind::Expr(expr) => {
                self.infer_expr_type(expr, scopes, context)?;
                Ok(())
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

        if !scopes.declare(&decl.name, decl.ty) {
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
        let Some(target_type) = scopes.lookup(&assign.target) else {
            return Err(SemanticError::at(
                assign.span,
                format!("Tanımsız değişkene atama yapılıyor: `{}`", assign.target),
            ));
        };

        let value_type = self.expect_value_expr(&assign.value, scopes, context)?;

        if value_type != target_type {
            return Err(SemanticError::at(
                assign.span,
                format!(
                    "`{}` değişkeni `{}` tipinde, ama atanan ifade `{}` tipinde",
                    assign.target, target_type, value_type
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

                context.saw_value_return = true;
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
                let Some(ty) = scopes.lookup(name) else {
                    return Err(SemanticError::at(
                        expr.span,
                        format!("Tanımsız değişken kullanılıyor: `{name}`"),
                    ));
                };

                Ok(ExprType::Value(ty))
            }
            ExprKind::Call { callee, args } => {
                self.analyze_call_expr(expr.span, callee, args, scopes, context)
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
}

#[cfg(test)]
mod tests {
    use super::Analyzer;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn analyze(source: &str) -> Result<(), String> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().map_err(|error| error.to_string())?;
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().map_err(|error| error.to_string())?;
        Analyzer::analyze(&program).map_err(|error| error.to_string())
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
}
