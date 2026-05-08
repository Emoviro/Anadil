use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn check_command_prints_ide_friendly_diagnostic() {
    let Some(anadil_bin) = anadil_binary() else {
        eprintln!("cli diagnostic test skipped: anadil binary path is not available");
        return;
    };

    let source_path = PathBuf::from("target")
        .join("cli_diagnostics")
        .join("tip_hatasi.ana");
    let parent = source_path
        .parent()
        .expect("diagnostic fixture path should have a parent");
    fs::create_dir_all(parent).expect("diagnostic fixture directory should be created");
    fs::write(
        &source_path,
        "\
Ana() {\n\
    x: say\u{0131} = do\u{011f}ru;\n\
}\n",
    )
    .expect("diagnostic fixture should be written");

    let output = Command::new(anadil_bin)
        .arg("kontrol")
        .arg(&source_path)
        .output()
        .expect("check command should run");

    assert!(!output.status.success(), "check command should fail");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Semantic hata"));
    assert!(stderr.contains("sat"));
    assert!(stderr.contains("sut") || stderr.contains("süt"));
    assert!(stderr.contains("2 | x:"));
    assert!(stderr.contains("^"));
}

#[test]
fn check_json_reports_success() {
    let Some(anadil_bin) = anadil_binary() else {
        eprintln!("cli diagnostic test skipped: anadil binary path is not available");
        return;
    };

    let source_path = write_fixture(
        "valid.ana",
        "\
Ana() {\n\
    yazdir(10);\n\
}\n",
    );

    let output = Command::new(anadil_bin)
        .arg("kontrol")
        .arg("--json")
        .arg(&source_path)
        .output()
        .expect("check command should run");

    assert!(output.status.success(), "json check command should pass");
    assert!(
        output.stderr.is_empty(),
        "json success should not write stderr"
    );

    let stdout = normalize_stdout(&output.stdout);
    assert_eq!(stdout, "{\"ok\":true,\"diagnostics\":[]}");
}

#[test]
fn check_json_reports_structured_error() {
    let Some(anadil_bin) = anadil_binary() else {
        eprintln!("cli diagnostic test skipped: anadil binary path is not available");
        return;
    };

    let source_path = write_fixture(
        "json_tip_hatasi.ana",
        "\
Ana() {\n\
    x: say\u{0131} = do\u{011f}ru;\n\
}\n",
    );

    let output = Command::new(anadil_bin)
        .arg("kontrol")
        .arg("--json")
        .arg(&source_path)
        .output()
        .expect("check command should run");

    assert!(!output.status.success(), "json check command should fail");
    assert!(
        output.stderr.is_empty(),
        "json failure should not write stderr"
    );

    let stdout = normalize_stdout(&output.stdout);
    assert!(stdout.contains("\"ok\":false"));
    assert!(stdout.contains("\"severity\":\"error\""));
    assert!(stdout.contains("\"stage\":\"semantic\""));
    assert!(stdout.contains("\"line\":2"));
    assert!(stdout.contains("\"column\":1"));
    assert!(stdout.contains("\\\"x\\\"") || stdout.contains("`x`"));
}

#[test]
fn run_json_reports_output() {
    let Some(anadil_bin) = anadil_binary() else {
        eprintln!("cli diagnostic test skipped: anadil binary path is not available");
        return;
    };

    let source_path = write_fixture(
        "run_valid.ana",
        "\
Ana() {\n\
    yazdir(10);\n\
    yazdir(20);\n\
}\n",
    );

    let output = Command::new(anadil_bin)
        .arg("calistir")
        .arg("--json")
        .arg(&source_path)
        .output()
        .expect("run command should run");

    assert!(output.status.success(), "json run command should pass");
    assert!(
        output.stderr.is_empty(),
        "json run success should not write stderr"
    );

    let stdout = normalize_stdout(&output.stdout);
    assert_eq!(
        stdout,
        "{\"ok\":true,\"output\":\"10\\n20\",\"diagnostics\":[]}"
    );
}

#[test]
fn run_json_reports_runtime_error() {
    let Some(anadil_bin) = anadil_binary() else {
        eprintln!("cli diagnostic test skipped: anadil binary path is not available");
        return;
    };

    let source_path = write_fixture(
        "run_division_by_zero.ana",
        "\
Ana() {\n\
    yazdir(10 / 0);\n\
}\n",
    );

    let output = Command::new(anadil_bin)
        .arg("calistir")
        .arg("--json")
        .arg(&source_path)
        .output()
        .expect("run command should run");

    assert!(!output.status.success(), "json run command should fail");
    assert!(
        output.stderr.is_empty(),
        "json run failure should not write stderr"
    );

    let stdout = normalize_stdout(&output.stdout);
    assert!(stdout.contains("\"ok\":false"));
    assert!(stdout.contains("\"output\":\"\""));
    assert!(stdout.contains("\"severity\":\"error\""));
    assert!(stdout.contains("\"stage\":\"runtime\""));
    assert!(stdout.contains("\"message\":\"Sifira bolme hatasi\""));
    assert!(stdout.contains("\"line\":2"));
}

fn write_fixture(name: &str, source: &str) -> PathBuf {
    let source_path = PathBuf::from("target").join("cli_diagnostics").join(name);
    let parent = source_path
        .parent()
        .expect("diagnostic fixture path should have a parent");
    fs::create_dir_all(parent).expect("diagnostic fixture directory should be created");
    fs::write(&source_path, source).expect("diagnostic fixture should be written");
    source_path
}

fn normalize_stdout(output: &[u8]) -> String {
    String::from_utf8_lossy(output)
        .replace("\r\n", "\n")
        .trim_end_matches('\n')
        .to_string()
}

fn anadil_binary() -> Option<PathBuf> {
    option_env!("CARGO_BIN_EXE_anadil").map(PathBuf::from)
}
