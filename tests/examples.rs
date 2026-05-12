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
fn runs_metin_example() {
    assert_example_output(
        include_str!("../examples/metin.ana"),
        "Merhaba Anadil\nYerel Derleyici\ndo\u{011f}ru",
    );
}

#[test]
fn runs_metin_v02_example() {
    assert_example_output(
        include_str!("../examples/metin_v02.ana"),
        "Merhaba Anadil\n14\n0\n2\n15\ndo\u{011f}ru",
    );
}

#[test]
fn runs_dizi_v03_example() {
    assert_example_output(include_str!("../examples/dizi_v03.ana"), "3\n1\niki");
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
fn runs_native_mvp_example() {
    assert_example_output(
        include_str!("../examples/native_mvp.ana"),
        "7\n15\ndo\u{011f}ru\nnative",
    );
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

#[test]
fn rejects_hata_kir_disarida_example() {
    let error = compile_source(include_str!("../examples/hata_kir_disarida.ana"))
        .expect_err("break outside loop example should fail");

    assert!(error.contains("Semantic hata"));
    assert!(error.contains("döngü") || error.contains("dÃ¶ngÃ¼"));
}

#[test]
fn rejects_hata_donus_eksik_example() {
    let error = compile_source(include_str!("../examples/hata_donus_eksik.ana"))
        .expect_err("missing return example should fail");

    assert!(error.contains("Semantic hata"));
    assert!(error.contains("kontrol yollar"));
}

#[test]
fn rejects_hata_karisik_karsilastirma_example() {
    let error = compile_source(include_str!("../examples/hata_karisik_karsilastirma.ana"))
        .expect_err("mixed type comparison example should fail");

    assert!(error.contains("Semantic hata"));
    assert!(error.contains("karşılaştırılamaz") || error.contains("karÅŸÄ±laÅŸtÄ±rÄ±lamaz"));
}

#[test]
fn rejects_hata_yazdir_deger_example() {
    let error = compile_source(include_str!("../examples/hata_yazdir_deger.ana"))
        .expect_err("using yazdir as a value should fail");

    assert!(error.contains("Semantic hata"));
    assert!(error.contains("değer üretmeli") || error.contains("deÄŸer Ã¼retmeli"));
}
