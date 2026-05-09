use std::{
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process,
};

mod ide;

use anadil::{
    check_source, compile_source, diagnostics::Diagnostic, emit_native_asm_source, parse_source,
    run_source, run_source_diagnostic,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Command {
    Run,
    Check,
    Ast,
    Typed,
    Asm,
    WriteAsm,
    CompileNative,
    Help,
    Version,
    Examples,
    Ide,
    Repl,
}

#[derive(Debug)]
enum ParsedArgs<'a> {
    WithFile {
        command: Command,
        path: &'a str,
        output: OutputFormat,
    },
    Standalone(Command),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputFormat {
    Text,
    Json,
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

    let ParsedArgs::WithFile {
        command,
        path,
        output,
    } = parsed
    else {
        unreachable!();
    };

    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            let message = format!("Dosya okunamadi `{path}`: {error}");
            if output == OutputFormat::Json {
                let diagnostic = Diagnostic::io(message);
                match command {
                    Command::Run => println!("{}", json_run_result(false, "", &[diagnostic])),
                    Command::CompileNative => {
                        println!("{}", json_build_result(false, None, &[diagnostic]));
                    }
                    _ => println!("{}", json_result(false, &[diagnostic])),
                }
            } else {
                eprintln!("{message}");
            }
            process::exit(1);
        }
    };

    if let Err(message) = run_command(command, path, &source, output) {
        if !message.is_empty() {
            eprintln!("{message}");
        }
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
                output: OutputFormat::Text,
            }),
        },
        [_program, command, path] => {
            let command = parse_command(command)?;
            if command.requires_file() {
                Ok(ParsedArgs::WithFile {
                    command,
                    path: path.as_str(),
                    output: OutputFormat::Text,
                })
            } else {
                Err(format!("`{}` komutu dosya yolu almaz.", command.name()))
            }
        }
        [_program, command, first, second] => {
            let command = parse_command(command)?;
            let (output, path) = parse_json_file_args(command, first, second)?;
            Ok(ParsedArgs::WithFile {
                command,
                path,
                output,
            })
        }
        _ => Err("Gecersiz arguman sayisi.".to_string()),
    }
}

fn parse_json_file_args<'a>(
    command: Command,
    first: &'a str,
    second: &'a str,
) -> Result<(OutputFormat, &'a str), String> {
    if !command.requires_file() {
        return Err(format!("`{}` komutu dosya yolu almaz.", command.name()));
    }

    match (first, second) {
        ("--json", path) | (path, "--json") => {
            if command.supports_json() {
                Ok((OutputFormat::Json, path))
            } else {
                Err(format!("`{}` komutu `--json` desteklemez.", command.name()))
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
        "asm" | "native-asm" => Ok(Command::Asm),
        "asm-yaz" | "asm-write" => Ok(Command::WriteAsm),
        "derle" | "native" | "native-compile" => Ok(Command::CompileNative),
        "yardim" | "yardım" | "help" | "-h" | "--help" => Ok(Command::Help),
        "surum" | "sürüm" | "version" | "-V" | "--version" => Ok(Command::Version),
        "ornekler" | "örnekler" | "examples" => Ok(Command::Examples),
        "ide" => Ok(Command::Ide),
        "repl" => Ok(Command::Repl),
        _ => Err(format!("Bilinmeyen komut: `{command}`")),
    }
}

impl Command {
    fn requires_file(&self) -> bool {
        matches!(
            self,
            Self::Run
                | Self::Check
                | Self::Ast
                | Self::Typed
                | Self::Asm
                | Self::WriteAsm
                | Self::CompileNative
        )
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Run => "calistir",
            Self::Check => "kontrol",
            Self::Ast => "ast",
            Self::Typed => "typed",
            Self::Asm => "asm",
            Self::WriteAsm => "asm-yaz",
            Self::CompileNative => "derle",
            Self::Help => "yardim",
            Self::Version => "surum",
            Self::Examples => "ornekler",
            Self::Ide => "ide",
            Self::Repl => "repl",
        }
    }

    fn supports_json(&self) -> bool {
        matches!(self, Self::Run | Self::Check | Self::CompileNative)
    }
}

fn usage(program: &str) -> String {
    format!(
        "Anadil {}\n\nKullanim:\n  {program} <dosya.ana>\n  {program} calistir <dosya.ana>\n  {program} calistir --json <dosya.ana>\n  {program} kontrol <dosya.ana>\n  {program} kontrol --json <dosya.ana>\n  {program} ast <dosya.ana>\n  {program} typed <dosya.ana>\n  {program} asm <dosya.ana>\n  {program} asm-yaz <dosya.ana>\n  {program} derle <dosya.ana>\n  {program} derle --json <dosya.ana>\n  {program} ide\n  {program} repl\n  {program} ornekler\n  {program} surum\n  {program} yardim",
        env!("CARGO_PKG_VERSION")
    )
}

fn run_standalone_command(command: Command, program: &str) {
    match command {
        Command::Help => println!("{}", usage(program)),
        Command::Version => println!("Anadil {}", env!("CARGO_PKG_VERSION")),
        Command::Examples => print_examples(),
        Command::Ide => {
            if let Err(message) = ide::run() {
                eprintln!("{message}");
                process::exit(1);
            }
        }
        Command::Repl => run_repl(),
        Command::Run
        | Command::Check
        | Command::Ast
        | Command::Typed
        | Command::Asm
        | Command::WriteAsm
        | Command::CompileNative => unreachable!(),
    }
}

fn print_examples() {
    println!("Ornek programlar:");
    println!("  examples\\topla.ana           Toplama");
    println!("  examples\\dongu.ana           Sayacli dongu, devam, kir");
    println!("  examples\\kosul.ana           Kosul ve mantik donen fonksiyon");
    println!("  examples\\fonksiyon.ana       Ic ice fonksiyon cagrilari");
    println!("  examples\\mantik.ana          Mantik degerleri");
    println!("  examples\\metin.ana           Metin degerleri");
    println!("  examples\\kosullu_dongu.ana   Kosullu dongu");
    println!("  examples\\sonsuz_dongu.ana    Sonsuz dongu ve kir");
    println!("  examples\\kapsam.ana          Scope/kapsam davranisi");
    println!("  examples\\negatif.ana         Negatif sayilar ve unary eksi");
    println!("  examples\\native_mvp.ana      Native MVP demo programi");
    println!("  examples\\hata_tip.ana        Bilerek hatali tip ornegi");
    println!("  examples\\hata_ana_yok.ana    Bilerek Ana() eksik ornegi");
    println!();
    println!("Native derleme:");
    println!("  cargo run -- derle examples\\topla.ana");
}

fn run_command(
    command: Command,
    path: &str,
    source: &str,
    output: OutputFormat,
) -> Result<(), String> {
    match command {
        Command::Run => {
            if output == OutputFormat::Json {
                match run_source_diagnostic(source) {
                    Ok(program_output) => {
                        println!("{}", json_run_result(true, &program_output, &[]));
                        Ok(())
                    }
                    Err(diagnostic) => {
                        println!("{}", json_run_result(false, "", &[diagnostic]));
                        Err(String::new())
                    }
                }
            } else {
                match run_source(source) {
                    Ok(output) if output.is_empty() => Ok(()),
                    Ok(output) => {
                        println!("{output}");
                        Ok(())
                    }
                    Err(message) => Err(message),
                }
            }
        }
        Command::Check => {
            if output == OutputFormat::Json {
                match check_source(source) {
                    Ok(()) => {
                        println!("{}", json_result(true, &[]));
                        Ok(())
                    }
                    Err(diagnostic) => {
                        println!("{}", json_result(false, &[diagnostic]));
                        Err(String::new())
                    }
                }
            } else {
                compile_source(source)?;
                println!("Tamam: program gecerli.");
                Ok(())
            }
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
        Command::Asm => {
            println!("{}", emit_native_asm_source(source)?);
            Ok(())
        }
        Command::WriteAsm => {
            let asm_path = write_native_asm(path, source)?;
            println!("Assembly yazildi: {}", asm_path.display());
            Ok(())
        }
        Command::CompileNative => {
            if output == OutputFormat::Json {
                if let Err(diagnostic) = check_source(source) {
                    println!("{}", json_build_result(false, None, &[diagnostic]));
                    return Err(String::new());
                }

                match compile_native(path, source) {
                    Ok(exe_path) => {
                        println!("{}", json_build_result(true, Some(&exe_path), &[]));
                        Ok(())
                    }
                    Err(message) => {
                        println!(
                            "{}",
                            json_build_result(false, None, &[Diagnostic::native(message)])
                        );
                        Err(String::new())
                    }
                }
            } else {
                let output = compile_native(path, source)?;
                println!("Native executable yazildi: {}", output.display());
                Ok(())
            }
        }
        Command::Help | Command::Version | Command::Examples | Command::Ide | Command::Repl => {
            unreachable!()
        }
    }
}

fn json_result(ok: bool, diagnostics: &[Diagnostic]) -> String {
    let diagnostics = diagnostics
        .iter()
        .map(json_diagnostic)
        .collect::<Vec<_>>()
        .join(",");
    format!("{{\"ok\":{ok},\"diagnostics\":[{diagnostics}]}}")
}

fn json_run_result(ok: bool, output: &str, diagnostics: &[Diagnostic]) -> String {
    let diagnostics = diagnostics
        .iter()
        .map(json_diagnostic)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"ok\":{ok},\"output\":\"{}\",\"diagnostics\":[{diagnostics}]}}",
        json_escape(output)
    )
}

fn json_build_result(ok: bool, exe: Option<&Path>, diagnostics: &[Diagnostic]) -> String {
    let diagnostics = diagnostics
        .iter()
        .map(json_diagnostic)
        .collect::<Vec<_>>()
        .join(",");
    let exe = exe
        .map(|path| format!("\"{}\"", json_escape(&path.display().to_string())))
        .unwrap_or_else(|| "null".to_string());
    format!("{{\"ok\":{ok},\"exe\":{exe},\"diagnostics\":[{diagnostics}]}}")
}

fn json_diagnostic(diagnostic: &Diagnostic) -> String {
    let (line, column) = match diagnostic.span {
        Some(span) => (span.line.to_string(), span.column.to_string()),
        None => ("null".to_string(), "null".to_string()),
    };

    format!(
        "{{\"severity\":\"{}\",\"stage\":\"{}\",\"message\":\"{}\",\"line\":{},\"column\":{}}}",
        diagnostic.severity.as_str(),
        diagnostic.stage.as_str(),
        json_escape(&diagnostic.message),
        line,
        column
    )
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{08}' => escaped.push_str("\\b"),
            '\u{0c}' => escaped.push_str("\\f"),
            ch if ch <= '\u{1f}' => escaped.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => escaped.push(ch),
        }
    }
    escaped
}

fn write_native_asm(path: &str, source: &str) -> Result<PathBuf, String> {
    let asm = emit_native_asm_source(source)?;
    let asm_path = output_path(path, "asm");
    fs::write(&asm_path, asm).map_err(|error| {
        format!(
            "Assembly dosyasi yazilamadi `{}`: {error}",
            asm_path.display()
        )
    })?;
    Ok(asm_path)
}

fn compile_native(path: &str, source: &str) -> Result<PathBuf, String> {
    let vcvars64 = if command_available("ml64") && command_available("link") {
        None
    } else {
        Some(find_vcvars64().ok_or_else(|| {
            "Native derleme icin Visual Studio Build Tools C++ araclari gerekli. \
             `ml64`, `link` veya `vcvars64.bat` bulunamadi."
                .to_string()
        })?)
    };

    let exe_path = output_path(path, "exe");
    let build_paths = create_native_build_paths(path)?;
    let asm = emit_native_asm_source(source)?;
    fs::write(&build_paths.asm, asm).map_err(|error| {
        format!(
            "Native build assembly dosyasi yazilamadi `{}`: {error}",
            build_paths.asm.display()
        )
    })?;

    run_tool(
        "ml64",
        &[
            "/nologo".to_string(),
            "/c".to_string(),
            format!("/Fo{}", build_paths.obj.display()),
            build_paths.asm.display().to_string(),
        ],
        vcvars64.as_deref(),
    )?;
    run_tool(
        "link",
        &[
            "/nologo".to_string(),
            "/SUBSYSTEM:CONSOLE".to_string(),
            "/ENTRY:main".to_string(),
            format!("/OUT:{}", build_paths.exe.display()),
            build_paths.obj.display().to_string(),
            "msvcrt.lib".to_string(),
            "ucrt.lib".to_string(),
            "vcruntime.lib".to_string(),
            "legacy_stdio_definitions.lib".to_string(),
        ],
        vcvars64.as_deref(),
    )?;

    fs::copy(&build_paths.exe, &exe_path).map_err(|error| {
        format!(
            "Native executable hedefe kopyalanamadi `{}` -> `{}`: {error}",
            build_paths.exe.display(),
            exe_path.display()
        )
    })?;

    Ok(exe_path)
}

#[derive(Debug)]
struct NativeBuildPaths {
    asm: PathBuf,
    obj: PathBuf,
    exe: PathBuf,
}

fn create_native_build_paths(source_path: &str) -> Result<NativeBuildPaths, String> {
    let stem = Path::new(source_path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(sanitize_build_name)
        .filter(|stem| !stem.is_empty())
        .unwrap_or_else(|| "program".to_string());

    let dir =
        PathBuf::from("target")
            .join("native-build")
            .join(format!("{}-{}", process::id(), stem));

    fs::create_dir_all(&dir).map_err(|error| {
        format!(
            "Native build klasoru olusturulamadi `{}`: {error}",
            dir.display()
        )
    })?;

    Ok(NativeBuildPaths {
        asm: dir.join(format!("{stem}.asm")),
        obj: dir.join(format!("{stem}.obj")),
        exe: dir.join(format!("{stem}.exe")),
    })
}

fn sanitize_build_name(name: &str) -> String {
    name.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn run_tool(program: &str, args: &[String], vcvars64: Option<&Path>) -> Result<(), String> {
    if let Some(vcvars64) = vcvars64 {
        run_process_in_vcvars(vcvars64, program, args)
    } else {
        run_process(program, args)
    }
}

fn run_process(program: &str, args: &[String]) -> Result<(), String> {
    let output = process::Command::new(program)
        .args(args)
        .output()
        .map_err(|error| format!("`{program}` calistirilamadi: {error}"))?;

    if output.status.success() {
        return Ok(());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(format!(
        "`{program}` basarisiz oldu.\nstdout:\n{stdout}\nstderr:\n{stderr}"
    ))
}

fn run_process_in_vcvars(vcvars64: &Path, program: &str, args: &[String]) -> Result<(), String> {
    let mut script = String::new();
    script.push_str("@echo off\r\n");
    script.push_str("call ");
    script.push_str(&quote_cmd_arg(&vcvars64.display().to_string()));
    script.push_str(" >nul\r\n");
    script.push_str(&quote_cmd_arg(program));
    for arg in args {
        script.push(' ');
        script.push_str(&quote_cmd_arg(arg));
    }
    script.push_str("\r\nexit /b %ERRORLEVEL%\r\n");

    let script_path =
        env::temp_dir().join(format!("anadil-native-{}-{program}.bat", process::id()));
    fs::write(&script_path, script).map_err(|error| {
        format!(
            "Native derleme gecici komut dosyasi yazilamadi `{}`: {error}",
            script_path.display()
        )
    })?;

    let result = run_process(
        "cmd.exe",
        &[
            "/d".to_string(),
            "/c".to_string(),
            script_path.display().to_string(),
        ],
    );
    let _ = fs::remove_file(script_path);
    result
}

fn output_path(path: &str, extension: &str) -> PathBuf {
    Path::new(path).with_extension(extension)
}

fn command_available(command: &str) -> bool {
    let Some(paths) = env::var_os("PATH") else {
        return false;
    };

    let candidates = command_candidates(command);
    env::split_paths(&paths).any(|dir| {
        candidates
            .iter()
            .any(|candidate| dir.join(candidate).is_file())
    })
}

fn command_candidates(command: &str) -> Vec<String> {
    if Path::new(command).extension().is_some() {
        return vec![command.to_string()];
    }

    vec![
        command.to_string(),
        format!("{command}.exe"),
        format!("{command}.cmd"),
        format!("{command}.bat"),
    ]
}

fn find_vcvars64() -> Option<PathBuf> {
    [
        r"C:\Program Files\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat",
        r"C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat",
        r"C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvars64.bat",
        r"C:\Program Files\Microsoft Visual Studio\2022\Enterprise\VC\Auxiliary\Build\vcvars64.bat",
        r"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat",
        r"C:\Program Files (x86)\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat",
        r"C:\Program Files (x86)\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvars64.bat",
        r"C:\Program Files (x86)\Microsoft Visual Studio\2022\Enterprise\VC\Auxiliary\Build\vcvars64.bat",
    ]
    .into_iter()
    .map(PathBuf::from)
    .find(|path| {
        path.is_file()
            && path
                .parent()
                .is_some_and(|parent| parent.join("vcvarsall.bat").is_file())
    })
}

fn quote_cmd_arg(arg: &str) -> String {
    if arg
        .chars()
        .any(|ch| ch.is_whitespace() || matches!(ch, '&' | '(' | ')' | '^' | '|' | '<' | '>'))
    {
        format!("\"{}\"", arg.replace('"', "\"\""))
    } else {
        arg.to_string()
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
    println!("  yazdır(10);");
    println!("  yazdır(10 + -3);");
    println!("  x: sayı = 5;");
    println!("  Kare(x: sayı) -> sayı {{");
    println!("      dön x * x;");
    println!("  }}");
    println!("  yazdır(Kare(5));");
    println!("  Ana() {{ yazdır(10); }}");
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
    use super::{parse_args, sanitize_build_name, Command, ParsedArgs};

    #[test]
    fn accepts_legacy_run_form() {
        let args = vec!["anadil".to_string(), "examples/topla.ana".to_string()];
        let ParsedArgs::WithFile {
            command,
            path,
            output,
        } = parse_args(&args).expect("args should parse")
        else {
            panic!("expected file command");
        };

        assert!(matches!(command, Command::Run));
        assert_eq!(path, "examples/topla.ana");
        assert_eq!(output, super::OutputFormat::Text);
    }

    #[test]
    fn sanitizes_native_build_file_name_for_cmd_tools() {
        assert_eq!(sanitize_build_name("adsiz"), "adsiz");
        assert_eq!(sanitize_build_name("Masaustu test"), "Masaustu_test");
        assert_eq!(sanitize_build_name("çalışma-1"), "_al__ma-1");
    }

    #[test]
    fn accepts_explicit_check_form() {
        let args = vec![
            "anadil".to_string(),
            "kontrol".to_string(),
            "examples/topla.ana".to_string(),
        ];
        let ParsedArgs::WithFile {
            command,
            path,
            output,
        } = parse_args(&args).expect("args should parse")
        else {
            panic!("expected file command");
        };

        assert!(matches!(command, Command::Check));
        assert_eq!(path, "examples/topla.ana");
        assert_eq!(output, super::OutputFormat::Text);
    }

    #[test]
    fn accepts_json_check_form() {
        let args = vec![
            "anadil".to_string(),
            "kontrol".to_string(),
            "--json".to_string(),
            "examples/topla.ana".to_string(),
        ];
        let ParsedArgs::WithFile {
            command,
            path,
            output,
        } = parse_args(&args).expect("args should parse")
        else {
            panic!("expected file command");
        };

        assert!(matches!(command, Command::Check));
        assert_eq!(path, "examples/topla.ana");
        assert_eq!(output, super::OutputFormat::Json);
    }

    #[test]
    fn accepts_json_run_form() {
        let args = vec![
            "anadil".to_string(),
            "calistir".to_string(),
            "--json".to_string(),
            "examples/topla.ana".to_string(),
        ];
        let ParsedArgs::WithFile {
            command,
            path,
            output,
        } = parse_args(&args).expect("args should parse")
        else {
            panic!("expected file command");
        };

        assert!(matches!(command, Command::Run));
        assert_eq!(path, "examples/topla.ana");
        assert_eq!(output, super::OutputFormat::Json);
    }

    #[test]
    fn accepts_json_native_compile_form() {
        let args = vec![
            "anadil".to_string(),
            "derle".to_string(),
            "--json".to_string(),
            "examples/topla.ana".to_string(),
        ];
        let ParsedArgs::WithFile {
            command,
            path,
            output,
        } = parse_args(&args).expect("args should parse")
        else {
            panic!("expected file command");
        };

        assert!(matches!(command, Command::CompileNative));
        assert_eq!(path, "examples/topla.ana");
        assert_eq!(output, super::OutputFormat::Json);
    }

    #[test]
    fn rejects_json_for_unsupported_commands() {
        let args = vec![
            "anadil".to_string(),
            "ast".to_string(),
            "--json".to_string(),
            "examples/topla.ana".to_string(),
        ];

        let error = parse_args(&args).expect_err("args should fail");
        assert!(error.contains("desteklemez"));
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
    fn accepts_standalone_ide_form() {
        let args = vec!["anadil".to_string(), "ide".to_string()];
        let ParsedArgs::Standalone(command) = parse_args(&args).expect("args should parse") else {
            panic!("expected standalone command");
        };

        assert!(matches!(command, Command::Ide));
    }

    #[test]
    fn wraps_repl_statement_in_entry_point() {
        let session = super::ReplSession::new();
        let source = session.prepare_source("yazdır(10);");

        assert!(source.contains("Ana()"));
        assert!(source.contains("yazdır(10);"));
    }

    #[test]
    fn keeps_repl_full_program_unchanged() {
        let session = super::ReplSession::new();
        let source = "Ana() { yazdır(10); }";

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
                .execute("yazdır(Kare(5));")
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
