# Anadil V0.1 Test Gap Analizi

Bu belge native compiler V0.1 icin interpreter/native parity ve runtime
edge case testlerinde eksik kalan senaryolari onceliklendirilmis halde
toplar. `Docs/test_coverage.md` icindeki kapsam matrisinin actionable
ozeti olarak hazirlanmistir.

Yontem: `Docs/project_status.md`, `Docs/native_compiler.md`,
`tests/native_examples.rs`, `tests/native_edge_cases.rs` ve
`tests/cli_diagnostics.rs` mevcut kapsam ile dil yuzeyinin (lexer,
parser, sema, native backend, runtime) karsilastirilmasi sonucunda elde
edilen sonuclardir.

## Oncelik Tanimlari

- **P0** — V0.1 kapatmadan once yapilmasi sart.
- **P1** — V0.1 kapatma engeli degil ama eklenmesi tavsiye edilir.
- **P2** — Sonraki surumlere ertelenebilir.

## P0 — V0.1 Kapatmadan Once Sart

Durum: Tamamlandi. Asagidaki testler `tests/native_edge_cases.rs`
icinde eklendi ve `cargo test --test native_edge_cases` ile gecti.

| Test adi | Neyi korur | Dosya |
|---|---|---|
| `native_void_function_matches_interpreter` | Donus tipsiz (`-> tip` yok) fonksiyon native codegen ve stack frame | `tests/native_edge_cases.rs` |
| `native_recursive_function_matches_interpreter` | Recursive call stack frame yonetimi (orn. faktoriyel veya kucuk Fibonacci) | `tests/native_edge_cases.rs` |
| `native_boolean_equality_matches_interpreter` | `mantik == mantik` ve `!=` sema karari + native codegen | `tests/native_edge_cases.rs` |

## P1 — Iyi Olur

Durum: Tamamlandi. Asagidaki testler `tests/native_edge_cases.rs`,
`tests/examples.rs` ve `tests/cli_diagnostics.rs` icinde eklendi ve gecti.

| Test adi | Neyi korur | Dosya |
|---|---|---|
| ~~`native_six_parameter_function_matches_interpreter`~~ | 4 register + 2 stack arg sinir durumu (5 ve 7 mevcut, 6 atlanmis) | `tests/native_edge_cases.rs` |
| ~~`native_nested_loop_break_scope_matches_interpreter`~~ | Ic ice donguda `kir` sadece ic donguyu kirar; `devam` dogru iterasyona doner | `tests/native_edge_cases.rs` |
| ~~`rejects_break_outside_loop_example`~~ | `kir`/`devam` dongu disinda sema hatasi | `tests/examples.rs` + `examples/hata_kir_disarida.ana` |
| ~~`rejects_missing_return_example`~~ | Donus tipli fonksiyonun bazi dallarinda `don` eksikligi sema hatasi | `tests/examples.rs` + `examples/hata_donus_eksik.ana` |
| ~~`rejects_mixed_type_comparison_example`~~ | `sayi == metin` gibi karisik tip karsilastirma sema hatasi | `tests/examples.rs` + `examples/hata_karisik_karsilastirma.ana` |
| ~~`rejects_yazdir_value_use_example`~~ | `yazdir` sonucunu deger gibi kullanma sema hatasi (`x = yazdir(10)`) | `tests/examples.rs` + `examples/hata_yazdir_deger.ana` |
| ~~`cli_rejects_missing_source_file`~~ | Var olmayan kaynak dosya cagirisinda diagnostic ve exit kodu | `tests/cli_diagnostics.rs` |

## P2 — Sonra

| Test adi | Neyi korur | Dosya |
|---|---|---|
| `native_triple_nested_if_matches_interpreter` | 3 derinlik `eger`/`degilse` dogrulugu | `tests/native_edge_cases.rs` |
| `native_yazdir_alias_mixed_with_turkce` | Ayni program icinde `yazdir` ve `yazdir` (Turkce) ardisik cagri | `tests/native_edge_cases.rs` |
| `native_handles_long_onedrive_style_path` | Windows `MAX_PATH` (260+) yakin uzun yollar | `tests/native_edge_cases.rs` |
| `native_runtime_integer_overflow_behavior` | `sayi` toplama/carpma tasma davranisi (interpreter ile parity dahil) | `tests/native_edge_cases.rs` |
| `cli_rejects_invalid_extension` | `.ana` olmayan dosyaya `derle` cagrisinda diagnostic | `tests/cli_diagnostics.rs` |

## Notlar

- P0'daki uc madde tamamlandi; `Docs/todo.md` "Test Bosluklari"
  basligi altinda isaretlendi.
- P1 sema reddedisleri icin `examples/hata_*.ana` dosyalari eklendi;
  mevcut `hata_tip.ana` ve `hata_ana_yok.ana` kalibini izlerler.
- P2 maddeleri V0.1 kapatma kriterine girmez; ileride `Docs/todo.md`
  "Sonra" basligina alinabilir.

## Toplam Yuk

- 3 P0 + tum P1 maddeleri tamamlandi; kalan yalnizca 5 P2 testtir.
- P1 borcu V0.2 baslangicinda kapatildi.
- P2'ler kademeli.

## Ilgili Belgeler

- `Docs/test_coverage.md` — V0.1 kapsam matrisi (tum dil yuzeyi).
- `Docs/project_status.md` — V0.1 Tamam Kriterleri.
- `Docs/todo.md` — Test Bosluklari basligi.
- `Docs/native_compiler.md` — Test Stratejisi basligi.
