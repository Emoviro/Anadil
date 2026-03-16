mod ast;
mod error;
mod lexer;
mod parser;
mod token;

use lexer::Lexer;
use parser::Parser;

fn main() {
    let examples = [
        (
            "Örnek 1",
            r#"
Topla(a: sayı, b: sayı) -> sayı {
    dön a + b;
}
"#,
        ),
        (
            "Örnek 2",
            r#"
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

        match parse_source(source) {
            Ok(program) => println!("{program:#?}"),
            Err(message) => eprintln!("{message}"),
        }

        println!();
    }
}

// This keeps the frontend pipeline visible while the project grows.
fn parse_source(source: &str) -> Result<ast::Program, String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer
        .tokenize()
        .map_err(|error| format!("Lexer hatası: {error}"))?;

    let mut parser = Parser::new(tokens);
    parser
        .parse_program()
        .map_err(|error| format!("Parser hatası: {error}"))
}
