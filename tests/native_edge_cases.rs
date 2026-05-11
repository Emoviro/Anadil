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
    yazdır(Topla4(1, 2, 3, 4));\n\
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
    yazdır(a == \"Merhaba\");\n\
    yazdır(a != b);\n\
}\n",
    );
}

#[test]
fn native_string_concat_matches_interpreter() {
    assert_native_output(
        "string_concat",
        "\
Ana() {\n\
    mesaj: metin = \"Merhaba\" + \" \" + \"Anadil\";\n\
    bos: metin = \"\" + \"\";\n\
    yazdir(mesaj);\n\
    yazdir(bos == \"\");\n\
}\n",
    );
}

#[test]
fn native_string_length_matches_interpreter() {
    assert_native_output(
        "string_length",
        "\
Ana() {\n\
    yazdir(uzunluk(\"Merhaba\"));\n\
    yazdir(uzunluk(\"A\" + \"B\"));\n\
}\n",
    );
}

#[test]
fn native_chained_string_concat_matches_interpreter() {
    assert_native_output(
        "string_concat_chained",
        "\
Uret() -> metin {\n\
    d\u{00f6}n \" C\" + \" D\";\n\
}\n\
\n\
Ana() {\n\
    mesaj: metin = \"A\" + \" B\" + Uret();\n\
    yazdir(mesaj);\n\
}\n",
    );
}

#[test]
fn native_owned_string_expression_cleanup_matches_interpreter() {
    assert_native_output(
        "owned_string_expression_cleanup",
        "\
Uret() -> metin {\n\
    d\u{00f6}n \"C\" + \"D\";\n\
}\n\
\n\
Ana() {\n\
    yazdir(\"A\" + \"B\");\n\
    Uret();\n\
}\n",
    );
}

#[test]
fn native_string_assignment_replacement_matches_interpreter() {
    assert_native_output(
        "string_assignment_replace",
        "\
Ana() {\n\
    mesaj: metin = \"Eski\";\n\
    mesaj = \"Yeni\" + \" Deger\";\n\
    yazdir(mesaj);\n\
}\n",
    );
}

#[test]
fn native_string_local_sharing_matches_interpreter() {
    assert_native_output(
        "string_local_share",
        "\
Ana() {\n\
    a: metin = \"Merhaba\" + \"!\";\n\
    b: metin = a;\n\
    b = a;\n\
    yazdir(a);\n\
    yazdir(b);\n\
}\n",
    );
}

#[test]
fn native_string_parameter_retain_matches_interpreter() {
    assert_native_output(
        "string_param_retain",
        "\
Selamla(ad: metin) {\n\
    yazdir(ad);\n\
}\n\
\n\
Ana() {\n\
    mesaj: metin = \"Merhaba\" + \"!\";\n\
    Selamla(mesaj);\n\
    yazdir(mesaj);\n\
}\n",
    );
}

#[test]
fn native_inline_owned_string_arguments_match_interpreter() {
    assert_native_output(
        "inline_owned_string_args",
        "\
Selamla(ad: metin) {\n\
    yazdir(ad);\n\
}\n\
\n\
Uret() -> metin {\n\
    d\u{00f6}n \"U\" + \"ret\";\n\
}\n\
\n\
Ana() {\n\
    Selamla(\"Merhaba\" + \"!\");\n\
    Selamla(Uret());\n\
}\n",
    );
}

#[test]
fn native_string_return_ownership_matches_interpreter() {
    assert_native_output(
        "string_return_ownership",
        "\
Uret() -> metin {\n\
    sonuc: metin = \"Merhaba\" + \"!\";\n\
    d\u{00f6}n sonuc;\n\
}\n\
\n\
Ana() {\n\
    mesaj: metin = Uret();\n\
    yazdir(mesaj);\n\
}\n",
    );
}

#[test]
fn native_if_branch_string_cleanup_matches_interpreter() {
    assert_native_output(
        "if_branch_string_cleanup",
        "\
Ana() {\n\
    kosul: mant\u{0131}k = do\u{011f}ru;\n\
    e\u{011f}er (kosul) {\n\
        mesaj: metin = \"Merhaba\" + \"!\";\n\
        yazdir(mesaj);\n\
    } de\u{011f}ilse {\n\
        diger: metin = \"Selam\" + \"!\";\n\
        yazdir(diger);\n\
    }\n\
}\n",
    );
}

#[test]
fn native_loop_string_cleanup_with_break_continue_matches_interpreter() {
    assert_native_output(
        "loop_string_cleanup_break_continue",
        "\
Ana() {\n\
    d\u{00f6}ng\u{00fc} (i: say\u{0131} = 0; i < 4; i = i + 1) {\n\
        mesaj: metin = \"Merhaba\" + \"!\";\n\
        e\u{011f}er (i == 1) {\n\
            devam;\n\
        }\n\
        e\u{011f}er (i == 3) {\n\
            k\u{0131}r;\n\
        }\n\
        yazdir(mesaj);\n\
    }\n\
}\n",
    );
}

#[test]
fn native_void_function_matches_interpreter() {
    assert_native_output(
        "void_function",
        "\
Selamla(ad: metin) {\n\
    yazdir(\"Merhaba\");\n\
    yazdir(ad);\n\
}\n\
\n\
Ana() {\n\
    Selamla(\"Anadil\");\n\
    yazdir(42);\n\
}\n",
    );
}

#[test]
fn native_recursive_function_matches_interpreter() {
    assert_native_output(
        "recursive_function",
        "\
Faktoriyel(n: say\u{0131}) -> say\u{0131} {\n\
    e\u{011f}er (n <= 1) {\n\
        d\u{00f6}n 1;\n\
    }\n\
    d\u{00f6}n n * Faktoriyel(n - 1);\n\
}\n\
\n\
Ana() {\n\
    yazdir(Faktoriyel(5));\n\
}\n",
    );
}

#[test]
fn native_boolean_equality_matches_interpreter() {
    assert_native_output(
        "boolean_equality",
        "\
Ana() {\n\
    a: mant\u{0131}k = do\u{011f}ru;\n\
    b: mant\u{0131}k = yanl\u{0131}\u{015f};\n\
    yazdir(a == do\u{011f}ru);\n\
    yazdir(a != b);\n\
    yazdir((1 < 2) == do\u{011f}ru);\n\
    yazdir((2 < 1) != b);\n\
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
    yazdır(toplam);\n\
}\n",
    );
}

#[test]
fn native_nested_loop_break_scope_matches_interpreter() {
    assert_native_output(
        "nested_loop_break_scope",
        "\
Ana() {\n\
    toplam: say\u{0131} = 0;\n\
    d\u{00f6}ng\u{00fc} (i: say\u{0131} = 0; i < 3; i = i + 1) {\n\
        d\u{00f6}ng\u{00fc} (j: say\u{0131} = 0; j < 4; j = j + 1) {\n\
            e\u{011f}er (j == 1) {\n\
                devam;\n\
            }\n\
            e\u{011f}er (j == 3) {\n\
                k\u{0131}r;\n\
            }\n\
            toplam = toplam + i * 10 + j;\n\
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
        yazdır(x);\n\
    }\n\
    yazdır(x);\n\
}\n",
    );
}

#[test]
fn native_five_parameter_function_matches_interpreter() {
    assert_native_output(
        "five_params",
        "\
Besli(a: say\u{0131}, b: say\u{0131}, c: say\u{0131}, d: say\u{0131}, e: say\u{0131}) -> say\u{0131} {\n\
    d\u{00f6}n a + b + c + d + e;\n\
}\n\
\n\
Ana() {\n\
    yazdır(Besli(1, 2, 3, 4, 5));\n\
}\n",
    );
}

#[test]
fn native_six_parameter_function_matches_interpreter() {
    assert_native_output(
        "six_params",
        "\
Altili(a: say\u{0131}, b: say\u{0131}, c: say\u{0131}, d: say\u{0131}, e: say\u{0131}, f: say\u{0131}) -> say\u{0131} {\n\
    d\u{00f6}n a + b + c + d + e + f;\n\
}\n\
\n\
Ana() {\n\
    yazdir(Altili(1, 2, 3, 4, 5, 6));\n\
}\n",
    );
}

#[test]
fn native_seven_parameter_function_matches_interpreter() {
    assert_native_output(
        "seven_params",
        "\
Topla7(a: say\u{0131}, b: say\u{0131}, c: say\u{0131}, d: say\u{0131}, e: say\u{0131}, f: say\u{0131}, g: say\u{0131}) -> say\u{0131} {\n\
    d\u{00f6}n a + b + c + d + e + f + g;\n\
}\n\
\n\
Ana() {\n\
    yazdır(Topla7(1, 2, 3, 4, 5, 6, 7));\n\
}\n",
    );
}

#[test]
fn native_nested_calls_preserve_arguments_and_order() {
    assert_native_output(
        "nested_calls_preserve_args",
        "\
Etiket(x: say\u{0131}) -> say\u{0131} {\n\
    yazdır(x);\n\
    d\u{00f6}n x;\n\
}\n\
\n\
Topla3(a: say\u{0131}, b: say\u{0131}, c: say\u{0131}) -> say\u{0131} {\n\
    d\u{00f6}n a * 100 + b * 10 + c;\n\
}\n\
\n\
Ana() {\n\
    yazdır(Topla3(Etiket(1), Etiket(2), Etiket(3)));\n\
    yazdır(10 + Etiket(4));\n\
}\n",
    );
}

#[test]
fn native_numeric_comparisons_match_interpreter() {
    assert_native_output(
        "numeric_comparisons",
        "\
Ana() {\n\
    yazdır(1 < 2);\n\
    yazdır(2 < 1);\n\
    yazdır(2 <= 2);\n\
    yazdır(3 > 2);\n\
    yazdır(2 >= 3);\n\
    yazdır(4 == 4);\n\
    yazdır(4 != 5);\n\
}\n",
    );
}

#[test]
fn native_runtime_prints_integer_edge_values() {
    assert_native_output(
        "runtime_integer_edges",
        "\
Ana() {\n\
    yazdir(0);\n\
    yazdir(-1);\n\
    yazdir(123456789012345678);\n\
    yazdir(-123456789012345678);\n\
}\n",
    );
}

#[test]
fn native_runtime_prints_empty_and_utf8_strings() {
    assert_native_output(
        "runtime_string_edges",
        "\
Ana() {\n\
    yazdir(\"ilk\");\n\
    yazdir(\"\");\n\
    yazdir(\"Merhaba, d\u{00fc}nya\");\n\
    yazdir(\"ayn\u{0131}\" == \"ayn\u{0131}\");\n\
    yazdir(\"a\" != \"\");\n\
}\n",
    );
}

#[test]
fn native_build_handles_spaces_and_turkish_paths() {
    let Some(anadil_bin) = anadil_binary() else {
        eprintln!("native path case skipped: anadil binary path is not available");
        return;
    };

    let source = "\
Ana() {\n\
    yazdir(\"path tamam\");\n\
    yazdir(42);\n\
}\n";
    let expected = run_source(source).expect("source should run with interpreter");
    let source_path = PathBuf::from("target")
        .join("native path cases")
        .join("T\u{00fc}rk\u{00e7}e Klas\u{00f6}r")
        .join("deneme dosyas\u{0131}.ana");
    let parent = source_path
        .parent()
        .expect("path case source path should have a parent");
    fs::create_dir_all(parent).expect("path case directory should be created");
    fs::write(&source_path, source).expect("path case source should be written");

    let compile_output = Command::new(&anadil_bin)
        .arg("derle")
        .arg(&source_path)
        .output()
        .expect("native compile command should run");

    if !compile_output.status.success() && native_toolchain_missing(&compile_output) {
        eprintln!("native path case skipped: Visual Studio native toolchain is not available");
        return;
    }

    assert!(
        compile_output.status.success(),
        "native compile failed for path case\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&compile_output.stdout),
        String::from_utf8_lossy(&compile_output.stderr)
    );

    let exe_path = source_path.with_extension("exe");
    assert!(
        exe_path.is_file(),
        "native compile should create exe at `{}`",
        exe_path.display()
    );

    let run_output = Command::new(&exe_path)
        .output()
        .expect("native executable should run");

    assert!(
        run_output.status.success(),
        "native executable failed for path case\nstderr:\n{}",
        String::from_utf8_lossy(&run_output.stderr)
    );

    let actual = normalize_output(&run_output.stdout);
    assert_eq!(actual, expected, "native output differs for path case");
}

#[test]
fn native_division_by_zero_reports_runtime_error() {
    let Some(anadil_bin) = anadil_binary() else {
        eprintln!("native edge case skipped: anadil binary path is not available");
        return;
    };

    let source = "\
Ana() {\n\
    yazdır(10 / 0);\n\
}\n";

    let interpreter_error = run_source(source).expect_err("interpreter should reject zero divide");
    assert!(interpreter_error.contains("Sifira bolme"));

    let compile_output = compile_source_with_native(&anadil_bin, "division_by_zero", source);
    if !compile_output.status.success() && native_toolchain_missing(&compile_output) {
        eprintln!("native edge case skipped: Visual Studio native toolchain is not available");
        return;
    }

    assert!(
        compile_output.status.success(),
        "native compile failed for division_by_zero\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&compile_output.stdout),
        String::from_utf8_lossy(&compile_output.stderr)
    );

    let exe_path = edge_case_path("division_by_zero").with_extension("exe");
    let run_output = Command::new(&exe_path)
        .output()
        .expect("native executable should run");

    assert!(
        !run_output.status.success(),
        "native executable should fail for division_by_zero"
    );

    let combined_output = format!(
        "{}{}",
        String::from_utf8_lossy(&run_output.stdout),
        String::from_utf8_lossy(&run_output.stderr)
    );
    assert!(combined_output.contains("Anadil runtime hatasi: Sifira bolme hatasi"));
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
