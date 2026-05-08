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

fn anadil_binary() -> Option<PathBuf> {
    option_env!("CARGO_BIN_EXE_anadil").map(PathBuf::from)
}
