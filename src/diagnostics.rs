use crate::ast::SourceSpan;
use crate::error::LexError;
use crate::interpreter::RuntimeError;
use crate::parser::ParseError;
use crate::sema::SemanticError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub stage: DiagnosticStage,
    pub message: String,
    pub span: Option<SourceSpan>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticStage {
    Io,
    Lexer,
    Native,
    Parser,
    Runtime,
    Semantic,
}

impl Diagnostic {
    pub fn io(message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            stage: DiagnosticStage::Io,
            message: message.into(),
            span: None,
        }
    }

    pub fn native(message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            stage: DiagnosticStage::Native,
            message: message.into(),
            span: None,
        }
    }

    pub fn from_lex_error(error: &LexError) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            stage: DiagnosticStage::Lexer,
            message: error.message.clone(),
            span: Some(SourceSpan::new(error.line, error.column)),
        }
    }

    pub fn from_parse_error(error: &ParseError) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            stage: DiagnosticStage::Parser,
            message: error.message.clone(),
            span: Some(error.span),
        }
    }

    pub fn from_semantic_error(error: &SemanticError) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            stage: DiagnosticStage::Semantic,
            message: error.message.clone(),
            span: error.span,
        }
    }

    pub fn from_runtime_error(error: &RuntimeError) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            stage: DiagnosticStage::Runtime,
            message: error.message.clone(),
            span: error.span,
        }
    }
}

impl DiagnosticSeverity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
        }
    }
}

impl DiagnosticStage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Io => "io",
            Self::Lexer => "lexer",
            Self::Native => "native",
            Self::Parser => "parser",
            Self::Runtime => "runtime",
            Self::Semantic => "semantic",
        }
    }
}

pub fn format_lex_error(source: &str, error: &LexError) -> String {
    format_diagnostic(
        source,
        "Lexer hatası",
        &error.message,
        Some(SourceSpan::new(error.line, error.column)),
    )
}

pub fn format_parse_error(source: &str, error: &ParseError) -> String {
    format_diagnostic(source, "Parser hatası", &error.message, Some(error.span))
}

pub fn format_semantic_error(source: &str, error: &SemanticError) -> String {
    format_diagnostic(source, "Semantic hata", &error.message, error.span)
}

fn format_diagnostic(source: &str, title: &str, message: &str, span: Option<SourceSpan>) -> String {
    let Some(span) = span else {
        return format!("{title}: {message}");
    };

    let Some(source_line) = source.lines().nth(span.line.saturating_sub(1)) else {
        return format!("{title}: {message} ({span})");
    };

    let line_number = span.line.to_string();
    let gutter_padding = " ".repeat(line_number.len());
    let caret_padding = build_caret_padding(source_line, span.column);

    format!(
        "{title}: {message} ({span})\n{line_number} | {source_line}\n{gutter_padding} | {caret_padding}^"
    )
}

fn build_caret_padding(source_line: &str, column: usize) -> String {
    source_line
        .chars()
        .take(column.saturating_sub(1))
        .map(|ch| if ch == '\t' { '\t' } else { ' ' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::format_semantic_error;
    use crate::ast::SourceSpan;
    use crate::sema::SemanticError;

    #[test]
    fn formats_caret_diagnostic() {
        let source = "Ana() {\n    kır;\n}\n";
        let error = SemanticError {
            message: "`kır` ifadesi yalnızca döngü içinde kullanılabilir".to_string(),
            span: Some(SourceSpan::new(2, 5)),
        };

        let formatted = format_semantic_error(source, &error);

        assert!(formatted.contains("2 |     kır;"));
        assert!(formatted.contains("  |     ^"));
    }
}
