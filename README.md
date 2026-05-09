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
- `yazdır` yerlesik fonksiyonu (`yazdir` ASCII alias'i da desteklenir)
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
> yazdır(10);
10
> yazdır(10 + -3);
7
> Kare(x: sayı) -> sayı {
|     dön x * x;
| }
Fonksiyon kaydedildi.
> yazdır(Kare(5));
25
> :cik
```

Not: REPL cok satirli girisi destekler ve fonksiyon tanimlarini oturum boyunca saklar. Degiskenler satirlar arasinda saklanmaz.

Lokal web IDE:

```powershell
cargo run -- ide
```

Komut yerel web IDE baslatir ve adresi terminale yazar. IDE icinde ornek dosyalar yuklenebilir, `.ana` dosyasi acilip kaydedilebilir, syntax highlighting ve canli diagnostics kullanilabilir, `Kontrol`, `Calistir` ve `EXE Derle` butonlariyla mevcut compiler protokolu calistirilabilir.

Native executable IDE:

```powershell
cargo run --bin anadil-ide
```

Release `.exe` uretmek:

```powershell
cargo build --release --bin anadil-ide
target\release\anadil-ide.exe
```

Native IDE browser veya localhost kullanmaz. Compiler API'lerini dogrudan cagirir; `EXE Derle` icin mevcut `anadil derle --json` protokolunu kullanir.

Native IDE kisayollari:

```text
Ctrl+O  Dosya ac
Ctrl+S  Kaydet
F5      Secili modu calistir
Ctrl+B  EXE Derle
Ctrl+Shift+F5  EXE Calistir
```

`Ac` ve `Farkli Kaydet` native Windows dosya penceresi acar. Kaydedilmemis degisiklik varsa pencere basliginda ve dosya adinda `*` gorunur.

Native IDE proje akisi:

- `Klasor Ac` ile bir proje klasoru secilir.
- Sol explorer'da klasor altindaki `.ana` dosyalari listelenir; `target` ve `.git` atlanir.
- Listedeki dosyaya tiklaninca editor aktif dosyayi acar.
- `Yeni` yeni bir `adsiz.ana` taslagi acar; sol paneldeki `Olustur` proje icinde adli dosya olusturur.
- Aktif dosya sol panelden yeniden adlandirilabilir veya onay penceresiyle silinebilir.
- Son acilan proje klasoru ve dosya bir sonraki IDE acilisinda geri yuklenir.
- Kaydedilmemis degisiklik varken baska dosya acmaya calisilirsa IDE onay ister.
- Ust bardaki mod seciciden `Interpret et`, `Compile et` veya `Karsilastir` secilir; `Yap` veya `F5` secili modu calistirir.
- `EXE Derle`, gerekiyorsa aktif dosyayi kaydeder ve `.exe` dosyasini aktif `.ana` dosyasinin yanina uretir.
- Build sekmesi derlenen kaynak dosyayi, uretilen `.exe` yolunu, exit/stdout/stderr detaylarini ve toolchain hatalarinda kisa cozum notunu gosterir.
- `EXE Calistir`, son uretilen native executable'i calistirir ve stdout/stderr/exit code bilgisini `Build` sekmesinde gosterir.
- `Karsilastir`, ayni kaynak kodu interpreter ve native executable olarak calistirir; stdout farklarini `Build` sekmesinde gosterir.
- Native executable, Explorer'dan cift tiklaninca terminal penceresi kapanmadan once Enter bekler.
- Alt panelde `Cikti`, `Diagnostics` ve `Build` sekmeleri vardir.
- `Diagnostics` sekmesindeki satir/sutun bilgili hata kartlarina tiklaninca editor ilgili konuma odaklanir.
- Editor ve explorer, VS Code benzeri koyu tema ve ince resize ayiricilari tasir; diagnostic kartina tiklaninca ilgili kod konumuna odaklanir.

## Ornek

```ana
// Iki sayıyı toplar.
Topla(a: sayı, b: sayı) -> sayı {
    dön a + b;
}

Ana() {
    sonuc: sayı = Topla(10, 20);
    yazdır(sonuc);
}
```

## Gelistirme

### Mimari

Proje iki parcaya ayrilmistir:

- `src/lib.rs`: Dil motoru. Lexer, parser, semantic analiz, typed AST ve interpreter burada kutuphane olarak disari acilir.
- `src/main.rs`: CLI katmani. Dosya okur, komutlari yorumlar ve `lib.rs` icindeki pipeline fonksiyonlarini cagirir.
- `src/native.rs`: Typed AST'den Windows x64 MASM assembly ureten native compiler MVP katmani.
- `src/ide.rs`: `ide` komutuyla calisan lokal web IDE server'i ve arayuzu.
- `src/bin/anadil-ide.rs`: Native executable IDE.

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
- `yazdır` (`yazdir` alias'i desteklenir)

Sinirlar:

- Sadece Windows x64 hedeflenir.
- Visual Studio Build Tools C++ araclari gerekir.
- Ilk 4 fonksiyon parametresi register ile, sonraki parametreler stack uzerinden tasinir.
- Runtime hatalari interpreter kadar ayrintili raporlanmaz.
- Sifira bolme native executable icinde kontrollu hata ve `exit(1)` ile raporlanir.
- Native executable program sonunda ve runtime hata cikisinda terminalin kapanmamasi icin Enter bekler.
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
- Native IDE smoke test: [Docs/ide_smoke_test.md](Docs/ide_smoke_test.md)
- Yapilacaklar: [Docs/todo.md](Docs/todo.md)
- Ornek programlar: [examples/README.md](examples/README.md)
