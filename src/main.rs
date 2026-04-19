use std::{
    env, fs,
    io::{self, Write},
    process,
};

use anadil::{compile_source, parse_source, run_source};

#[derive(Debug)]
enum Command {
    Run,
    Check,
    Ast,
    Typed,
    Help,
    Version,
    Examples,
    Repl,
}

#[derive(Debug)]
enum ParsedArgs<'a> {
    WithFile { command: Command, path: &'a str },
    Standalone(Command),
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let parsed = match parse_args(&args) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("{message}");
            eprintln!("{}", usage(&args[0]));
            process::exit(2);
        }
    };

    if let ParsedArgs::Standalone(command) = parsed {
        run_standalone_command(command, &args[0]);
        return;
    }

    let ParsedArgs::WithFile { command, path } = parsed else {
        unreachable!();
    };

    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("Dosya okunamadi `{path}`: {error}");
            process::exit(1);
        }
    };

    if let Err(message) = run_command(command, &source) {
        eprintln!("{message}");
        process::exit(1);
    }
}

fn parse_args(args: &[String]) -> Result<ParsedArgs<'_>, String> {
    match args {
        [_program] => Ok(ParsedArgs::Standalone(Command::Help)),
        [_program, arg] => match parse_command(arg) {
            Ok(command) if command.requires_file() => {
                Err(format!("`{}` komutu dosya yolu bekler.", command.name()))
            }
            Ok(command) => Ok(ParsedArgs::Standalone(command)),
            Err(_) => Ok(ParsedArgs::WithFile {
                command: Command::Run,
                path: arg.as_str(),
            }),
        },
        [_program, command, path] => {
            let command = parse_command(command)?;
            if command.requires_file() {
                Ok(ParsedArgs::WithFile {
                    command,
                    path: path.as_str(),
                })
            } else {
                Err(format!("`{}` komutu dosya yolu almaz.", command.name()))
            }
        }
        _ => Err("Gecersiz arguman sayisi.".to_string()),
    }
}

fn parse_command(command: &str) -> Result<Command, String> {
    match command {
        "calistir" | "çalıştır" | "run" => Ok(Command::Run),
        "kontrol" | "check" => Ok(Command::Check),
        "ast" => Ok(Command::Ast),
        "typed" => Ok(Command::Typed),
        "yardim" | "yardım" | "help" | "-h" | "--help" => Ok(Command::Help),
        "surum" | "sürüm" | "version" | "-V" | "--version" => Ok(Command::Version),
        "ornekler" | "örnekler" | "examples" => Ok(Command::Examples),
        "repl" => Ok(Command::Repl),
        _ => Err(format!("Bilinmeyen komut: `{command}`")),
    }
}

impl Command {
    fn requires_file(&self) -> bool {
        matches!(self, Self::Run | Self::Check | Self::Ast | Self::Typed)
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Run => "calistir",
            Self::Check => "kontrol",
            Self::Ast => "ast",
            Self::Typed => "typed",
            Self::Help => "yardim",
            Self::Version => "surum",
            Self::Examples => "ornekler",
            Self::Repl => "repl",
        }
    }
}

fn usage(program: &str) -> String {
    format!(
        "Anadil {}\n\nKullanim:\n  {program} <dosya.ana>\n  {program} calistir <dosya.ana>\n  {program} kontrol <dosya.ana>\n  {program} ast <dosya.ana>\n  {program} typed <dosya.ana>\n  {program} repl\n  {program} ornekler\n  {program} surum\n  {program} yardim",
        env!("CARGO_PKG_VERSION")
    )
}

fn run_standalone_command(command: Command, program: &str) {
    match command {
        Command::Help => println!("{}", usage(program)),
        Command::Version => println!("Anadil {}", env!("CARGO_PKG_VERSION")),
        Command::Examples => print_examples(),
        Command::Repl => run_repl(),
        Command::Run | Command::Check | Command::Ast | Command::Typed => unreachable!(),
    }
}

fn print_examples() {
    println!("Ornek programlar:");
    println!("  examples\\topla.ana           Toplama");
    println!("  examples\\dongu.ana           Sayacli dongu, devam, kir");
    println!("  examples\\kosul.ana           Kosul ve mantik donen fonksiyon");
    println!("  examples\\fonksiyon.ana       Ic ice fonksiyon cagrilari");
    println!("  examples\\mantik.ana          Mantik degerleri");
    println!("  examples\\kosullu_dongu.ana   Kosullu dongu");
    println!("  examples\\sonsuz_dongu.ana    Sonsuz dongu ve kir");
    println!("  examples\\kapsam.ana          Scope/kapsam davranisi");
    println!("  examples\\negatif.ana         Negatif sayilar ve unary eksi");
    println!("  examples\\hata_tip.ana        Bilerek hatali tip ornegi");
    println!("  examples\\hata_ana_yok.ana    Bilerek Ana() eksik ornegi");
}

fn run_command(command: Command, source: &str) -> Result<(), String> {
    match command {
        Command::Run => match run_source(source) {
            Ok(output) if output.is_empty() => Ok(()),
            Ok(output) => {
                println!("{output}");
                Ok(())
            }
            Err(message) => Err(message),
        },
        Command::Check => {
            compile_source(source)?;
            println!("Tamam: program gecerli.");
            Ok(())
        }
        Command::Ast => {
            let program = parse_source(source)?;
            println!("{program:#?}");
            Ok(())
        }
        Command::Typed => {
            let program = compile_source(source)?;
            println!("{program:#?}");
            Ok(())
        }
        Command::Help | Command::Version | Command::Examples | Command::Repl => unreachable!(),
    }
}

fn run_repl() {
    println!("Anadil {} REPL", env!("CARGO_PKG_VERSION"));
    println!("Cok satirli giris desteklenir. Yardim icin :yardim, cikmak icin :cik yaz.");

    let stdin = io::stdin();
    let mut input = String::new();
    let mut buffer = String::new();
    let mut session = ReplSession::new();

    loop {
        print!("{}", if buffer.is_empty() { "> " } else { "| " });
        if let Err(error) = io::stdout().flush() {
            eprintln!("Cikti yazilamadi: {error}");
            return;
        }

        input.clear();
        match stdin.read_line(&mut input) {
            Ok(0) => break,
            Ok(_) => {}
            Err(error) => {
                eprintln!("Girdi okunamadi: {error}");
                break;
            }
        }

        let trimmed = input.trim();
        if buffer.is_empty() && trimmed.is_empty() {
            continue;
        }

        if buffer.is_empty() && matches!(trimmed, ":cik" | ":q" | "cik" | "exit") {
            break;
        }

        if buffer.is_empty() && matches!(trimmed, ":yardim" | ":help" | "yardim") {
            print_repl_help();
            continue;
        }

        buffer.push_str(&input);

        if !is_repl_input_complete(&buffer) {
            continue;
        }

        let entry = buffer.trim().to_string();
        buffer.clear();

        match session.execute(&entry) {
            Ok(ReplOutcome::StoredFunction) => println!("Fonksiyon kaydedildi."),
            Ok(ReplOutcome::Output(output)) if output.is_empty() => {}
            Ok(ReplOutcome::Output(output)) => println!("{output}"),
            Err(message) => eprintln!("{message}"),
        }
    }
}

fn print_repl_help() {
    println!("Ornekler:");
    println!("  yazdir(10);");
    println!("  yazdir(10 + -3);");
    println!("  x: sayı = 5;");
    println!("  Kare(x: sayı) -> sayı {{");
    println!("      dön x * x;");
    println!("  }}");
    println!("  yazdir(Kare(5));");
    println!("  Ana() {{ yazdir(10); }}");
    println!();
    println!("Fonksiyon tanimlari oturum boyunca saklanir.");
    println!("Degiskenler satirlar arasinda saklanmaz.");
}

#[derive(Debug, Default)]
struct ReplSession {
    functions: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
enum ReplOutcome {
    StoredFunction,
    Output(String),
}

impl ReplSession {
    fn new() -> Self {
        Self::default()
    }

    fn execute(&mut self, input: &str) -> Result<ReplOutcome, String> {
        if self.is_function_definition(input) {
            self.store_function(input)?;
            Ok(ReplOutcome::StoredFunction)
        } else {
            run_source(&self.prepare_source(input)).map(ReplOutcome::Output)
        }
    }

    fn is_function_definition(&self, input: &str) -> bool {
        if looks_like_entry_program(input) {
            return false;
        }

        let source = self.source_with_entry(input, "");
        parse_source(&source).is_ok()
    }

    fn store_function(&mut self, input: &str) -> Result<(), String> {
        let source = self.source_with_entry(input, "");
        compile_source(&source)?;
        self.functions.push(input.to_string());
        Ok(())
    }

    fn prepare_source(&self, input: &str) -> String {
        if looks_like_entry_program(input) {
            self.source_with_prelude(input)
        } else {
            self.source_with_entry("", input)
        }
    }

    fn source_with_entry(&self, extra_functions: &str, body: &str) -> String {
        let mut source = self.source_with_prelude(extra_functions);
        source.push_str("\nAna() {\n");
        if !body.trim().is_empty() {
            source.push_str("    ");
            source.push_str(body);
            source.push('\n');
        }
        source.push_str("}\n");
        source
    }

    fn source_with_prelude(&self, input: &str) -> String {
        let mut source = String::new();
        for function in &self.functions {
            source.push_str(function);
            source.push_str("\n\n");
        }
        source.push_str(input);
        source.push('\n');
        source
    }
}

fn is_repl_input_complete(input: &str) -> bool {
    brace_balance(input) == 0
}

fn brace_balance(input: &str) -> i32 {
    let mut balance = 0;
    for line in input.lines() {
        let code = line.split_once("//").map_or(line, |(code, _comment)| code);
        for ch in code.chars() {
            match ch {
                '{' => balance += 1,
                '}' => balance -= 1,
                _ => {}
            }
        }
    }
    balance
}

fn looks_like_entry_program(input: &str) -> bool {
    input.contains("Ana(")
}

#[cfg(test)]
mod tests {
    use super::{parse_args, Command, ParsedArgs};

    #[test]
    fn accepts_legacy_run_form() {
        let args = vec!["anadil".to_string(), "examples/topla.ana".to_string()];
        let ParsedArgs::WithFile { command, path } = parse_args(&args).expect("args should parse")
        else {
            panic!("expected file command");
        };

        assert!(matches!(command, Command::Run));
        assert_eq!(path, "examples/topla.ana");
    }

    #[test]
    fn accepts_explicit_check_form() {
        let args = vec![
            "anadil".to_string(),
            "kontrol".to_string(),
            "examples/topla.ana".to_string(),
        ];
        let ParsedArgs::WithFile { command, path } = parse_args(&args).expect("args should parse")
        else {
            panic!("expected file command");
        };

        assert!(matches!(command, Command::Check));
        assert_eq!(path, "examples/topla.ana");
    }

    #[test]
    fn accepts_standalone_help_form() {
        let args = vec!["anadil".to_string(), "yardim".to_string()];
        let ParsedArgs::Standalone(command) = parse_args(&args).expect("args should parse") else {
            panic!("expected standalone command");
        };

        assert!(matches!(command, Command::Help));
    }

    #[test]
    fn accepts_standalone_repl_form() {
        let args = vec!["anadil".to_string(), "repl".to_string()];
        let ParsedArgs::Standalone(command) = parse_args(&args).expect("args should parse") else {
            panic!("expected standalone command");
        };

        assert!(matches!(command, Command::Repl));
    }

    #[test]
    fn wraps_repl_statement_in_entry_point() {
        let session = super::ReplSession::new();
        let source = session.prepare_source("yazdir(10);");

        assert!(source.contains("Ana()"));
        assert!(source.contains("yazdir(10);"));
    }

    #[test]
    fn keeps_repl_full_program_unchanged() {
        let session = super::ReplSession::new();
        let source = "Ana() { yazdir(10); }";

        assert_eq!(session.prepare_source(source), format!("{source}\n"));
    }

    #[test]
    fn detects_multiline_repl_completion() {
        assert!(!super::is_repl_input_complete("Kare(x: sayı) -> sayı {\n"));
        assert!(super::is_repl_input_complete(
            "Kare(x: sayı) -> sayı {\n    dön x * x;\n}\n"
        ));
    }

    #[test]
    fn stores_repl_function_for_later_calls() {
        let mut session = super::ReplSession::new();
        let function = r#"
Kare(x: sayı) -> sayı {
    dön x * x;
}
"#;

        assert_eq!(
            session.execute(function).expect("function should store"),
            super::ReplOutcome::StoredFunction
        );
        assert_eq!(
            session
                .execute("yazdir(Kare(5));")
                .expect("call should run"),
            super::ReplOutcome::Output("25".to_string())
        );
    }

    #[test]
    fn rejects_file_for_standalone_command() {
        let args = vec![
            "anadil".to_string(),
            "surum".to_string(),
            "examples/topla.ana".to_string(),
        ];

        let error = parse_args(&args).expect_err("args should fail");
        assert!(error.contains("dosya yolu almaz"));
    }

    #[test]
    fn rejects_unknown_command() {
        let args = vec![
            "anadil".to_string(),
            "bilinmeyen".to_string(),
            "examples/topla.ana".to_string(),
        ];

        let error = parse_args(&args).expect_err("args should fail");
        assert!(error.contains("Bilinmeyen komut"));
    }
}
