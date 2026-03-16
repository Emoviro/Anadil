mod ast;
mod diagnostics;
mod error;
mod lexer;
mod parser;
mod sema;
mod token;
mod typed;

use diagnostics::{format_lex_error, format_parse_error, format_semantic_error};
use lexer::Lexer;
use parser::Parser;
use sema::Analyzer;

fn main() {
    let examples = [
        (
            "Örnek 1",
            r#"
Topla(a: sayı, b: sayı) -> sayı {
    dön a + b;
}

Ana() {
    sonuç: sayı = Topla(1, 2);
    yazdir(sonuç);
}
"#,
        ),
        (
            "Örnek 2",
            r#"
Topla(a: sayı, b: sayı) -> sayı {
    dön a + b;
}

Ana() {
    x: sayı = 10;
    y: sayı = 20;
    sonuc: sayı = Topla(x, y);
    yazdir(sonuc);
}
"#,
        ),
        (
            "Örnek 3",
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
        ),
    ];

    for (label, source) in examples {
        println!("== {label} ==");

        match compile_frontend(source) {
            Ok(program) => {
                println!("Semantic analiz başarılı.");
                println!("{program:#?}");
            }
            Err(message) => eprintln!("{message}"),
        }

        println!();
    }
}

// This keeps the frontend pipeline visible while the project grows.
fn compile_frontend(source: &str) -> Result<typed::TypedProgram, String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer
        .tokenize()
        .map_err(|error| format_lex_error(source, &error))?;

    let mut parser = Parser::new(tokens);
    let program = parser
        .parse_program()
        .map_err(|error| format_parse_error(source, &error))?;

    Analyzer::analyze(&program).map_err(|error| format_semantic_error(source, &error))
}
