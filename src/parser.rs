use std::fmt;

use crate::ast::{
    AssignStmt, BinaryOp, Block, Expr, Function, IfStmt, LoopPart, LoopStmt, Param, Program, Stmt,
    Type, VarDecl,
};
use crate::token::{Token, TokenKind};

// Parser errors mirror lexer errors with source-aware diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl ParseError {
    fn expected(expected: impl Into<String>, found: &Token) -> Self {
        Self {
            message: format!(
                "Beklenen {}, bulunan {}",
                expected.into(),
                found.kind.describe()
            ),
            line: found.line,
            column: found.column,
        }
    }

    fn message(message: impl Into<String>, found: &Token) -> Self {
        Self {
            message: message.into(),
            line: found.line,
            column: found.column,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (satır {}, sütun {})",
            self.message, self.line, self.column
        )
    }
}

impl std::error::Error for ParseError {}

// A small recursive descent parser is enough for the V1 grammar.
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut functions = Vec::new();

        while !self.is_at_end() {
            functions.push(self.parse_function()?);
        }

        Ok(Program { functions })
    }

    fn parse_function(&mut self) -> Result<Function, ParseError> {
        let name = self.expect_ident()?;
        self.expect("`(`", |kind| matches!(kind, TokenKind::LParen))?;
        let params = self.parse_params()?;
        self.expect("`)`", |kind| matches!(kind, TokenKind::RParen))?;

        let return_type = if self
            .consume_if(|kind| matches!(kind, TokenKind::Arrow))
            .is_some()
        {
            Some(self.parse_type()?)
        } else {
            None
        };

        let body = self.parse_block()?;

        Ok(Function {
            name,
            params,
            return_type,
            body,
        })
    }

    fn parse_params(&mut self) -> Result<Vec<Param>, ParseError> {
        let mut params = Vec::new();

        if self.check(|kind| matches!(kind, TokenKind::RParen)) {
            return Ok(params);
        }

        loop {
            let name = self.expect_ident()?;
            self.expect("`:`", |kind| matches!(kind, TokenKind::Colon))?;
            let ty = self.parse_type()?;
            params.push(Param { name, ty });

            if self
                .consume_if(|kind| matches!(kind, TokenKind::Comma))
                .is_none()
            {
                break;
            }
        }

        Ok(params)
    }

    fn parse_type(&mut self) -> Result<Type, ParseError> {
        match &self.current().kind {
            TokenKind::Sayi => {
                self.bump();
                Ok(Type::Sayi)
            }
            TokenKind::Mantik => {
                self.bump();
                Ok(Type::Mantik)
            }
            _ => Err(ParseError::expected(
                "bir tip (`sayı` veya `mantık`)",
                self.current(),
            )),
        }
    }

    fn parse_block(&mut self) -> Result<Block, ParseError> {
        self.expect("`{`", |kind| matches!(kind, TokenKind::LBrace))?;

        let mut statements = Vec::new();
        while !self.check(|kind| matches!(kind, TokenKind::RBrace | TokenKind::Eof)) {
            statements.push(self.parse_statement()?);
        }

        self.expect("`}`", |kind| matches!(kind, TokenKind::RBrace))?;
        Ok(Block { statements })
    }

    fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        if self
            .consume_if(|kind| matches!(kind, TokenKind::Eger))
            .is_some()
        {
            return self.parse_if_statement();
        }

        if self
            .consume_if(|kind| matches!(kind, TokenKind::Dongu))
            .is_some()
        {
            return self.parse_loop_statement();
        }

        if self
            .consume_if(|kind| matches!(kind, TokenKind::Kir))
            .is_some()
        {
            self.expect("`;`", |kind| matches!(kind, TokenKind::Semicolon))?;
            return Ok(Stmt::Break);
        }

        if self
            .consume_if(|kind| matches!(kind, TokenKind::Devam))
            .is_some()
        {
            self.expect("`;`", |kind| matches!(kind, TokenKind::Semicolon))?;
            return Ok(Stmt::Continue);
        }

        if self
            .consume_if(|kind| matches!(kind, TokenKind::Don))
            .is_some()
        {
            return self.parse_return_statement();
        }

        if self.is_var_decl_start() {
            let decl = self.parse_var_decl()?;
            self.expect("`;`", |kind| matches!(kind, TokenKind::Semicolon))?;
            return Ok(Stmt::VarDecl(decl));
        }

        if self.is_assignment_start() {
            let assign = self.parse_assignment()?;
            self.expect("`;`", |kind| matches!(kind, TokenKind::Semicolon))?;
            return Ok(Stmt::Assign(assign));
        }

        let expr = self.parse_expression()?;
        self.expect("`;`", |kind| matches!(kind, TokenKind::Semicolon))?;
        Ok(Stmt::Expr(expr))
    }

    fn parse_if_statement(&mut self) -> Result<Stmt, ParseError> {
        self.expect("`(`", |kind| matches!(kind, TokenKind::LParen))?;
        let condition = self.parse_expression()?;
        self.expect("`)`", |kind| matches!(kind, TokenKind::RParen))?;

        let then_branch = self.parse_block()?;
        let else_branch = if self
            .consume_if(|kind| matches!(kind, TokenKind::Degilse))
            .is_some()
        {
            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(Stmt::If(IfStmt {
            condition,
            then_branch,
            else_branch,
        }))
    }

    fn parse_loop_statement(&mut self) -> Result<Stmt, ParseError> {
        if self.check(|kind| matches!(kind, TokenKind::LBrace)) {
            let body = self.parse_block()?;
            return Ok(Stmt::Loop(LoopStmt {
                init: None,
                condition: None,
                step: None,
                body,
            }));
        }

        self.expect("`(`", |kind| matches!(kind, TokenKind::LParen))?;

        if self.has_top_level_semicolon_before_rparen() {
            let init = if self.check(|kind| matches!(kind, TokenKind::Semicolon)) {
                None
            } else {
                Some(self.parse_loop_part()?)
            };
            self.expect("`;`", |kind| matches!(kind, TokenKind::Semicolon))?;

            let condition = if self.check(|kind| matches!(kind, TokenKind::Semicolon)) {
                None
            } else {
                Some(self.parse_expression()?)
            };
            self.expect("`;`", |kind| matches!(kind, TokenKind::Semicolon))?;

            let step = if self.check(|kind| matches!(kind, TokenKind::RParen)) {
                None
            } else {
                Some(self.parse_loop_part()?)
            };
            self.expect("`)`", |kind| matches!(kind, TokenKind::RParen))?;

            let body = self.parse_block()?;
            return Ok(Stmt::Loop(LoopStmt {
                init,
                condition,
                step,
                body,
            }));
        }

        if self.check(|kind| matches!(kind, TokenKind::RParen)) {
            return Err(ParseError::message(
                "Koşullu döngü içinde bir ifade bekleniyordu",
                self.current(),
            ));
        }

        let condition = Some(self.parse_expression()?);
        self.expect("`)`", |kind| matches!(kind, TokenKind::RParen))?;
        let body = self.parse_block()?;

        Ok(Stmt::Loop(LoopStmt {
            init: None,
            condition,
            step: None,
            body,
        }))
    }

    fn parse_return_statement(&mut self) -> Result<Stmt, ParseError> {
        if self.check(|kind| matches!(kind, TokenKind::Semicolon)) {
            self.bump();
            return Ok(Stmt::Return(None));
        }

        let value = self.parse_expression()?;
        self.expect("`;`", |kind| matches!(kind, TokenKind::Semicolon))?;
        Ok(Stmt::Return(Some(value)))
    }

    fn parse_loop_part(&mut self) -> Result<LoopPart, ParseError> {
        if self.is_var_decl_start() {
            return Ok(LoopPart::VarDecl(self.parse_var_decl()?));
        }

        if self.is_assignment_start() {
            return Ok(LoopPart::Assign(self.parse_assignment()?));
        }

        Ok(LoopPart::Expr(self.parse_expression()?))
    }

    fn parse_var_decl(&mut self) -> Result<VarDecl, ParseError> {
        let name = self.expect_ident()?;
        self.expect("`:`", |kind| matches!(kind, TokenKind::Colon))?;
        let ty = self.parse_type()?;
        self.expect("`=`", |kind| matches!(kind, TokenKind::Assign))?;
        let value = self.parse_expression()?;

        Ok(VarDecl { name, ty, value })
    }

    fn parse_assignment(&mut self) -> Result<AssignStmt, ParseError> {
        let target = self.expect_ident()?;
        self.expect("`=`", |kind| matches!(kind, TokenKind::Assign))?;
        let value = self.parse_expression()?;

        Ok(AssignStmt { target, value })
    }

    fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.parse_equality()
    }

    fn parse_equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_comparison()?;

        loop {
            let op = if self
                .consume_if(|kind| matches!(kind, TokenKind::EqEq))
                .is_some()
            {
                Some(BinaryOp::Equal)
            } else if self
                .consume_if(|kind| matches!(kind, TokenKind::NotEq))
                .is_some()
            {
                Some(BinaryOp::NotEqual)
            } else {
                None
            };

            let Some(op) = op else {
                break;
            };

            let right = self.parse_comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_term()?;

        loop {
            let op = if self
                .consume_if(|kind| matches!(kind, TokenKind::Less))
                .is_some()
            {
                Some(BinaryOp::Less)
            } else if self
                .consume_if(|kind| matches!(kind, TokenKind::Greater))
                .is_some()
            {
                Some(BinaryOp::Greater)
            } else if self
                .consume_if(|kind| matches!(kind, TokenKind::LessEq))
                .is_some()
            {
                Some(BinaryOp::LessEqual)
            } else if self
                .consume_if(|kind| matches!(kind, TokenKind::GreaterEq))
                .is_some()
            {
                Some(BinaryOp::GreaterEqual)
            } else {
                None
            };

            let Some(op) = op else {
                break;
            };

            let right = self.parse_term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_factor()?;

        loop {
            let op = if self
                .consume_if(|kind| matches!(kind, TokenKind::Plus))
                .is_some()
            {
                Some(BinaryOp::Add)
            } else if self
                .consume_if(|kind| matches!(kind, TokenKind::Minus))
                .is_some()
            {
                Some(BinaryOp::Subtract)
            } else {
                None
            };

            let Some(op) = op else {
                break;
            };

            let right = self.parse_factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary()?;

        loop {
            let op = if self
                .consume_if(|kind| matches!(kind, TokenKind::Star))
                .is_some()
            {
                Some(BinaryOp::Multiply)
            } else if self
                .consume_if(|kind| matches!(kind, TokenKind::Slash))
                .is_some()
            {
                Some(BinaryOp::Divide)
            } else {
                None
            };

            let Some(op) = op else {
                break;
            };

            let right = self.parse_primary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.current().kind.clone() {
            TokenKind::Number(value) => {
                self.bump();
                Ok(Expr::Number(value))
            }
            TokenKind::Dogru => {
                self.bump();
                Ok(Expr::Bool(true))
            }
            TokenKind::Yanlis => {
                self.bump();
                Ok(Expr::Bool(false))
            }
            TokenKind::Ident(name) => {
                self.bump();

                if self
                    .consume_if(|kind| matches!(kind, TokenKind::LParen))
                    .is_some()
                {
                    let args = self.parse_arguments()?;
                    self.expect("`)`", |kind| matches!(kind, TokenKind::RParen))?;
                    Ok(Expr::Call { callee: name, args })
                } else {
                    Ok(Expr::Variable(name))
                }
            }
            TokenKind::LParen => {
                self.bump();
                let expr = self.parse_expression()?;
                self.expect("`)`", |kind| matches!(kind, TokenKind::RParen))?;
                Ok(expr)
            }
            _ => Err(ParseError::expected("bir ifade", self.current())),
        }
    }

    fn parse_arguments(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut args = Vec::new();

        if self.check(|kind| matches!(kind, TokenKind::RParen)) {
            return Ok(args);
        }

        loop {
            args.push(self.parse_expression()?);

            if self
                .consume_if(|kind| matches!(kind, TokenKind::Comma))
                .is_none()
            {
                break;
            }
        }

        Ok(args)
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        match self.current().kind.clone() {
            TokenKind::Ident(name) => {
                self.bump();
                Ok(name)
            }
            _ => Err(ParseError::expected("bir tanımlayıcı", self.current())),
        }
    }

    fn current(&self) -> &Token {
        self.tokens
            .get(self.pos)
            .unwrap_or_else(|| self.tokens.last().expect("token stream is empty"))
    }

    fn peek_kind(&self) -> Option<&TokenKind> {
        self.tokens.get(self.pos + 1).map(|token| &token.kind)
    }

    fn is_at_end(&self) -> bool {
        matches!(&self.current().kind, TokenKind::Eof)
    }

    fn bump(&mut self) -> Token {
        let token = self.current().clone();

        if !self.is_at_end() {
            self.pos += 1;
        }

        token
    }

    fn check<F>(&self, predicate: F) -> bool
    where
        F: Fn(&TokenKind) -> bool,
    {
        predicate(&self.current().kind)
    }

    fn consume_if<F>(&mut self, predicate: F) -> Option<Token>
    where
        F: Fn(&TokenKind) -> bool,
    {
        if predicate(&self.current().kind) {
            Some(self.bump())
        } else {
            None
        }
    }

    fn expect<F>(&mut self, expected: &str, predicate: F) -> Result<Token, ParseError>
    where
        F: Fn(&TokenKind) -> bool,
    {
        if predicate(&self.current().kind) {
            Ok(self.bump())
        } else {
            Err(ParseError::expected(expected, self.current()))
        }
    }

    fn is_var_decl_start(&self) -> bool {
        matches!(&self.current().kind, TokenKind::Ident(_))
            && matches!(self.peek_kind(), Some(kind) if matches!(kind, TokenKind::Colon))
    }

    fn is_assignment_start(&self) -> bool {
        matches!(&self.current().kind, TokenKind::Ident(_))
            && matches!(self.peek_kind(), Some(kind) if matches!(kind, TokenKind::Assign))
    }

    fn has_top_level_semicolon_before_rparen(&self) -> bool {
        let mut depth = 0usize;
        let mut index = self.pos;

        while let Some(token) = self.tokens.get(index) {
            match &token.kind {
                TokenKind::LParen => depth += 1,
                TokenKind::RParen => {
                    if depth == 0 {
                        return false;
                    }
                    depth -= 1;
                }
                TokenKind::Semicolon if depth == 0 => return true,
                TokenKind::Eof => return false,
                _ => {}
            }

            index += 1;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::Parser;
    use crate::ast::{BinaryOp, Expr, LoopPart, Stmt, Type};
    use crate::lexer::Lexer;

    fn parse(source: &str) -> crate::ast::Program {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().expect("lexing failed");
        let mut parser = Parser::new(tokens);
        parser.parse_program().expect("parsing failed")
    }

    #[test]
    fn parses_returning_function() {
        let program = parse(
            r#"
Topla(a: sayı, b: sayı) -> sayı {
    dön a + b;
}
"#,
        );

        assert_eq!(program.functions.len(), 1);

        let function = &program.functions[0];
        assert_eq!(function.name.as_str(), "Topla");
        assert_eq!(function.return_type.as_ref(), Some(&Type::Sayi));
        assert_eq!(function.params.len(), 2);

        match &function.body.statements[0] {
            Stmt::Return(Some(Expr::Binary { op, .. })) => assert_eq!(*op, BinaryOp::Add),
            other => panic!("beklenmeyen dönüş ifadesi: {other:?}"),
        }
    }

    #[test]
    fn parses_variable_declarations_and_calls() {
        let program = parse(
            r#"
Ana() {
    x: sayı = 10;
    y: sayı = 20;
    sonuc: sayı = Topla(x, y);
    yazdir(sonuc);
}
"#,
        );

        let function = &program.functions[0];
        assert_eq!(function.name.as_str(), "Ana");
        assert_eq!(function.body.statements.len(), 4);

        match &function.body.statements[2] {
            Stmt::VarDecl(decl) => match &decl.value {
                Expr::Call { callee, args } => {
                    assert_eq!(callee, "Topla");
                    assert_eq!(args.len(), 2);
                }
                other => panic!("beklenmeyen ifade: {other:?}"),
            },
            other => panic!("beklenmeyen statement: {other:?}"),
        }
    }

    #[test]
    fn parses_counter_loop_and_if() {
        let program = parse(
            r#"
Ana() {
    döngü (i: sayı = 0; i < 10; i = i + 1) {
        eğer (i == 5) {
            devam;
        }
        yazdir(i);
    }
}
"#,
        );

        let function = &program.functions[0];

        match &function.body.statements[0] {
            Stmt::Loop(loop_stmt) => {
                assert!(matches!(&loop_stmt.init, Some(LoopPart::VarDecl(_))));
                assert!(matches!(&loop_stmt.step, Some(LoopPart::Assign(_))));
                assert_eq!(loop_stmt.body.statements.len(), 2);
            }
            other => panic!("beklenmeyen statement: {other:?}"),
        }
    }
}
