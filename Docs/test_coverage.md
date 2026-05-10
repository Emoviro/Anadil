# Anadil V0.1 Test Kapsam Matrisi

Bu belge V0.1 dil ozelliklerinin interpreter ve native compiler tarafindaki
test kapsamini cikartir, bosluk olan yerleri listeler. `project_status.md`
icindeki V0.1 Tamam Kriterleri "son bosluk tarama gecisi" maddesinin
somut karsiligidir.

## Yontem

Kaynaklar:

- Dil yuzeyi: `Docs/dil_referansi.md` ve `Docs/anadil_v1_spec.txt`.
- Test dosyalari: `tests/examples.rs`, `tests/native_examples.rs`,
  `tests/native_edge_cases.rs`, `tests/cli_diagnostics.rs`.
- Ornek programlar: `examples/*.ana`.

Her dil ozelligi icin "interpreter testi var mi?", "native parity testi
var mi?", "edge case testi var mi?" sorulariyla karsilastirildi.

## Kapsam Matrisi

Sembol acikalamasi: `✓` kapsam mevcut, `·` kapsam yok, `n/a` ozellik
test edilebilir bir yuzey degil.

### Tipler ve literal'ler

| Ozellik | Interpreter | Native parity | Native edge |
|---|---|---|---|
| `sayi` literal | ✓ topla, fonksiyon | ✓ | ✓ runtime_integer_edges |
| Negatif `sayi` (`-10`, `-x`) | ✓ negatif | ✓ | ✓ runtime_integer_edges |
| Buyuk `sayi` (i64 sinirlari) | · | · | ✓ runtime_integer_edges (123456789012345678) |
| `mantik` literal (`dogru`/`yanlis`) | ✓ mantik | ✓ | · |
| `metin` literal | ✓ metin | ✓ | · |
| Bos `metin` `""` | · | · | ✓ runtime_string_edges |
| UTF-8 `metin` | ✓ metin | ✓ | ✓ runtime_string_edges |

### Operatorler

| Ozellik | Interpreter | Native parity | Native edge |
|---|---|---|---|
| `+`, `-`, `*`, `/` (sayi) | ✓ | ✓ | (covered) |
| Unary `-` | ✓ negatif | ✓ | · |
| `==`, `!=` (sayi) | ✓ | ✓ | ✓ numeric_comparisons |
| `<`, `<=`, `>`, `>=` (sayi) | ✓ kosul | ✓ | ✓ numeric_comparisons |
| `==`, `!=` (metin) | · | · | ✓ string_compare, runtime_string_edges |
| `==`, `!=` (mantik) | · | · | ✓ boolean_equality |
| Parantezli karmasik ifade `(a + b) * c` | (implicit) | (implicit) | · |

### Kontrol akisi

| Ozellik | Interpreter | Native parity | Native edge |
|---|---|---|---|
| `eger` tek dal | ✓ kosul | ✓ | · |
| `eger` / `degilse` | ✓ kosul, native_mvp | ✓ | · |
| Ic ice `eger` | · | · | ✓ nested_if_loop |
| Sonsuz dongu (`dongu { }`) | ✓ sonsuz_dongu | ✓ | · |
| Kosullu dongu (`dongu (kosul) { }`) | ✓ kosullu_dongu | ✓ | · |
| Sayacli dongu (`dongu (i: sayi = ...; ...; ...)`) | ✓ dongu, native_mvp | ✓ | ✓ nested_if_loop |
| `kir` | ✓ sonsuz_dongu, dongu | ✓ | ✓ nested_if_loop |
| `devam` | ✓ dongu, native_mvp | ✓ | ✓ nested_if_loop |

### Fonksiyon

| Ozellik | Interpreter | Native parity | Native edge |
|---|---|---|---|
| Argumansiz (`Ana()`) | ✓ | ✓ | (her yerde) |
| 1 parametre | ✓ fonksiyon, native_mvp | ✓ | · |
| 2 parametre | ✓ topla | ✓ | · |
| 3 parametre | · | · | (nested_calls_preserve_args icinde Topla3) |
| 4 parametre (son register arg) | · | · | ✓ four_params |
| 5 parametre (ilk stack arg) | · | · | ✓ five_params |
| 6 parametre | · | · | ✓ six_params |
| 7 parametre | · | · | ✓ seven_params |
| Donus tipli (`-> sayi`) ile `don` | ✓ topla, fonksiyon | ✓ | (her yerde) |
| Donus tipsiz (void) fonksiyon | · | · | ✓ void_function |
| Erken `don` (icli kosul icinden) | ✓ native_mvp PozitifMi | ✓ | · |
| Recursive call | · | · | ✓ recursive_function |
| Nested call (deger arguman olarak fonksiyon cagri) | · | · | ✓ nested_calls_preserve_args |
| Kapsam shadowing (ayni isim, ic kapsam) | ✓ kapsam | ✓ | ✓ scope_shadowing |

### Yerlesik fonksiyon

| Ozellik | Interpreter | Native parity | Native edge |
|---|---|---|---|
| `yazdir(sayi)` | ✓ | ✓ | ✓ runtime_integer_edges |
| `yazdir(mantik)` | ✓ mantik | ✓ | ✓ boolean_equality |
| `yazdir(metin)` | ✓ metin | ✓ | ✓ runtime_string_edges |
| ASCII alias `yazdir` | · | · | ✓ runtime_integer/string_edges |

### Diagnostics (compile-time hatalar)

| Ozellik | Test |
|---|---|
| `Ana()` eksikligi | ✓ examples.rs `rejects_hata_ana_yok_example` |
| Tip uyumsuzlugu | ✓ examples.rs `rejects_hata_tip_example` |
| Tanimsiz degisken | · |
| Tanimsiz fonksiyon | · |
| Yanlis arg sayisi | · |
| Yanlis arg tipi | · |
| `kir`/`devam` dongu disinda | · |
| `yazdir` sonucunu deger gibi kullanma | · |
| Donus tipli fonksiyonun tum dallarinda donus eksik | · |
| `Ana()` parametre alma | · |
| `Ana()` donus tipi belirtme | · |

### Diagnostics (runtime hatalar)

| Ozellik | Test |
|---|---|
| Sifira bolme | ✓ native_edge_cases `division_by_zero` |

### Path / Windows dayanikliligi

| Ozellik | Test |
|---|---|
| Bosluklu yol | ✓ native_edge_cases `spaces_and_turkish_paths` |
| Turkce karakterli yol | ✓ native_edge_cases `spaces_and_turkish_paths` |
| OneDrive/Masaustu uzun yol | (manuel test, doc'ta dusulmus) |

### CLI yuzeyi

| Komut | Test |
|---|---|
| `kontrol` | ✓ cli_diagnostics |
| `kontrol --json` | ✓ cli_diagnostics |
| `calistir` | ✓ cli_diagnostics native derle-ve-calistir |
| `calistir --json` | ✓ cli_diagnostics native derle-ve-calistir |
| `yorumla --json` | ✓ cli_diagnostics interpreter/debug |
| `derle` | ✓ native_examples, native_edge_cases |
| `derle --json` | ✓ cli_diagnostics |
| `asm`, `asm-yaz`, `ast`, `typed`, `ornekler`, `surum`, `yardim`, `repl`, `ide` | · |

## Bulunan Bosluklar

Asagidakiler V0.1'in dil yuzeyinde mevcut ama test edilmeyen davranislardir.
Onceliklendirme acidan onemli (yuksek riski olanlar uste).

### Yuksek oncelik (V0.1 kapatilmadan duzeltilmesi onerilir)

Durum: Tamamlandi. Bu dort madde `tests/native_edge_cases.rs` icinde
inline parity testleriyle kapatildi.

1. ~~**Donus tipsiz (void) fonksiyon**~~: `YazdirDeger(x: sayi) { yazdir(x) }`
   gibi `-> tip` belirtmeyen fonksiyonlar dilde mevcut ancak test yok.
   Native backend stack ayarinin dogru oldugu test edilmemis.

2. ~~**Recursive fonksiyon**~~: Faktoriyel veya basit recursive Fibonacci
   ornegi yok. Native backend stack frame ve recursion derinligi
   davranisini test etmiyor.

3. ~~**6 parametreli fonksiyon**~~: 4 register + 2 stack arg sinir durumu;
   five_params ve seven_params var ama 6 atlanmis. Stack alignment
   regresyon riskini kapatir.

4. ~~**`mantik` esitlik (`==`, `!=`)**~~: `dogru == dogru`, `yanlis != dogru`
   parser/sema/native testi yok. Sema'nin bunu kabul edip etmedigi de
   acik degil; davranis netlestirilmeli.

### Orta oncelik (V0.1 sonrasi yapilabilir)

5. **Compile-time diagnostic kapsami**: Mevcut sema kontrollerinin
   sadece ikisi (`hata_tip`, `hata_ana_yok`) test ediliyor. Eksikler:

   - Tanimsiz degisken kullanimi
   - Tanimsiz fonksiyon cagrisi
   - Yanlis arg sayisi
   - Yanlis arg tipi
   - `kir`/`devam` dongu disinda
   - `yazdir` sonucunun deger gibi kullanilmasi
   - Donus tipli fonksiyonun bir dalinda donus eksik
   - `Ana()` parametre almasi
   - `Ana()` donus tipi belirtmesi

   Her birine kucuk bir negative test eklemek `tests/examples.rs` veya
   yeni `tests/sema_diagnostics.rs` icinde basittir.

6. **CLI alt komutlari**: `calistir`, `yorumla --json`, `derle --json`
   icin ek success/failure integration testleri genisletilebilir.
   Temel `calistir --json`, `yorumla --json`, `kontrol --json` ve
   `derle --json` akislari `cli_diagnostics.rs` icinde kapsandi.

### Dusuk oncelik (kozmetik)

7. **Parantezli karmasik ifade**: `(a + b) * c` operator oncelik
   testi implicit covered ama explicit ornek yok.

8. **Unary `-` runtime edge**: `-i64::MIN` durumu (overflow). Native
   backend'in nasil davrandigi test edilmemis. Dile zarar vermez ama
   mevcut bir bug olabilir.

9. **Yorum satiri**: `//` parser tarafindan tanindigi `dongu.ana`
   icinde dolayli covered, ozel bir test yok.

## Onerilen Aksiyon Plani

V0.1 kapatma kriterini hizlamak icin minimum set:

| # | Eklenecek test | Hedef dosya |
|---|---|---|
| 1 | Void fonksiyon parity | Tamamlandi: `native_void_function_matches_interpreter` |
| 2 | Recursive fonksiyon parity (kucuk derinlik) | Tamamlandi: `native_recursive_function_matches_interpreter` |
| 3 | 6 parametre fonksiyon parity | Tamamlandi: `native_six_parameter_function_matches_interpreter` |
| 4 | `mantik` esitlik parity ve sema durumu | Tamamlandi: `native_boolean_equality_matches_interpreter` |

Bu dort ek V0.1 dil yuzeyini test acidan kapatir. Diger
bosluklar V0.2+ ile birlikte kademeli ele alinabilir.

Eklenen testler `cargo test --test native_examples` ve
`cargo test --test native_edge_cases` ile dogrulanmalidir.

## Yenileme

Bu belge dil yuzeyi degistikce (V0.2+ heap, dizi, yapi vs.) yeniden
taranmali. Her yeni dil ozelliginin matrise eklenmesi onerilir.
