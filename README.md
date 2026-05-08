# Anadil

Anadil, Turkce anahtar kelimelerle yazilan kucuk bir programlama dili denemesidir.
V1 hedefi sade, statik tipli ve genisletilebilir bir cekirdek olusturmaktir.

Proje su anda kaynak dosyayi okuyabilen, lexer/parser/semantic analiz yapan, typed AST uzerinden programi calistiran bir interpreter ve Windows x64 icin native compiler MVP'si icerir.

## Durum

Yapilanlar:

- `sayı`, `mantık` ve `metin` temel tipleri
- Degisken tanimlama ve atama
- Aritmetik islemler: `+`, `-`, `*`, `/`
- Unary eksi: `-10`, `-x`, `10 + -3`
- Karsilastirma islemleri: `==`, `!=`, `<`, `>`, `<=`, `>=`
- `eğer` / `değilse`
- Sonsuz, kosullu ve sayacli `döngü`
- `kır`, `devam`, `dön`
- Fonksiyon tanimlama ve fonksiyon cagirma
- `Ana()` giris noktasi
- `yazdir` yerlesik fonksiyonu
- `//` satir yorumlari
- CLI komutlari: `calistir`, `kontrol`, `ast`, `typed`, `asm`, `asm-yaz`, `derle`, `ide`, `ornekler`, `surum`, `yardim`
- Etkilesimli REPL komutu: `repl`

Henuz yapilmayanlar:

- Dizi, struct, class, modul sistemi
- Dosya paketleme veya kurulum araci

## Calistirma

Varsayilan calistirma:

```powershell
cargo run -- examples\topla.ana
```

Acik komutla calistirma:

```powershell
cargo run -- calistir examples\topla.ana
```

IDE veya arac entegrasyonu icin JSON calistirma ciktisi:

```powershell
cargo run -- calistir --json examples\topla.ana
```

Basarili cikti:

```json
{"ok":true,"output":"30","diagnostics":[]}
```

Runtime hatasi:

```json
{"ok":false,"output":"","diagnostics":[{"severity":"error","stage":"runtime","message":"Sifira bolme hatasi","line":2,"column":12}]}
```

Program gecerli mi kontrol etme:

```powershell
cargo run -- kontrol examples\topla.ana
```

IDE veya arac entegrasyonu icin JSON diagnostic ciktisi:

```powershell
cargo run -- kontrol --json examples\hata_tip.ana
```

Basarili cikti:

```json
{"ok":true,"diagnostics":[]}
```

Hatali cikti:

```json
{"ok":false,"diagnostics":[{"severity":"error","stage":"semantic","message":"...","line":2,"column":1}]}
```

Parse edilmis AST'yi yazdirma:

```powershell
cargo run -- ast examples\topla.ana
```

Semantic analizden sonraki typed AST'yi yazdirma:

```powershell
cargo run -- typed examples\topla.ana
```

Windows x64 assembly uretme:

```powershell
cargo run -- asm examples\topla.ana
```

Assembly dosyasini yazma:

```powershell
cargo run -- asm-yaz examples\topla.ana
```

Native executable derleme:

```powershell
cargo run -- derle examples\topla.ana
examples\topla.exe
```

IDE veya arac entegrasyonu icin JSON native build ciktisi:

```powershell
cargo run -- derle --json examples\topla.ana
```

Basarili cikti:

```json
{"ok":true,"exe":"examples\\topla.exe","diagnostics":[]}
```

Build veya toolchain hatasi:

```json
{"ok":false,"exe":null,"diagnostics":[{"severity":"error","stage":"native","message":"...","line":null,"column":null}]}
```

Not: Native derleme Windows x64 hedefler ve Visual Studio Build Tools C++ araclarini kullanir. `derle` komutu `ml64`/`link` PATH icinde yoksa kurulu Build Tools icindeki `vcvars64.bat` dosyasini otomatik bulmaya calisir.

Ornek dosyalari listeleme:

```powershell
cargo run -- ornekler
```

Surum bilgisi:

```powershell
cargo run -- surum
```

Yardim:

```powershell
cargo run -- yardim
```

Etkilesimli REPL:

```powershell
cargo run -- repl
```

REPL icinde:

```text
> yazdir(10);
10
> yazdir(10 + -3);
7
> Kare(x: sayı) -> sayı {
|     dön x * x;
| }
Fonksiyon kaydedildi.
> yazdir(Kare(5));
25
> :cik
```

Not: REPL cok satirli girisi destekler ve fonksiyon tanimlarini oturum boyunca saklar. Degiskenler satirlar arasinda saklanmaz.

Lokal IDE:

```powershell
cargo run -- ide
```

Komut yerel web IDE baslatir ve adresi terminale yazar. IDE icinde ornek dosyalar yuklenebilir, `.ana` dosyasi acilip kaydedilebilir, syntax highlighting ve canli diagnostics kullanilabilir, `Kontrol`, `Calistir` ve `EXE Derle` butonlariyla mevcut compiler protokolu calistirilabilir.

## Ornek

```ana
// Iki sayıyı toplar.
Topla(a: sayı, b: sayı) -> sayı {
    dön a + b;
}

Ana() {
    sonuc: sayı = Topla(10, 20);
    yazdir(sonuc);
}
```

## Gelistirme

### Mimari

Proje iki parcaya ayrilmistir:

- `src/lib.rs`: Dil motoru. Lexer, parser, semantic analiz, typed AST ve interpreter burada kutuphane olarak disari acilir.
- `src/main.rs`: CLI katmani. Dosya okur, komutlari yorumlar ve `lib.rs` icindeki pipeline fonksiyonlarini cagirir.
- `src/native.rs`: Typed AST'den Windows x64 MASM assembly ureten native compiler MVP katmani.
- `src/ide.rs`: `ide` komutuyla calisan lokal web IDE server'i ve arayuzu.

Kutuphane tarafinda uc ana giris fonksiyonu vardir:

```rust
anadil::parse_source(source)
anadil::compile_source(source)
anadil::check_source(source)
anadil::run_source(source)
anadil::run_source_diagnostic(source)
anadil::emit_native_asm_source(source)
```

### Native Compiler MVP

Native compiler hatti:

```text
.ana -> lexer -> parser -> semantic analiz -> typed AST -> Windows x64 assembly -> obj -> exe
```

Desteklenenler:

- `sayi`, `mantik`, `metin`
- Degisken tanimlama ve atama
- Aritmetik ve karsilastirma islemleri
- `eger` / `degilse`
- Sonsuz, kosullu ve sayacli donguler
- `kir`, `devam`, `don`
- Fonksiyon tanimlama ve fonksiyon cagirma
- `yazdir`

Sinirlar:

- Sadece Windows x64 hedeflenir.
- Visual Studio Build Tools C++ araclari gerekir.
- Ilk 4 fonksiyon parametresi register ile, sonraki parametreler stack uzerinden tasinir.
- Runtime hatalari interpreter kadar ayrintili raporlanmaz.
- Sifira bolme native executable icinde kontrollu hata ve `exit(1)` ile raporlanir.
- CLI compile-time hatalari satir/sutun ve caret bilgisiyle basilir; bu cikti mini IDE tarafindan diagnostics paneline baglanabilecek durumdadir.
- `kontrol --json` IDE icin makine okunabilir diagnostic protokolu saglar.
- `calistir --json` IDE icin interpreter output ve diagnostic protokolu saglar.
- `derle --json` IDE icin native build sonucu ve executable yolunu saglar.

### Komutlar

Format kontrolu:

```powershell
cargo fmt --check
```

Testler:

```powershell
cargo test
```

Bu komut unit testleri, CLI testlerini, `examples/` altindaki interpreter testlerini ve Visual Studio Build Tools varsa native executable ornek testlerini calistirir.

Clippy:

```powershell
cargo clippy --all-targets --all-features -- -D warnings
```

## Dokumantasyon

- Guncel dil referansi: [Docs/dil_referansi.md](Docs/dil_referansi.md)
- Native compiler notlari: [Docs/native_compiler.md](Docs/native_compiler.md)
- Local IDE notlari: [Docs/local_ide.md](Docs/local_ide.md)
- Ornek programlar: [examples/README.md](examples/README.md)
