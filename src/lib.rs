pub mod ast;
pub mod diagnostics;
pub mod error;
pub mod interpreter;
pub mod ir;
pub mod lexer;
pub mod native;
pub mod optimizer;
pub mod parser;
pub mod sema;
pub mod token;
pub mod typed;

use diagnostics::{format_lex_error, format_parse_error, format_semantic_error, Diagnostic};
use interpreter::Interpreter;
use ir::{format_ir, lower_typed_program};
use lexer::Lexer;
use optimizer::optimize_typed_program;
use parser::Parser;
use sema::Analyzer;

pub fn parse_source(source: &str) -> Result<ast::Program, String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer
        .tokenize()
        .map_err(|error| format_lex_error(source, &error))?;

    let mut parser = Parser::new(tokens);
    parser
        .parse_program()
        .map_err(|error| format_parse_error(source, &error))
}

pub fn compile_source(source: &str) -> Result<typed::TypedProgram, String> {
    let program = parse_source(source)?;
    Analyzer::analyze(&program)
        .map(optimize_typed_program)
        .map_err(|error| format_semantic_error(source, &error))
}

pub fn check_source(source: &str) -> Result<(), Diagnostic> {
    compile_source_diagnostic(source).map(|_| ())
}

pub fn run_source_diagnostic(source: &str) -> Result<String, Diagnostic> {
    let program = compile_source_diagnostic(source)?;
    Interpreter::run(&program).map_err(|error| Diagnostic::from_runtime_error(&error))
}

fn compile_source_diagnostic(source: &str) -> Result<typed::TypedProgram, Diagnostic> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer
        .tokenize()
        .map_err(|error| Diagnostic::from_lex_error(&error))?;

    let mut parser = Parser::new(tokens);
    let program = parser
        .parse_program()
        .map_err(|error| Diagnostic::from_parse_error(&error))?;

    Analyzer::analyze(&program)
        .map(optimize_typed_program)
        .map_err(|error| Diagnostic::from_semantic_error(&error))
}

pub fn run_source(source: &str) -> Result<String, String> {
    let program = compile_source(source)?;
    Interpreter::run(&program).map_err(|error| format!("Calisma zamani hatasi: {error}"))
}

pub fn emit_native_asm_source(source: &str) -> Result<String, String> {
    let program = compile_source(source)?;
    native::emit_windows_x64_asm(&program)
}

pub fn emit_ir_source(source: &str) -> Result<String, String> {
    let program = compile_source(source)?;
    Ok(format_ir(&lower_typed_program(&program)))
}

#[cfg(test)]
mod tests {
    use super::{compile_source, parse_source, run_source};

    #[test]
    fn parses_source_through_library_api() {
        let source = r#"
Ana() {
    yazdır(10);
}
"#;

        let program = parse_source(source).expect("source should parse");
        assert_eq!(program.functions.len(), 1);
    }

    #[test]
    fn compiles_source_through_library_api() {
        let source = r#"
Ana() {
    yazdır(10);
}
"#;

        let program = compile_source(source).expect("source should compile");
        assert_eq!(program.functions[0].name, "Ana");
    }

    #[test]
    fn runs_source_through_library_api() {
        let source = r#"
Ana() {
    yazdır(10 + 20);
}
"#;

        assert_eq!(run_source(source).expect("source should run"), "30");
    }
}
