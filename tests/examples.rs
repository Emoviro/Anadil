use anadil::{compile_source, run_source};

fn assert_example_output(source: &str, expected: &str) {
    let output = run_source(source).expect("example should run");
    assert_eq!(output, expected);
}

#[test]
fn runs_topla_example() {
    assert_example_output(include_str!("../examples/topla.ana"), "30");
}

#[test]
fn runs_dongu_example() {
    assert_example_output(include_str!("../examples/dongu.ana"), "0\n2\n3");
}

#[test]
fn runs_kosul_example() {
    assert_example_output(include_str!("../examples/kosul.ana"), "15");
}

#[test]
fn runs_fonksiyon_example() {
    assert_example_output(include_str!("../examples/fonksiyon.ana"), "25");
}

#[test]
fn runs_mantik_example() {
    assert_example_output(
        include_str!("../examples/mantik.ana"),
        "doğru\nyanlış\ndoğru\nyanlış",
    );
}

#[test]
fn runs_kosullu_dongu_example() {
    assert_example_output(include_str!("../examples/kosullu_dongu.ana"), "0\n1\n2");
}

#[test]
fn runs_sonsuz_dongu_example() {
    assert_example_output(include_str!("../examples/sonsuz_dongu.ana"), "0\n1\n2");
}

#[test]
fn runs_kapsam_example() {
    assert_example_output(include_str!("../examples/kapsam.ana"), "2\n1");
}

#[test]
fn runs_negatif_example() {
    assert_example_output(include_str!("../examples/negatif.ana"), "-10\n7\n10");
}

#[test]
fn rejects_hata_tip_example() {
    let error = compile_source(include_str!("../examples/hata_tip.ana"))
        .expect_err("type error example should fail");

    assert!(error.contains("Semantic hata"));
    assert!(error.contains("sayı"));
    assert!(error.contains("mantık"));
}

#[test]
fn rejects_hata_ana_yok_example() {
    let error = compile_source(include_str!("../examples/hata_ana_yok.ana"))
        .expect_err("missing Ana example should fail");

    assert!(error.contains("Semantic hata"));
    assert!(error.contains("Ana()"));
}
