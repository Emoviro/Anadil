use crate::ast::SourceSpan;
use crate::error::LexError;
use crate::parser::ParseError;
use crate::sema::SemanticError;

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
