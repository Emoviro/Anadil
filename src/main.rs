use std::{
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process, thread,
    time::Duration,
};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// `Command`'a Windows uzerinde CREATE_NO_WINDOW flag'i ekler; boylece
/// GUI process'lerden (orn. anadil-ide.exe -> anadil.exe -> cmd.exe)
/// spawn edildiginde console penceresi yanip sonmez.
fn hide_command_window(command: &mut process::Command) {
    #[cfg(windows)]
    {
        command.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(not(windows))]
    {
        let _ = command;
    }
}

mod ide;

use anadil::{
    check_source, compile_source, diagnostics::Diagnostic, emit_ir_source, emit_native_asm_source,
    parse_source, run_source, run_source_diagnostic,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Command {
    Run,
    Interpret,
    Check,
    Ast,
    Typed,
    Ir,
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
                    Command::Run | Command::Interpret => {
                        println!("{}", json_run_result(false, "", &[diagnostic]));
                    }
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
        "yorumla" | "interpret" | "interp" => Ok(Command::Interpret),
        "kontrol" | "check" => Ok(Command::Check),
        "ast" => Ok(Command::Ast),
        "typed" => Ok(Command::Typed),
        "ir" => Ok(Command::Ir),
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
                | Self::Interpret
                | Self::Check
                | Self::Ast
                | Self::Typed
                | Self::Ir
                | Self::Asm
                | Self::WriteAsm
                | Self::CompileNative
        )
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Run => "calistir",
            Self::Interpret => "yorumla",
            Self::Check => "kontrol",
            Self::Ast => "ast",
            Self::Typed => "typed",
            Self::Ir => "ir",
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
        matches!(
            self,
            Self::Run | Self::Interpret | Self::Check | Self::CompileNative
        )
    }
}

fn usage(program: &str) -> String {
    format!(
        "Anadil {}\n\nKullanim:\n  {program} <dosya.ana>\n  {program} calistir <dosya.ana>\n  {program} calistir --json <dosya.ana>\n  {program} yorumla <dosya.ana>\n  {program} yorumla --json <dosya.ana>\n  {program} kontrol <dosya.ana>\n  {program} kontrol --json <dosya.ana>\n  {program} ast <dosya.ana>\n  {program} typed <dosya.ana>\n  {program} ir <dosya.ana>\n  {program} asm <dosya.ana>\n  {program} asm-yaz <dosya.ana>\n  {program} derle <dosya.ana>\n  {program} derle --json <dosya.ana>\n  {program} ide\n  {program} repl\n  {program} ornekler\n  {program} surum\n  {program} yardim",
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
        | Command::Interpret
        | Command::Check
        | Command::Ast
        | Command::Typed
        | Command::Ir
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
        Command::Run => run_native_and_maybe_json(path, source, output),
        Command::Interpret => {
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
        Command::Ir => {
            println!("{}", emit_ir_source(source)?);
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

fn run_native_and_maybe_json(path: &str, source: &str, output: OutputFormat) -> Result<(), String> {
    if output == OutputFormat::Json {
        if let Err(diagnostic) = check_source(source) {
            println!("{}", json_run_result(false, "", &[diagnostic]));
            return Err(String::new());
        }

        let exe_path = match compile_native(path, source) {
            Ok(exe_path) => exe_path,
            Err(message) => {
                println!(
                    "{}",
                    json_run_result(false, "", &[Diagnostic::native(message)])
                );
                return Err(String::new());
            }
        };

        match run_native_executable(&exe_path) {
            Ok(native_output) => {
                let program_output = normalize_program_output(&native_output.stdout);
                if native_output.status.success() {
                    println!("{}", json_run_result(true, &program_output, &[]));
                    Ok(())
                } else {
                    let diagnostic = Diagnostic::native(format!(
                        "Native program basarisiz bitti: {}",
                        exit_code_label(&native_output.status)
                    ));
                    println!("{}", json_run_result(false, &program_output, &[diagnostic]));
                    Err(String::new())
                }
            }
            Err(message) => {
                println!(
                    "{}",
                    json_run_result(false, "", &[Diagnostic::native(message)])
                );
                Err(String::new())
            }
        }
    } else {
        let exe_path = compile_native(path, source)?;
        let native_output = run_native_executable(&exe_path)?;
        print!("{}", String::from_utf8_lossy(&native_output.stdout));
        eprint!("{}", String::from_utf8_lossy(&native_output.stderr));

        if native_output.status.success() {
            Ok(())
        } else {
            Err(format!(
                "Native program basarisiz bitti: {}",
                exit_code_label(&native_output.status)
            ))
        }
    }
}

fn run_native_executable(path: &Path) -> Result<process::Output, String> {
    let mut command = process::Command::new(path);
    if let Some(parent) = path.parent() {
        command.current_dir(parent);
    }
    hide_command_window(&mut command);
    command.output().map_err(|error| {
        format!(
            "Native executable calistirilamadi `{}`: {error}",
            path.display()
        )
    })
}

fn normalize_program_output(output: &[u8]) -> String {
    String::from_utf8_lossy(output)
        .replace("\r\n", "\n")
        .trim_end_matches('\n')
        .to_string()
}

fn exit_code_label(status: &process::ExitStatus) -> String {
    status
        .code()
        .map(|code| code.to_string())
        .unwrap_or_else(|| "signal".to_string())
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
    let has_packaged_runtime = packaged_runtime_lib_path().is_some();
    let vcvars64 = if command_available("ml64")
        && command_available("link")
        && (has_packaged_runtime || command_available("lib"))
    {
        None
    } else {
        Some(find_vcvars64().ok_or_else(build_tools_missing_message)?)
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
    let runtime_lib = ensure_runtime_lib(vcvars64.as_deref())?;
    let runtime_lib_arg = runtime_tool_arg(&runtime_lib, &relative_runtime_lib());
    run_tool(
        "link",
        &[
            "/nologo".to_string(),
            "/SUBSYSTEM:CONSOLE".to_string(),
            "/ENTRY:main".to_string(),
            format!("/OUT:{}", build_paths.exe.display()),
            build_paths.obj.display().to_string(),
            runtime_lib_arg,
            "kernel32.lib".to_string(),
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

fn build_tools_missing_message() -> String {
    "Visual Studio Build Tools bulunamadi.\nIndirme: https://visualstudio.microsoft.com/visual-cpp-build-tools/\nInterpreter ile calistirmak icin: anadil yorumla <dosya>.ana".to_string()
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

fn ensure_runtime_lib(vcvars64: Option<&Path>) -> Result<PathBuf, String> {
    if let Some(runtime_lib) = packaged_runtime_lib_path() {
        return Ok(runtime_lib);
    }

    let _lock = acquire_runtime_cache_lock()?;
    let runtime_asm = runtime_asm_path()?;
    let runtime_obj = runtime_obj_cache_path()?;
    let runtime_lib = runtime_lib_cache_path()?;

    // Tools'a Turkce karakterli absolute path gondermemek icin once
    // cwd-relative ASCII path dene; cwd proje koku ise bu calisir,
    // degilse absolute fallback. Bkz `runtime_tool_arg`.
    let runtime_asm_arg = runtime_tool_arg(&runtime_asm, &relative_runtime_asm());
    let runtime_obj_arg = runtime_tool_arg(&runtime_obj, &relative_runtime_obj());
    let runtime_lib_arg = runtime_tool_arg(&runtime_lib, &relative_runtime_lib());

    if runtime_obj_needs_rebuild(&runtime_asm, &runtime_obj)? {
        run_tool(
            "ml64",
            &[
                "/nologo".to_string(),
                "/c".to_string(),
                format!("/Fo{runtime_obj_arg}"),
                runtime_asm_arg,
            ],
            vcvars64,
        )?;
    }

    if runtime_lib_needs_rebuild(&runtime_obj, &runtime_lib)? {
        run_tool(
            "lib",
            &[
                "/nologo".to_string(),
                format!("/OUT:{runtime_lib_arg}"),
                runtime_obj_arg,
            ],
            vcvars64,
        )?;
    }

    Ok(runtime_lib)
}

fn packaged_runtime_lib_path() -> Option<PathBuf> {
    let exe_path = env::current_exe().ok()?;
    let runtime_lib = packaged_runtime_lib_path_from_exe(&exe_path)?;
    runtime_lib.is_file().then_some(runtime_lib)
}

fn packaged_runtime_lib_path_from_exe(exe_path: &Path) -> Option<PathBuf> {
    Some(
        exe_path
            .parent()?
            .join("runtime")
            .join("anadil_runtime.lib"),
    )
}

fn relative_runtime_asm() -> PathBuf {
    PathBuf::from("runtime").join("anadil_runtime.asm")
}

fn relative_runtime_obj() -> PathBuf {
    PathBuf::from("target")
        .join("native-runtime")
        .join("anadil_runtime.obj")
}

fn relative_runtime_lib() -> PathBuf {
    PathBuf::from("target")
        .join("native-runtime")
        .join("anadil_runtime.lib")
}

/// Tool cagrilarinda kullanilacak path string'ini secer.
///
/// MS toolchain araclari (ml64/lib/link) cmd.exe + .bat aracilığıyla
/// cagrildigi icin .bat dosyasindaki UTF-8 byte'lari OEM codepage'de
/// yanlis yorumlaniyor; bu yuzden path argumanlari saf ASCII olmali.
///
/// Sira:
/// 1. Onerilen statik relative aday (orn. `runtime/anadil_runtime.asm`,
///    `target/native-runtime/anadil_runtime.lib`) gercekten ayni dosyaya
///    isaret ediyorsa kullan.
/// 2. Aksi halde cwd'den absolute hedefe dinamik relative yol hesapla;
///    saf ASCII ise onu kullan. Cwd absolute path'i Turkce karakter
///    icerse bile sorun olmaz; Windows path resolution UTF-16 ile yapilir,
///    biz sadece komut satirina giden string'i ASCII tutmaya calisiyoruz.
/// 3. Son care olarak absolute path string'ini doner.
fn runtime_tool_arg(absolute: &Path, relative: &Path) -> String {
    if relative_path_is_usable(absolute, relative) {
        return relative.display().to_string();
    }

    if let Some(cwd_relative) = cwd_relative_ascii_path(absolute) {
        return cwd_relative;
    }

    absolute.display().to_string()
}

fn relative_path_is_usable(absolute: &Path, relative: &Path) -> bool {
    let Ok(absolute_canon) = fs::canonicalize(absolute) else {
        // Absolute hedef yoksa (henuz olusturulmamis cikti dosyasi gibi),
        // relative parent dizininin var olup olmadigina bakariz.
        return relative
            .parent()
            .map(|parent| parent.as_os_str().is_empty() || parent.is_dir())
            .unwrap_or(false);
    };

    fs::canonicalize(relative)
        .map(|relative_canon| relative_canon == absolute_canon)
        .unwrap_or(false)
}

/// Cwd'den hedef absolute path'e relative bir yol hesaplar; sonuc tam
/// ASCII ise doner. ASCII degilse veya hesaplanamiyorsa `None`.
fn cwd_relative_ascii_path(absolute: &Path) -> Option<String> {
    let cwd = env::current_dir().ok()?;
    let cwd_canon = fs::canonicalize(&cwd).ok()?;
    let absolute_canon = fs::canonicalize(absolute).ok()?;
    let relative = compute_relative_path(&absolute_canon, &cwd_canon)?;
    let display = relative.to_string_lossy().into_owned();
    if display.is_empty() || !display.is_ascii() {
        return None;
    }
    Some(display)
}

/// `target` path'inin `base`'ten goruldugundeki relative formunu uretir.
/// Iki path ayni root'a sahip olmali (canonicalize edilmis halleri tercih).
fn compute_relative_path(target: &Path, base: &Path) -> Option<PathBuf> {
    use std::path::Component;

    let target_components: Vec<Component> = target.components().collect();
    let base_components: Vec<Component> = base.components().collect();

    let mut common = 0;
    for (a, b) in target_components.iter().zip(base_components.iter()) {
        if a == b {
            common += 1;
        } else {
            break;
        }
    }

    if common == 0 {
        // Farkli root'lar; relative yapilamaz.
        return None;
    }

    let mut result = PathBuf::new();
    let parents_needed = base_components.len() - common;
    for _ in 0..parents_needed {
        result.push("..");
    }
    for component in &target_components[common..] {
        match component {
            Component::Normal(part) => result.push(part),
            Component::CurDir => {}
            _ => return None,
        }
    }

    if result.as_os_str().is_empty() {
        // Cwd ile hedef ayni; bos string yerine "." vermek tools icin
        // anlamsiz. Bu durumda relative'in degeri yok, None done.
        return None;
    }
    Some(result)
}

struct RuntimeCacheLock {
    path: PathBuf,
}

impl Drop for RuntimeCacheLock {
    fn drop(&mut self) {
        let _ = fs::remove_dir(&self.path);
    }
}

fn acquire_runtime_cache_lock() -> Result<RuntimeCacheLock, String> {
    let lock_dir = runtime_cache_dir()?.join("anadil_runtime.lock");
    for _ in 0..200 {
        match fs::create_dir(&lock_dir) {
            Ok(()) => {
                return Ok(RuntimeCacheLock { path: lock_dir });
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                thread::sleep(Duration::from_millis(25));
            }
            Err(error) => {
                return Err(format!(
                    "Native runtime cache lock olusturulamadi `{}`: {error}",
                    lock_dir.display()
                ));
            }
        }
    }

    Err(format!(
        "Native runtime cache lock beklerken zaman asimi `{}`",
        lock_dir.display()
    ))
}

fn runtime_asm_path() -> Result<PathBuf, String> {
    let mut tried = Vec::new();
    if let Some(path) = exe_relative_runtime_asm_path() {
        tried.push(path.clone());
        if path.is_file() {
            return Ok(path);
        }
    }

    let dev_path = dev_runtime_asm_path();
    tried.push(dev_path.clone());
    if dev_path.is_file() {
        return Ok(dev_path);
    }

    let tried_paths = tried
        .iter()
        .map(|path| format!("`{}`", path.display()))
        .collect::<Vec<_>>()
        .join(", ");
    Err(format!(
        "Anadil runtime assembly dosyasi bulunamadi. Denenen yollar: {tried_paths}"
    ))
}

fn runtime_obj_cache_path() -> Result<PathBuf, String> {
    Ok(runtime_cache_dir()?.join("anadil_runtime.obj"))
}

fn runtime_lib_cache_path() -> Result<PathBuf, String> {
    Ok(runtime_cache_dir()?.join("anadil_runtime.lib"))
}

fn runtime_cache_dir() -> Result<PathBuf, String> {
    let dir = runtime_cache_dir_candidate()?;
    fs::create_dir_all(&dir).map_err(|error| {
        format!(
            "Native runtime cache klasoru olusturulamadi `{}`: {error}",
            dir.display()
        )
    })?;
    Ok(dir)
}

fn exe_relative_runtime_asm_path() -> Option<PathBuf> {
    let exe_path = env::current_exe().ok()?;
    Some(
        exe_path
            .parent()?
            .join("runtime")
            .join("anadil_runtime.asm"),
    )
}

fn dev_runtime_asm_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("runtime")
        .join("anadil_runtime.asm")
}

fn runtime_cache_dir_candidate() -> Result<PathBuf, String> {
    let dev_dir = dev_runtime_cache_dir();
    if is_development_executable() && dev_dir.parent().is_some_and(|target| target.is_dir()) {
        return Ok(dev_dir);
    }

    let local_app_data = env::var_os("LOCALAPPDATA").ok_or_else(|| {
        "LOCALAPPDATA ortam degiskeni bulunamadi; native runtime cache klasoru secilemedi."
            .to_string()
    })?;
    Ok(PathBuf::from(local_app_data).join("Anadil").join("cache"))
}

fn dev_runtime_cache_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("native-runtime")
}

fn is_development_executable() -> bool {
    let Ok(exe_path) = env::current_exe() else {
        return false;
    };
    is_development_executable_path(&exe_path)
}

fn is_development_executable_path(exe_path: &Path) -> bool {
    let dev_target = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target");
    let Ok(exe_path) = fs::canonicalize(exe_path) else {
        return false;
    };
    fs::canonicalize(dev_target)
        .map(|target| exe_path.starts_with(target))
        .unwrap_or(false)
}

fn runtime_obj_needs_rebuild(runtime_asm: &Path, runtime_obj: &Path) -> Result<bool, String> {
    if !runtime_obj.is_file() {
        return Ok(true);
    }

    let asm_modified = fs::metadata(runtime_asm)
        .and_then(|metadata| metadata.modified())
        .map_err(|error| {
            format!(
                "Native runtime assembly zamani okunamadi `{}`: {error}",
                runtime_asm.display()
            )
        })?;
    let obj_modified = fs::metadata(runtime_obj)
        .and_then(|metadata| metadata.modified())
        .map_err(|error| {
            format!(
                "Native runtime object zamani okunamadi `{}`: {error}",
                runtime_obj.display()
            )
        })?;

    Ok(asm_modified > obj_modified)
}

fn runtime_lib_needs_rebuild(runtime_obj: &Path, runtime_lib: &Path) -> Result<bool, String> {
    if !runtime_lib.is_file() {
        return Ok(true);
    }

    let obj_modified = fs::metadata(runtime_obj)
        .and_then(|metadata| metadata.modified())
        .map_err(|error| {
            format!(
                "Native runtime object zamani okunamadi `{}`: {error}",
                runtime_obj.display()
            )
        })?;
    let lib_modified = fs::metadata(runtime_lib)
        .and_then(|metadata| metadata.modified())
        .map_err(|error| {
            format!(
                "Native runtime library zamani okunamadi `{}`: {error}",
                runtime_lib.display()
            )
        })?;

    Ok(obj_modified > lib_modified)
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
    let mut command = process::Command::new(program);
    command.args(args);
    hide_command_window(&mut command);
    let output = command
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
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{
        packaged_runtime_lib_path_from_exe, parse_args, runtime_lib_needs_rebuild,
        runtime_obj_needs_rebuild, sanitize_build_name, Command, ParsedArgs,
    };

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

    fn unique_runtime_cache_test_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after UNIX_EPOCH")
            .as_nanos();
        let dir = PathBuf::from("target")
            .join("runtime_cache_unit_tests")
            .join(format!("{name}_{}_{}", std::process::id(), nanos));
        fs::create_dir_all(&dir).expect("test cache dir should be created");
        dir
    }

    #[test]
    fn runtime_obj_cache_rebuilds_when_obj_is_missing() {
        let dir = unique_runtime_cache_test_dir("obj_missing");
        let runtime_asm = dir.join("anadil_runtime.asm");
        let runtime_obj = dir.join("anadil_runtime.obj");
        fs::write(&runtime_asm, "; test runtime").expect("runtime asm should be written");
        let _ = fs::remove_file(&runtime_obj);

        assert!(runtime_obj_needs_rebuild(&runtime_asm, &runtime_obj)
            .expect("runtime cache state should be checked"));
    }

    #[test]
    fn runtime_lib_cache_rebuilds_when_lib_is_missing() {
        let dir = unique_runtime_cache_test_dir("lib_missing");
        let runtime_obj = dir.join("anadil_runtime.obj");
        let runtime_lib = dir.join("anadil_runtime.lib");
        fs::write(&runtime_obj, "test object placeholder").expect("runtime obj should be written");
        let _ = fs::remove_file(&runtime_lib);

        assert!(runtime_lib_needs_rebuild(&runtime_obj, &runtime_lib)
            .expect("runtime lib cache state should be checked"));
    }

    #[test]
    fn packaged_runtime_lib_is_resolved_next_to_executable() {
        let exe = PathBuf::from(r"C:\Anadil\anadil.exe");
        let runtime_lib =
            packaged_runtime_lib_path_from_exe(&exe).expect("exe parent should resolve");

        assert_eq!(
            runtime_lib,
            PathBuf::from(r"C:\Anadil\runtime\anadil_runtime.lib")
        );
    }

    #[test]
    fn build_tools_missing_message_guides_user_to_download_and_interpret() {
        let message = super::build_tools_missing_message();

        assert!(message.contains("Visual Studio Build Tools bulunamadi."));
        assert!(message.contains("https://visualstudio.microsoft.com/visual-cpp-build-tools/"));
        assert!(message.contains("anadil yorumla <dosya>.ana"));
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
    fn accepts_json_interpret_form() {
        let args = vec![
            "anadil".to_string(),
            "yorumla".to_string(),
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

        assert!(matches!(command, Command::Interpret));
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
    fn accepts_ir_form() {
        let args = vec![
            "anadil".to_string(),
            "ir".to_string(),
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

        assert!(matches!(command, Command::Ir));
        assert_eq!(path, "examples/topla.ana");
        assert_eq!(output, super::OutputFormat::Text);
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
