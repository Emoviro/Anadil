use std::collections::HashMap;
use std::fmt;

use crate::ast::{BinaryOp, SourceSpan};
use crate::typed::{
    BuiltinFunction, CallTarget, TypedAssignStmt, TypedBlock, TypedExpr, TypedExprKind,
    TypedFunction, TypedLoopPart, TypedProgram, TypedStmt, TypedStmtKind, TypedVarDecl,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeError {
    pub message: String,
    pub span: Option<SourceSpan>,
}

impl RuntimeError {
    fn at(span: SourceSpan, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span: Some(span),
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.span {
            Some(span) => write!(
                f,
                "{} (satir {}, sutun {})",
                self.message, span.line, span.column
            ),
            None => f.write_str(&self.message),
        }
    }
}

impl std::error::Error for RuntimeError {}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Value {
    Number(i64),
    Bool(bool),
    String(String),
    Array(Vec<Value>),
}

impl Value {
    fn render(&self) -> String {
        match self {
            Value::Number(value) => value.to_string(),
            Value::Bool(true) => "do\u{011f}ru".to_string(),
            Value::Bool(false) => "yanl\u{0131}\u{015f}".to_string(),
            Value::String(value) => value.clone(),
            Value::Array(values) => {
                let rendered = values
                    .iter()
                    .map(Value::render)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{{{rendered}}}")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Flow {
    None,
    Break,
    Continue,
    Return(Option<Value>),
}

pub struct Interpreter<'a> {
    functions: HashMap<&'a str, &'a TypedFunction>,
    scopes: Vec<HashMap<String, Value>>,
    output: Vec<String>,
}

impl<'a> Interpreter<'a> {
    pub fn run(program: &'a TypedProgram) -> Result<String, RuntimeError> {
        let mut interpreter = Self::new(program);
        interpreter.call_entry()?;
        Ok(interpreter.output.join("\n"))
    }

    fn new(program: &'a TypedProgram) -> Self {
        let functions = program
            .functions
            .iter()
            .map(|function| (function.name.as_str(), function))
            .collect();

        Self {
            functions,
            scopes: Vec::new(),
            output: Vec::new(),
        }
    }

    fn call_entry(&mut self) -> Result<(), RuntimeError> {
        self.call_function("Ana", Vec::new()).map(|_| ())
    }

    fn call_function(
        &mut self,
        name: &str,
        args: Vec<Value>,
    ) -> Result<Option<Value>, RuntimeError> {
        let function = *self.functions.get(name).ok_or_else(|| RuntimeError {
            message: format!("Fonksiyon bulunamadi: `{name}`"),
            span: None,
        })?;

        self.scopes.push(HashMap::new());

        for (param, arg) in function.params.iter().zip(args) {
            self.declare(param.name.clone(), arg);
        }

        let flow = self.execute_block(&function.body, false)?;
        self.scopes.pop();

        match flow {
            Flow::None => Ok(None),
            Flow::Return(value) => Ok(value),
            Flow::Break => Err(RuntimeError::at(
                function.span,
                "`kir` fonksiyon disina tasamaz",
            )),
            Flow::Continue => Err(RuntimeError::at(
                function.span,
                "`devam` fonksiyon disina tasamaz",
            )),
        }
    }

    fn execute_block(
        &mut self,
        block: &TypedBlock,
        create_scope: bool,
    ) -> Result<Flow, RuntimeError> {
        if create_scope {
            self.scopes.push(HashMap::new());
        }

        let result = (|| {
            for statement in &block.statements {
                let flow = self.execute_statement(statement)?;
                if flow != Flow::None {
                    return Ok(flow);
                }
            }

            Ok(Flow::None)
        })();

        if create_scope {
            self.scopes.pop();
        }

        result
    }

    fn execute_statement(&mut self, statement: &TypedStmt) -> Result<Flow, RuntimeError> {
        match &statement.kind {
            TypedStmtKind::VarDecl(decl) => {
                self.execute_var_decl(decl)?;
                Ok(Flow::None)
            }
            TypedStmtKind::Assign(assign) => {
                self.execute_assignment(assign)?;
                Ok(Flow::None)
            }
            TypedStmtKind::If(if_stmt) => {
                if self.eval_bool(&if_stmt.condition)? {
                    self.execute_block(&if_stmt.then_branch, true)
                } else if let Some(else_branch) = &if_stmt.else_branch {
                    self.execute_block(else_branch, true)
                } else {
                    Ok(Flow::None)
                }
            }
            TypedStmtKind::Loop(loop_stmt) => {
                self.scopes.push(HashMap::new());

                let result = (|| {
                    if let Some(init) = &loop_stmt.init {
                        self.execute_loop_part(init)?;
                    }

                    loop {
                        if let Some(condition) = &loop_stmt.condition {
                            if !self.eval_bool(condition)? {
                                break;
                            }
                        }

                        match self.execute_block(&loop_stmt.body, true)? {
                            Flow::None => {}
                            Flow::Break => break,
                            Flow::Continue => {}
                            Flow::Return(value) => return Ok(Flow::Return(value)),
                        }

                        if let Some(step) = &loop_stmt.step {
                            self.execute_loop_part(step)?;
                        }
                    }

                    Ok(Flow::None)
                })();

                self.scopes.pop();
                result
            }
            TypedStmtKind::Break => Ok(Flow::Break),
            TypedStmtKind::Continue => Ok(Flow::Continue),
            TypedStmtKind::Return(value) => {
                let value = value
                    .as_ref()
                    .map(|expr| self.eval_value(expr))
                    .transpose()?;
                Ok(Flow::Return(value))
            }
            TypedStmtKind::Expr(expr) => {
                self.eval_expr(expr)?;
                Ok(Flow::None)
            }
        }
    }

    fn execute_loop_part(&mut self, part: &TypedLoopPart) -> Result<(), RuntimeError> {
        match part {
            TypedLoopPart::VarDecl(decl) => self.execute_var_decl(decl),
            TypedLoopPart::Assign(assign) => self.execute_assignment(assign),
            TypedLoopPart::Expr(expr) => {
                self.eval_expr(expr)?;
                Ok(())
            }
        }
    }

    fn execute_var_decl(&mut self, decl: &TypedVarDecl) -> Result<(), RuntimeError> {
        let value = self.eval_value(&decl.value)?;
        self.declare(decl.name.clone(), value);
        Ok(())
    }

    fn execute_assignment(&mut self, assign: &TypedAssignStmt) -> Result<(), RuntimeError> {
        let value = self.eval_value(&assign.value)?;
        self.assign(&assign.target.name, value).ok_or_else(|| {
            RuntimeError::at(
                assign.span,
                format!("Degisken bulunamadi: `{}`", assign.target.name),
            )
        })
    }

    fn eval_expr(&mut self, expr: &TypedExpr) -> Result<Option<Value>, RuntimeError> {
        match &expr.kind {
            TypedExprKind::Number(value) => Ok(Some(Value::Number(*value))),
            TypedExprKind::Bool(value) => Ok(Some(Value::Bool(*value))),
            TypedExprKind::String(value) => Ok(Some(Value::String(value.clone()))),
            TypedExprKind::Array(elements) => {
                let mut values = Vec::with_capacity(elements.len());
                for element in elements {
                    values.push(self.eval_value(element)?);
                }
                Ok(Some(Value::Array(values)))
            }
            TypedExprKind::Variable(local) => self.lookup(&local.name).map(Some).ok_or_else(|| {
                RuntimeError::at(expr.span, format!("Degisken bulunamadi: `{}`", local.name))
            }),
            TypedExprKind::Index { target, index } => {
                let target = self.eval_value(target)?;
                let index = self.eval_value(index)?;
                let Value::Array(values) = target else {
                    return Err(RuntimeError::at(
                        expr.span,
                        "Index okuma icin dizi bekleniyordu",
                    ));
                };
                let Value::Number(index) = index else {
                    return Err(RuntimeError::at(expr.span, "Dizi index'i sayi olmali"));
                };
                if index < 0 {
                    return Err(RuntimeError::at(expr.span, "Dizi index'i negatif olamaz"));
                }
                values
                    .get(index as usize)
                    .cloned()
                    .map(Some)
                    .ok_or_else(|| RuntimeError::at(expr.span, "Dizi index'i aralik disinda"))
            }
            TypedExprKind::Call { target, args } => self.eval_call(expr.span, target, args),
            TypedExprKind::Unary { op: _, expr: inner } => {
                let value = self.eval_value(inner)?;
                match value {
                    Value::Number(value) => Ok(Some(Value::Number(-value))),
                    Value::Bool(_) | Value::String(_) | Value::Array(_) => Err(RuntimeError::at(
                        expr.span,
                        "Unary eksi icin sayi bekleniyordu",
                    )),
                }
            }
            TypedExprKind::Binary { left, op, right } => {
                let left = self.eval_value(left)?;
                let right = self.eval_value(right)?;
                Ok(Some(self.eval_binary(expr.span, *op, left, right)?))
            }
        }
    }

    fn eval_call(
        &mut self,
        span: SourceSpan,
        target: &CallTarget,
        args: &[TypedExpr],
    ) -> Result<Option<Value>, RuntimeError> {
        let mut values = Vec::with_capacity(args.len());
        for arg in args {
            values.push(self.eval_value(arg)?);
        }

        match target {
            CallTarget::Function { name, .. } => self.call_function(name, values),
            CallTarget::Builtin(BuiltinFunction::Yazdir) => {
                let value = values
                    .into_iter()
                    .next()
                    .ok_or_else(|| RuntimeError::at(span, "`yazdır` bir arguman bekler"))?;
                self.output.push(value.render());
                Ok(None)
            }
            CallTarget::Builtin(BuiltinFunction::Uzunluk) => {
                let value = values
                    .into_iter()
                    .next()
                    .ok_or_else(|| RuntimeError::at(span, "`uzunluk` bir arguman bekler"))?;
                match value {
                    Value::String(value) => Ok(Some(Value::Number(value.len() as i64))),
                    Value::Array(values) => Ok(Some(Value::Number(values.len() as i64))),
                    _ => Err(RuntimeError::at(
                        span,
                        "`uzunluk` argumani calisma zamaninda metin veya dizi olmali",
                    )),
                }
            }
        }
    }

    fn eval_binary(
        &self,
        span: SourceSpan,
        op: BinaryOp,
        left: Value,
        right: Value,
    ) -> Result<Value, RuntimeError> {
        match (op, left, right) {
            (BinaryOp::Add, Value::Number(left), Value::Number(right)) => {
                Ok(Value::Number(left + right))
            }
            (BinaryOp::Add, Value::String(left), Value::String(right)) => {
                Ok(Value::String(format!("{left}{right}")))
            }
            (BinaryOp::Subtract, Value::Number(left), Value::Number(right)) => {
                Ok(Value::Number(left - right))
            }
            (BinaryOp::Multiply, Value::Number(left), Value::Number(right)) => {
                Ok(Value::Number(left * right))
            }
            (BinaryOp::Divide, Value::Number(_), Value::Number(0)) => {
                Err(RuntimeError::at(span, "Sifira bolme hatasi"))
            }
            (BinaryOp::Divide, Value::Number(left), Value::Number(right)) => {
                Ok(Value::Number(left / right))
            }
            (BinaryOp::Less, Value::Number(left), Value::Number(right)) => {
                Ok(Value::Bool(left < right))
            }
            (BinaryOp::Greater, Value::Number(left), Value::Number(right)) => {
                Ok(Value::Bool(left > right))
            }
            (BinaryOp::LessEqual, Value::Number(left), Value::Number(right)) => {
                Ok(Value::Bool(left <= right))
            }
            (BinaryOp::GreaterEqual, Value::Number(left), Value::Number(right)) => {
                Ok(Value::Bool(left >= right))
            }
            (BinaryOp::Equal, left, right) => Ok(Value::Bool(left == right)),
            (BinaryOp::NotEqual, left, right) => Ok(Value::Bool(left != right)),
            _ => Err(RuntimeError::at(span, "Gecersiz ikili islem")),
        }
    }

    fn eval_value(&mut self, expr: &TypedExpr) -> Result<Value, RuntimeError> {
        self.eval_expr(expr)?
            .ok_or_else(|| RuntimeError::at(expr.span, "Bu ifade calisma zamaninda deger uretmedi"))
    }

    fn eval_bool(&mut self, expr: &TypedExpr) -> Result<bool, RuntimeError> {
        match self.eval_value(expr)? {
            Value::Bool(value) => Ok(value),
            Value::Number(_) | Value::String(_) | Value::Array(_) => {
                Err(RuntimeError::at(expr.span, "Mantik degeri bekleniyordu"))
            }
        }
    }

    fn declare(&mut self, name: String, value: Value) {
        self.scopes
            .last_mut()
            .expect("interpreter must have an active scope")
            .insert(name, value);
    }

    fn assign(&mut self, name: &str, value: Value) -> Option<()> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return Some(());
            }
        }

        None
    }

    fn lookup(&self, name: &str) -> Option<Value> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::Interpreter;
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::sema::Analyzer;

    fn run(source: &str) -> Result<String, String> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().map_err(|error| error.to_string())?;
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().map_err(|error| error.to_string())?;
        let typed = Analyzer::analyze(&program).map_err(|error| error.to_string())?;
        Interpreter::run(&typed).map_err(|error| error.to_string())
    }

    #[test]
    fn runs_function_calls_and_prints_output() {
        let source = r#"
Topla(a: sayı, b: sayı) -> sayı {
    dön a + b;
}

Ana() {
    sonuc: sayı = Topla(10, 20);
    yazdır(sonuc);
}
"#;

        assert_eq!(run(source).expect("program should run"), "30");
    }

    #[test]
    fn runs_counter_loop_with_continue_and_break() {
        let source = r#"
Ana() {
    döngü (i: sayı = 0; i < 5; i = i + 1) {
        eğer (i == 1) {
            devam;
        }
        eğer (i == 4) {
            kır;
        }
        yazdır(i);
    }
}
"#;

        assert_eq!(run(source).expect("program should run"), "0\n2\n3");
    }

    #[test]
    fn reports_division_by_zero() {
        let source = r#"
Ana() {
    yazdır(10 / 0);
}
"#;

        let error = run(source).expect_err("program should fail");
        assert!(error.contains("Sifira bolme"));
    }

    #[test]
    fn runs_if_else_and_prints_bool_values() {
        let source = r#"
BuyukMu(x: sayı) -> mantık {
    dön x > 10;
}

Ana() {
    eğer (BuyukMu(12)) {
        yazdır(doğru);
    } değilse {
        yazdır(yanlış);
    }

    yazdır(BuyukMu(5));
}
"#;

        assert_eq!(run(source).expect("program should run"), "doğru\nyanlış");
    }

    #[test]
    fn runs_conditional_loop() {
        let source = r#"
Ana() {
    x: sayı = 0;

    döngü (x < 3) {
        yazdır(x);
        x = x + 1;
    }
}
"#;

        assert_eq!(run(source).expect("program should run"), "0\n1\n2");
    }

    #[test]
    fn runs_infinite_loop_until_break() {
        let source = r#"
Ana() {
    x: sayı = 0;

    döngü {
        eğer (x == 3) {
            kır;
        }

        yazdır(x);
        x = x + 1;
    }
}
"#;

        assert_eq!(run(source).expect("program should run"), "0\n1\n2");
    }

    #[test]
    fn supports_inner_scope_shadowing() {
        let source = r#"
Ana() {
    x: sayı = 1;

    eğer (doğru) {
        x: sayı = 2;
        yazdır(x);
    }

    yazdır(x);
}
"#;

        assert_eq!(run(source).expect("program should run"), "2\n1");
    }

    #[test]
    fn runs_unary_minus() {
        let source = r#"
Ana() {
    x: sayı = -10;
    yazdır(x);
    yazdır(10 + -3);
    yazdır(-x);
}
"#;

        assert_eq!(run(source).expect("program should run"), "-10\n7\n10");
    }

    #[test]
    fn runs_string_values_and_equality() {
        let source = r#"
Ana() {
    mesaj: metin = "Merhaba";
    yazdır(mesaj);
    yazdır(mesaj == "Merhaba");
    yazdır(mesaj != "Baska");
}
"#;

        assert_eq!(
            run(source).expect("program should run"),
            "Merhaba\ndo\u{011f}ru\ndo\u{011f}ru"
        );
    }

    #[test]
    fn runs_string_length_builtin() {
        let source = r#"
Ana() {
    yazdir(uzunluk("Merhaba"));
    yazdir(uzunluk("A" + "B"));
}
"#;

        assert_eq!(run(source).expect("program should run"), "7\n2");
    }

    #[test]
    fn runs_array_literals_index_reads_and_length() {
        let source = r#"
Ana() {
    degerler: dizi = {1, "iki", 3};
    yazdir(uzunluk(degerler));
    yazdir(degerler[0]);
    yazdir(degerler[1]);
}
"#;

        assert_eq!(run(source).expect("program should run"), "3\n1\niki");
    }
}
