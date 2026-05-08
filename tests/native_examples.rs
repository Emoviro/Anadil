use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anadil::run_source;

const EXAMPLES: &[&str] = &[
    "topla",
    "negatif",
    "kosul",
    "fonksiyon",
    "mantik",
    "metin",
    "kosullu_dongu",
    "dongu",
    "sonsuz_dongu",
    "kapsam",
    "native_mvp",
];

#[test]
fn native_examples_match_interpreter_output() {
    let Some(anadil_bin) = anadil_binary() else {
        eprintln!("native examples skipped: anadil binary path is not available");
        return;
    };

    for example in EXAMPLES {
        assert_native_example_matches_interpreter(&anadil_bin, example);
    }
}

fn assert_native_example_matches_interpreter(anadil_bin: &Path, example: &str) {
    let source_path = PathBuf::from("examples").join(format!("{example}.ana"));
    let source = fs::read_to_string(&source_path).expect("example source should be readable");
    let expected = run_source(&source).expect("example should run with interpreter");

    let work_dir = PathBuf::from("target").join("native_examples");
    fs::create_dir_all(&work_dir).expect("native example output directory should be created");

    let copied_source = work_dir.join(format!("{example}.ana"));
    fs::copy(&source_path, &copied_source).expect("example source should be copied");

    let compile_output = Command::new(anadil_bin)
        .arg("derle")
        .arg(&copied_source)
        .output()
        .expect("native compile command should run");

    if !compile_output.status.success() && native_toolchain_missing(&compile_output) {
        eprintln!("native examples skipped: Visual Studio native toolchain is not available");
        return;
    }

    assert!(
        compile_output.status.success(),
        "native compile failed for {example}\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&compile_output.stdout),
        String::from_utf8_lossy(&compile_output.stderr)
    );

    let exe_path = copied_source.with_extension("exe");
    let run_output = Command::new(&exe_path)
        .output()
        .expect("native executable should run");

    assert!(
        run_output.status.success(),
        "native executable failed for {example}\nstderr:\n{}",
        String::from_utf8_lossy(&run_output.stderr)
    );

    let actual = normalize_output(&run_output.stdout);
    assert_eq!(actual, expected, "native output differs for {example}");
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
