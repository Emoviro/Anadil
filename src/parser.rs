use std::fmt;

use crate::ast::{
    AssignStmt, BinaryOp, Block, Expr, ExprKind, Function, IfStmt, LoopPart, LoopStmt, Param,
    Program, SourceSpan, Stmt, StmtKind, Type, UnaryOp, VarDecl,
};
use crate::token::{Token, TokenKind};

// Parser errors mirror lexer errors with source-aware diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub span: SourceSpan,
}

impl ParseError {
    fn expected(expected: impl Into<String>, found: &Token) -> Self {
        Self {
            message: format!(
                "Beklenen {}, bulunan {}",
                expected.into(),
                found.kind.describe()
            ),
            span: found.span(),
        }
    }

    fn message(message: impl Into<String>, span: SourceSpan) -> Self {
        Self {
            message: message.into(),
            span,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (satır {}, sütun {})",
            self.message, self.span.line, self.span.column
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
        let (name, span) = self.expect_ident()?;
        self.expect_token("`(`", |kind| matches!(kind, TokenKind::LParen))?;
        let params = self.parse_params()?;
        self.expect_token("`)`", |kind| matches!(kind, TokenKind::RParen))?;

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
            span,
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
            let (name, span) = self.expect_ident()?;
            self.expect_token("`:`", |kind| matches!(kind, TokenKind::Colon))?;
            let ty = self.parse_type()?;
            params.push(Param { span, name, ty });

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
            TokenKind::Metin => {
                self.bump();
                Ok(Type::Metin)
            }
            TokenKind::Dizi => {
                self.bump();
                Ok(Type::Dizi)
            }
            _ => Err(ParseError::expected(
                "bir tip (`sayı` veya `mantık`)",
                self.current(),
            )),
        }
    }

    fn parse_block(&mut self) -> Result<Block, ParseError> {
        let lbrace = self.expect_token("`{`", |kind| matches!(kind, TokenKind::LBrace))?;
        let mut statements = Vec::new();

        while !self.check(|kind| matches!(kind, TokenKind::RBrace | TokenKind::Eof)) {
            statements.push(self.parse_statement()?);
        }

        self.expect_token("`}`", |kind| matches!(kind, TokenKind::RBrace))?;

        Ok(Block {
            span: lbrace.span(),
            statements,
        })
    }

    fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        if self.check(|kind| matches!(kind, TokenKind::Eger)) {
            return self.parse_if_statement();
        }

        if self.check(|kind| matches!(kind, TokenKind::Dongu)) {
            return self.parse_loop_statement();
        }

        if self.check(|kind| matches!(kind, TokenKind::Kir)) {
            return self.parse_break_statement();
        }

        if self.check(|kind| matches!(kind, TokenKind::Devam)) {
            return self.parse_continue_statement();
        }

        if self.check(|kind| matches!(kind, TokenKind::Don)) {
            return self.parse_return_statement();
        }

        if self.is_var_decl_start() {
            let decl = self.parse_var_decl()?;
            self.expect_token("`;`", |kind| matches!(kind, TokenKind::Semicolon))?;
            return Ok(Stmt::new(decl.span, StmtKind::VarDecl(decl)));
        }

        if self.is_assignment_start() {
            let assign = self.parse_assignment()?;
            self.expect_token("`;`", |kind| matches!(kind, TokenKind::Semicolon))?;
            return Ok(Stmt::new(assign.span, StmtKind::Assign(assign)));
        }

        let expr = self.parse_expression()?;
        self.expect_token("`;`", |kind| matches!(kind, TokenKind::Semicolon))?;
        Ok(Stmt::new(expr.span, StmtKind::Expr(expr)))
    }

    fn parse_if_statement(&mut self) -> Result<Stmt, ParseError> {
        let if_token = self.expect_token("`eğer`", |kind| matches!(kind, TokenKind::Eger))?;
        self.expect_token("`(`", |kind| matches!(kind, TokenKind::LParen))?;
        let condition = self.parse_expression()?;
        self.expect_token("`)`", |kind| matches!(kind, TokenKind::RParen))?;

        let then_branch = self.parse_block()?;
        let else_branch = if self
            .consume_if(|kind| matches!(kind, TokenKind::Degilse))
            .is_some()
        {
            Some(self.parse_block()?)
        } else {
            None
        };

        let span = if_token.span();
        Ok(Stmt::new(
            span,
            StmtKind::If(IfStmt {
                span,
                condition,
                then_branch,
                else_branch,
            }),
        ))
    }

    fn parse_loop_statement(&mut self) -> Result<Stmt, ParseError> {
        let loop_token = self.expect_token("`döngü`", |kind| matches!(kind, TokenKind::Dongu))?;
        let span = loop_token.span();

        if self.check(|kind| matches!(kind, TokenKind::LBrace)) {
            let body = self.parse_block()?;
            return Ok(Stmt::new(
                span,
                StmtKind::Loop(LoopStmt {
                    span,
                    init: None,
                    condition: None,
                    step: None,
                    body,
                }),
            ));
        }

        self.expect_token("`(`", |kind| matches!(kind, TokenKind::LParen))?;

        if self.has_top_level_semicolon_before_rparen() {
            let init = if self.check(|kind| matches!(kind, TokenKind::Semicolon)) {
                None
            } else {
                Some(self.parse_loop_part()?)
            };
            self.expect_token("`;`", |kind| matches!(kind, TokenKind::Semicolon))?;

            let condition = if self.check(|kind| matches!(kind, TokenKind::Semicolon)) {
                None
            } else {
                Some(self.parse_expression()?)
            };
            self.expect_token("`;`", |kind| matches!(kind, TokenKind::Semicolon))?;

            let step = if self.check(|kind| matches!(kind, TokenKind::RParen)) {
                None
            } else {
                Some(self.parse_loop_part()?)
            };
            self.expect_token("`)`", |kind| matches!(kind, TokenKind::RParen))?;

            let body = self.parse_block()?;

            return Ok(Stmt::new(
                span,
                StmtKind::Loop(LoopStmt {
                    span,
                    init,
                    condition,
                    step,
                    body,
                }),
            ));
        }

        if self.check(|kind| matches!(kind, TokenKind::RParen)) {
            return Err(ParseError::message(
                "Koşullu döngü içinde bir ifade bekleniyordu",
                self.current_span(),
            ));
        }

        let condition = Some(self.parse_expression()?);
        self.expect_token("`)`", |kind| matches!(kind, TokenKind::RParen))?;
        let body = self.parse_block()?;

        Ok(Stmt::new(
            span,
            StmtKind::Loop(LoopStmt {
                span,
                init: None,
                condition,
                step: None,
                body,
            }),
        ))
    }

    fn parse_break_statement(&mut self) -> Result<Stmt, ParseError> {
        let token = self.expect_token("`kır`", |kind| matches!(kind, TokenKind::Kir))?;
        self.expect_token("`;`", |kind| matches!(kind, TokenKind::Semicolon))?;
        Ok(Stmt::new(token.span(), StmtKind::Break))
    }

    fn parse_continue_statement(&mut self) -> Result<Stmt, ParseError> {
        let token = self.expect_token("`devam`", |kind| matches!(kind, TokenKind::Devam))?;
        self.expect_token("`;`", |kind| matches!(kind, TokenKind::Semicolon))?;
        Ok(Stmt::new(token.span(), StmtKind::Continue))
    }

    fn parse_return_statement(&mut self) -> Result<Stmt, ParseError> {
        let token = self.expect_token("`dön`", |kind| matches!(kind, TokenKind::Don))?;
        let span = token.span();

        if self.check(|kind| matches!(kind, TokenKind::Semicolon)) {
            self.bump();
            return Ok(Stmt::new(span, StmtKind::Return(None)));
        }

        let value = self.parse_expression()?;
        self.expect_token("`;`", |kind| matches!(kind, TokenKind::Semicolon))?;
        Ok(Stmt::new(span, StmtKind::Return(Some(value))))
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
        let (name, span) = self.expect_ident()?;
        self.expect_token("`:`", |kind| matches!(kind, TokenKind::Colon))?;
        let ty = self.parse_type()?;
        self.expect_token("`=`", |kind| matches!(kind, TokenKind::Assign))?;
        let value = self.parse_expression()?;

        Ok(VarDecl {
            span,
            name,
            ty,
            value,
        })
    }

    fn parse_assignment(&mut self) -> Result<AssignStmt, ParseError> {
        let (target, span) = self.expect_ident()?;
        self.expect_token("`=`", |kind| matches!(kind, TokenKind::Assign))?;
        let value = self.parse_expression()?;

        Ok(AssignStmt {
            span,
            target,
            value,
        })
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

            let span = expr.span;
            let right = self.parse_comparison()?;
            expr = Expr::new(
                span,
                ExprKind::Binary {
                    left: Box::new(expr),
                    op,
                    right: Box::new(right),
                },
            );
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

            let span = expr.span;
            let right = self.parse_term()?;
            expr = Expr::new(
                span,
                ExprKind::Binary {
                    left: Box::new(expr),
                    op,
                    right: Box::new(right),
                },
            );
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

            let span = expr.span;
            let right = self.parse_factor()?;
            expr = Expr::new(
                span,
                ExprKind::Binary {
                    left: Box::new(expr),
                    op,
                    right: Box::new(right),
                },
            );
        }

        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_unary()?;

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

            let span = expr.span;
            let right = self.parse_unary()?;
            expr = Expr::new(
                span,
                ExprKind::Binary {
                    left: Box::new(expr),
                    op,
                    right: Box::new(right),
                },
            );
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        if self
            .consume_if(|kind| matches!(kind, TokenKind::Minus))
            .is_some()
        {
            let span = self.previous_span();
            let expr = self.parse_unary()?;
            return Ok(Expr::new(
                span,
                ExprKind::Unary {
                    op: UnaryOp::Negate,
                    expr: Box::new(expr),
                },
            ));
        }

        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary()?;

        while self
            .consume_if(|kind| matches!(kind, TokenKind::LBracket))
            .is_some()
        {
            let span = expr.span;
            let index = self.parse_expression()?;
            self.expect_token("`]`", |kind| matches!(kind, TokenKind::RBracket))?;
            expr = Expr::new(
                span,
                ExprKind::Index {
                    target: Box::new(expr),
                    index: Box::new(index),
                },
            );
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.current().kind.clone() {
            TokenKind::Number(value) => {
                let token = self.bump();
                Ok(Expr::new(token.span(), ExprKind::Number(value)))
            }
            TokenKind::String(value) => {
                let token = self.bump();
                Ok(Expr::new(token.span(), ExprKind::String(value)))
            }
            TokenKind::Dogru => {
                let token = self.bump();
                Ok(Expr::new(token.span(), ExprKind::Bool(true)))
            }
            TokenKind::Yanlis => {
                let token = self.bump();
                Ok(Expr::new(token.span(), ExprKind::Bool(false)))
            }
            TokenKind::Ident(name) => {
                let token = self.bump();
                let span = token.span();

                if self
                    .consume_if(|kind| matches!(kind, TokenKind::LParen))
                    .is_some()
                {
                    let args = self.parse_arguments()?;
                    self.expect_token("`)`", |kind| matches!(kind, TokenKind::RParen))?;
                    Ok(Expr::new(span, ExprKind::Call { callee: name, args }))
                } else {
                    Ok(Expr::new(span, ExprKind::Variable(name)))
                }
            }
            TokenKind::LParen => {
                self.bump();
                let expr = self.parse_expression()?;
                self.expect_token("`)`", |kind| matches!(kind, TokenKind::RParen))?;
                Ok(expr)
            }
            TokenKind::LBrace => self.parse_array_literal(),
            _ => Err(ParseError::expected("bir ifade", self.current())),
        }
    }

    fn parse_array_literal(&mut self) -> Result<Expr, ParseError> {
        let lbrace = self.expect_token("`{`", |kind| matches!(kind, TokenKind::LBrace))?;
        let mut elements = Vec::new();

        if self.check(|kind| matches!(kind, TokenKind::RBrace)) {
            self.bump();
            return Ok(Expr::new(lbrace.span(), ExprKind::Array(elements)));
        }

        loop {
            elements.push(self.parse_expression()?);

            if self
                .consume_if(|kind| matches!(kind, TokenKind::Comma))
                .is_none()
            {
                break;
            }
        }

        self.expect_token("`}`", |kind| matches!(kind, TokenKind::RBrace))?;
        Ok(Expr::new(lbrace.span(), ExprKind::Array(elements)))
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

    fn expect_ident(&mut self) -> Result<(String, SourceSpan), ParseError> {
        match self.current().kind.clone() {
            TokenKind::Ident(name) => {
                let token = self.bump();
                Ok((name, token.span()))
            }
            _ => Err(ParseError::expected("bir tanımlayıcı", self.current())),
        }
    }

    fn current(&self) -> &Token {
        self.tokens
            .get(self.pos)
            .unwrap_or_else(|| self.tokens.last().expect("token stream is empty"))
    }

    fn current_span(&self) -> SourceSpan {
        self.current().span()
    }

    fn previous_span(&self) -> SourceSpan {
        self.tokens
            .get(self.pos.saturating_sub(1))
            .map(Token::span)
            .unwrap_or_else(|| self.current_span())
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

    fn expect_token<F>(&mut self, expected: &str, predicate: F) -> Result<Token, ParseError>
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
    use crate::ast::{BinaryOp, ExprKind, LoopPart, StmtKind, Type};
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

        match &function.body.statements[0].kind {
            StmtKind::Return(Some(expr)) => match &expr.kind {
                ExprKind::Binary { op, .. } => assert_eq!(*op, BinaryOp::Add),
                other => panic!("beklenmeyen dönüş ifadesi: {other:?}"),
            },
            other => panic!("beklenmeyen statement: {other:?}"),
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
    yazdır(sonuc);
}
"#,
        );

        let function = &program.functions[0];
        assert_eq!(function.name.as_str(), "Ana");
        assert_eq!(function.body.statements.len(), 4);

        match &function.body.statements[2].kind {
            StmtKind::VarDecl(decl) => match &decl.value.kind {
                ExprKind::Call { callee, args } => {
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
        yazdır(i);
    }
}
"#,
        );

        let function = &program.functions[0];

        match &function.body.statements[0].kind {
            StmtKind::Loop(loop_stmt) => {
                assert!(matches!(&loop_stmt.init, Some(LoopPart::VarDecl(_))));
                assert!(matches!(&loop_stmt.step, Some(LoopPart::Assign(_))));
                assert_eq!(loop_stmt.body.statements.len(), 2);
            }
            other => panic!("beklenmeyen statement: {other:?}"),
        }
    }

    #[test]
    fn parses_unary_minus() {
        let program = parse(
            r#"
Ana() {
    x: sayı = -10;
    yazdır(10 + -x);
}
"#,
        );

        let function = &program.functions[0];

        match &function.body.statements[0].kind {
            StmtKind::VarDecl(decl) => {
                assert!(matches!(&decl.value.kind, ExprKind::Unary { .. }));
            }
            other => panic!("beklenmeyen statement: {other:?}"),
        }
    }
}
