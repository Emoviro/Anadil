use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anadil::run_source;

#[test]
fn native_four_parameter_function_matches_interpreter() {
    assert_native_output(
        "four_params",
        "\
Topla4(a: say\u{0131}, b: say\u{0131}, c: say\u{0131}, d: say\u{0131}) -> say\u{0131} {\n\
    d\u{00f6}n a + b + c + d;\n\
}\n\
\n\
Ana() {\n\
    yazdir(Topla4(1, 2, 3, 4));\n\
}\n",
    );
}

#[test]
fn native_string_equality_and_inequality_match_interpreter() {
    assert_native_output(
        "string_compare",
        "\
Ana() {\n\
    a: metin = \"Merhaba\";\n\
    b: metin = \"Dunya\";\n\
    yazdir(a == \"Merhaba\");\n\
    yazdir(a != b);\n\
}\n",
    );
}

#[test]
fn native_nested_if_loop_matches_interpreter() {
    assert_native_output(
        "nested_if_loop",
        "\
Ana() {\n\
    toplam: say\u{0131} = 0;\n\
    d\u{00f6}ng\u{00fc} (i: say\u{0131} = 0; i < 6; i = i + 1) {\n\
        e\u{011f}er (i == 1) {\n\
            devam;\n\
        } de\u{011f}ilse {\n\
            e\u{011f}er (i == 5) {\n\
                k\u{0131}r;\n\
            }\n\
            toplam = toplam + i;\n\
        }\n\
    }\n\
    yazdir(toplam);\n\
}\n",
    );
}

#[test]
fn native_scope_shadowing_matches_interpreter() {
    assert_native_output(
        "scope_shadowing",
        "\
Ana() {\n\
    x: say\u{0131} = 7;\n\
    e\u{011f}er (do\u{011f}ru) {\n\
        x: say\u{0131} = 11;\n\
        yazdir(x);\n\
    }\n\
    yazdir(x);\n\
}\n",
    );
}

#[test]
fn native_rejects_more_than_four_function_parameters() {
    let Some(anadil_bin) = anadil_binary() else {
        eprintln!("native edge case skipped: anadil binary path is not available");
        return;
    };

    let source = "\
Besli(a: say\u{0131}, b: say\u{0131}, c: say\u{0131}, d: say\u{0131}, e: say\u{0131}) -> say\u{0131} {\n\
    d\u{00f6}n a + b + c + d + e;\n\
}\n\
\n\
Ana() {\n\
    yazdir(Besli(1, 2, 3, 4, 5));\n\
}\n";

    let output = compile_source_with_native(&anadil_bin, "five_params", source);

    if !output.status.success() && native_toolchain_missing(&output) {
        eprintln!("native edge case skipped: Visual Studio native toolchain is not available");
        return;
    }

    assert!(
        !output.status.success(),
        "native compile should reject more than 4 parameters"
    );

    let text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        text.contains("en fazla 4"),
        "expected 4-parameter limit error, got:\n{text}"
    );
}

fn assert_native_output(name: &str, source: &str) {
    let Some(anadil_bin) = anadil_binary() else {
        eprintln!("native edge case skipped: anadil binary path is not available");
        return;
    };

    let expected = run_source(source).expect("source should run with interpreter");
    let compile_output = compile_source_with_native(&anadil_bin, name, source);

    if !compile_output.status.success() && native_toolchain_missing(&compile_output) {
        eprintln!("native edge case skipped: Visual Studio native toolchain is not available");
        return;
    }

    assert!(
        compile_output.status.success(),
        "native compile failed for {name}\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&compile_output.stdout),
        String::from_utf8_lossy(&compile_output.stderr)
    );

    let exe_path = edge_case_path(name).with_extension("exe");
    let run_output = Command::new(&exe_path)
        .output()
        .expect("native executable should run");

    assert!(
        run_output.status.success(),
        "native executable failed for {name}\nstderr:\n{}",
        String::from_utf8_lossy(&run_output.stderr)
    );

    let actual = normalize_output(&run_output.stdout);
    assert_eq!(actual, expected, "native output differs for {name}");
}

fn compile_source_with_native(anadil_bin: &Path, name: &str, source: &str) -> std::process::Output {
    let source_path = edge_case_path(name);
    let parent = source_path
        .parent()
        .expect("edge case source path should have a parent");
    fs::create_dir_all(parent).expect("edge case output directory should be created");
    fs::write(&source_path, source).expect("edge case source should be written");

    Command::new(anadil_bin)
        .arg("derle")
        .arg(&source_path)
        .output()
        .expect("native compile command should run")
}

fn edge_case_path(name: &str) -> PathBuf {
    PathBuf::from("target")
        .join("native_edge_cases")
        .join(format!("{name}.ana"))
}

fn anadil_binary() -> Option<PathBuf> {
    option_env!("CARGO_BIN_EXE_anadil").map(PathBuf::from)
}

fn native_toolchain_missing(output: &std::process::Output) -> bool {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    stdout.contains("Visual Studio Build Tools")
        || stderr.contains("Visual Studio Build Tools")
        || stdout.contains("vcvars64.bat")
        || stderr.contains("vcvars64.bat")
}

fn normalize_output(output: &[u8]) -> String {
    String::from_utf8_lossy(output)
        .replace("\r\n", "\n")
        .trim_end_matches('\n')
        .to_string()
}
