# Anadil

Anadil, Turkce anahtar kelimelerle yazilan kucuk bir programlama dili denemesidir.
V0.1 hedefi lokal IDE'de ve CLI'da ana calistirma yolunu Windows x64 native
executable compiler hattina tasimaktir. Interpreter bu surecte gecici
dogrulama/test araci olarak kalir; kullanici akisi derleme odaklidir.

Proje su anda kaynak dosyayi okuyabilen, lexer/parser/semantic analiz yapan,
typed AST uzerinden Windows x64 native executable uretebilen bir compiler
MVP'si icerir. Interpreter hatti dogrulama ve test destegi icin korunur.

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
- V0.2 branch'inde `uzunluk(metin) -> sayı` yerlesik fonksiyonu
- V0.2 branch'inde `metin + metin` dinamik birlestirme MVP'si
- `//` satir yorumlari
- CLI komutlari: `calistir`, `yorumla`, `kontrol`, `ast`, `typed`, `ir`, `asm`, `asm-yaz`, `derle`, `ide`, `ornekler`, `surum`, `yardim`
- Etkilesimli REPL komutu: `repl`

Bu CLI yuzeyi V0.1 icin sabit kabul edilir.

Henuz yapilmayanlar:

- Dizi, struct, class, modul sistemi
- Otomatik referans sayma cleanup'i

## Indir ve Kullan

Anadil'i kullanmak icin Rust toolchain'ine veya kaynak kodu derlemeye
ihtiyaciniz yok. Hazir Windows x64 paketi:

[Anadil v0.1.2 - GitHub Releases](https://github.com/ArsenAlighieri/Anadil/releases/tag/v0.1.2)

Iki kurulum yolu vardir:

- **Setup sihirbazi** (`Anadil-Setup-vX.Y.Z.exe`): per-user kurulum,
  opsiyonel `PATH` eklemesi, Baslat menusu kisayollari ve `.ana` dosya
  eslemesi. Admin gerekmez. Standart Windows kaldirma sihirbazi ile
  temiz silinir.
- **ZIP arsivi** (`Anadil-vX.Y.Z-windows-x64.zip`): herhangi bir klasore
  cikartip dogrudan calistirilabilir; tasinabilir kullanim icin uygun.

Detayli kurulum talimati ZIP/Setup icindeki `KURULUM.txt` dosyasinda;
surum notlari `CHANGELOG.txt` icinde.

Native derleme (`anadil derle`) icin Visual Studio Build Tools
(`link.exe`) kurulu olmalidir; Anadil paketi runtime kutuphanesini
onceden derlenmis olarak ship eder, Build Tools yalnizca son linkleme
adimi icin gereklidir. Interpreter modu (`anadil yorumla`) ve native
IDE Build Tools olmadan calisir.

[Build Tools indirme sayfasi](https://visualstudio.microsoft.com/visual-cpp-build-tools/).

## Calistirma

V0.1'de ana yol native executable'dir. `calistir` ve ciplak dosya
cagirmak kaynak dosyayi once `.exe` olarak derler, sonra uretilen
programi calistirir. Interpreter sadece `yorumla` komutuyla acikca
istenirse kullanilir.

Komut ozeti:

- `anadil <dosya.ana>`: native derle ve calistir.
- `anadil calistir <dosya.ana>`: native derle ve calistir.
- `anadil yorumla <dosya.ana>`: interpreter/debug yolu.
- `anadil derle <dosya.ana>`: sadece native `.exe` uretir.

Cargo uzerinden varsayilan calistirma:

```powershell
cargo run -- examples\topla.ana
```

Acik komutla native calistirma:

```powershell
cargo run -- calistir examples\topla.ana
```

IDE veya arac entegrasyonu icin JSON native calistirma ciktisi:

```powershell
cargo run -- calistir --json examples\topla.ana
```

Basarili cikti:

```json
{"ok":true,"output":"30","diagnostics":[]}
```

Runtime hatasi:

```json
{"ok":false,"output":"Anadil runtime hatasi: Sifira bolme hatasi","diagnostics":[{"severity":"error","stage":"native","message":"Native program basarisiz bitti: 1","line":null,"column":null}]}
```

Native executable icindeki runtime hatalari su an kaynak satir/sutun bilgisi
tasimaz; hata metnini standart output/stderr uzerinden raporlar ve exit code
`1` ile biter. Interpreter/debug yolu gerekiyorsa:

```powershell
cargo run -- yorumla examples\topla.ana
cargo run -- yorumla --json examples\topla.ana
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

V0.2 ara temsilini yazdirma:

```powershell
cargo run -- ir examples\topla.ana
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

Not: Native derleme Windows x64 hedefler ve Visual Studio Build Tools C++ araclarini kullanir. `derle` komutu `ml64`/`link`/`lib` PATH icinde yoksa kurulu Build Tools icindeki `vcvars64.bat` dosyasini otomatik bulmaya calisir.

Native build hatti program object'ini cached Anadil runtime library ile linkler.
Runtime artifact'leri `target/native-runtime/anadil_runtime.obj` ve
`target/native-runtime/anadil_runtime.lib` altinda tutulur; `runtime/anadil_runtime.asm`
degismediyse sonraki derlemelerde runtime yeniden assemble edilmez. Link
satirinda yalnizca program object'i, `anadil_runtime.lib` ve `kernel32.lib`
bulunur; C runtime kutuphaneleri (`msvcrt`, `ucrt`, `vcruntime`,
`legacy_stdio_definitions`) artik gerekli degildir.

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

Lokal web IDE (deneysel / ikincil):

```powershell
cargo run -- ide
```

Bu komut eski yerel web IDE'yi baslatir ve adresi terminale yazar. V0.1
icin birincil IDE `anadil-ide.exe` native desktop IDE'dir; web IDE yalnizca
deneysel/ikincil arac olarak tutulur.

Native executable IDE:

```powershell
cargo run --bin anadil-ide
```

Release `.exe` uretmek:

```powershell
cargo build --release --bin anadil-ide
target\release\anadil-ide.exe
```

Native IDE browser veya localhost kullanmaz. Compiler API'lerini dogrudan cagirir; build icin mevcut `anadil derle --json` protokolunu kullanir.

Native IDE kisayollari:

```text
Ctrl+O  Dosya ac
Ctrl+S  Kaydet
F5      Native derle ve calistir
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
- `Yap` veya `F5`, gerekiyorsa aktif dosyayi kaydeder, native `.exe` uretir ve hemen calistirir.
- `EXE Derle` veya `Ctrl+B`, sadece `.exe` dosyasini aktif `.ana` dosyasinin yanina uretir.
- `Yap`/`F5` basariliysa program stdout/stderr sonucu `Cikti` sekmesinde gorunur.
- Build sekmesi derlenen kaynak dosyayi, uretilen `.exe` yolunu, exit/stdout/stderr detaylarini ve toolchain hatalarinda baslikli bolumler halinde kisa cozum notunu gosterir.
- `EXE Calistir`, son uretilen native executable'i tekrar calistirir ve stdout/stderr/exit code bilgisini `Cikti` ve `Build` sekmelerinde gosterir.
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

### V0.2 Metin Ornegi

```ana
Selamla(ad: metin) -> metin {
    dön "Merhaba " + ad;
}

Ana() {
    mesaj: metin = Selamla("Anadil");
    yazdir(mesaj);
    yazdir(uzunluk(mesaj));
}
```

## Gelistirme

### Mimari

Proje iki parcaya ayrilmistir:

- `src/lib.rs`: Dil motoru. Lexer, parser, semantic analiz, typed AST, native compiler API'leri ve gecici interpreter dogrulama API'leri burada kutuphane olarak disari acilir.
- `src/main.rs`: CLI katmani. Dosya okur, komutlari yorumlar ve `lib.rs` icindeki pipeline fonksiyonlarini cagirir.
- `src/native.rs`: Typed AST'den Windows x64 MASM assembly ureten native compiler MVP katmani.
- `runtime/anadil_runtime.asm`: Native executable'lara cached `.lib` olarak linklenen Anadil runtime helper modulu.
- `src/ide.rs`: `ide` komutuyla calisan lokal web IDE server'i ve arayuzu.
- `src/bin/anadil-ide.rs`: Native executable IDE.

Kutuphane tarafinda ana giris fonksiyonlari:

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
.ana -> lexer -> parser -> semantic analiz -> typed AST -> Windows x64 assembly -> program obj + Anadil runtime lib -> exe
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
- `uzunluk(metin) -> sayı`
- `yazdir`, metin karsilastirma ve runtime hata cikislari ayri Anadil runtime kutuphanesi uzerinden linklenir.
- Runtime kutuphanesi `target/native-runtime/anadil_runtime.lib` olarak cache'lenir ve `runtime/anadil_runtime.asm` timestamp'iyle invalidate edilir.
- Typed AST optimizer sabit katlama ve basit cebirsel sadelestirme uygular.
- `anadil ir` V0.2 ara temsilinde runtime operasyonlarini `runtime.yazdir_metin` ve `runtime.metin_esit` gibi acik isimlerle gosterir.
- Static `metin` literal'lari native assembly'de length-prefixed Anadil metin nesnesi olarak emit edilir.
- `metin + metin`, runtime heap allocation ile yeni length-prefixed metin uretir.
- `uzunluk(metin)` native backend'de `anadil_runtime_metin_uzunluk` helper'ina dusurulur.
- Nested `metin + metin` ve user-defined fonksiyon return operandlari concat sonrasi temizlenir.
- `yazdir` icindeki owned `metin` temporary'leri ve kullanilmayan owned expression sonuclari temizlenir.
- Void fonksiyonlardaki ust seviye `metin` local'leri icin temel `birak` cleanup'i emit edilir.
- Literal/concat RHS ile `metin` yeniden atamalarinda eski deger temel cleanup ile birakilir.
- Local `metin` paylasiminda `paylas` emit edilir.
- User-defined fonksiyonlara local `metin` argumani gecirilirken `paylas` emit edilir.
- Local `metin` return degerleri cleanup sonrasi caller'a canli doner.
- If/else branch'lerinin normal cikisinda branch-scope `metin` local cleanup'i vardir.
- Loop body `metin` local'leri normal tur sonu, `kır` ve `devam` akislarinda temizlenir.
- Native cikti dogrulugu su an interpreter oracle'i kullanan ornek programlar ve edge-case testleriyle korunur.

Sinirlar:

- Sadece Windows x64 hedeflenir.
- Visual Studio Build Tools C++ araclari gerekir.
- Native runtime I/O ve process cikisi Windows `kernel32` API'leri uzerinden calisir.
- Runtime helper'lari `GetStdHandle`, `WriteFile`, `ReadFile` ve `ExitProcess` kullanir; `printf`, `getchar`, `strcmp` veya C `exit` cagrisi yoktur.
- Link satirinda Anadil runtime library ve `kernel32.lib` disinda CRT kutuphanesi yoktur.
- Ilk 4 fonksiyon parametresi register ile, sonraki parametreler stack uzerinden tasinir.
- Dinamik `metin` allocation simdilik `metin + metin` ile sinirlidir; otomatik `birak` emit'i fonksiyon cikisi, if/loop scope cikisi ve owned/static RHS assignment replacement icin kademeli olarak vardir.
- Return ownership simdilik local `metin` ve owned concat return degerleriyle sinirlidir.
- RC emit henuz last-use ve tum kompleks ownership optimizasyonlarini kapsayan tam bir model degildir.
- Runtime hatalari interpreter kadar ayrintili raporlanmaz.
- Sifira bolme native executable icinde kontrollu hata ve process exit code `1` ile raporlanir.
- Native executable program sonunda ve runtime hata cikisinda terminalin kapanmamasi icin Enter bekler.
- CLI compile-time hatalari satir/sutun ve caret bilgisiyle basilir; bu cikti mini IDE tarafindan diagnostics paneline baglanabilecek durumdadir.
- `kontrol --json` IDE icin makine okunabilir diagnostic protokolu saglar.
- `calistir --json` native derle-ve-calistir output ve diagnostic protokolu saglar.
- `yorumla --json` gecici interpreter/debug output ve diagnostic protokolu saglar.
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

Bu komut unit testleri, CLI testlerini, `examples/` altindaki interpreter testlerini ve Visual Studio Build Tools varsa native executable ornek/parity testlerini calistirir.

Clippy:

```powershell
cargo clippy --all-targets --all-features -- -D warnings
```

## Dokumantasyon

- Proje raporu (akademik): [Docs/proje_raporu.md](Docs/proje_raporu.md)
- Sunum demo akisi: [Docs/demo_akisi.md](Docs/demo_akisi.md)
- Guncel dil referansi: [Docs/dil_referansi.md](Docs/dil_referansi.md)
- Proje durum ozeti: [Docs/project_status.md](Docs/project_status.md)
- Native compiler notlari: [Docs/native_compiler.md](Docs/native_compiler.md)
- Bellek modeli notlari: [Docs/memory_model.md](Docs/memory_model.md)
- Runtime platform soyutlama notlari: [Docs/runtime_platform_abstraction.md](Docs/runtime_platform_abstraction.md)
- Release dagitim modeli: [Docs/release_layout.md](Docs/release_layout.md)
- Local IDE notlari: [Docs/local_ide.md](Docs/local_ide.md)
- Native IDE smoke test: [Docs/ide_smoke_test.md](Docs/ide_smoke_test.md)
- Test kapsam ve bosluk analizi: [Docs/test_coverage.md](Docs/test_coverage.md)
- Test gap onceliklendirmesi: [Docs/test_gap_analizi.md](Docs/test_gap_analizi.md)
- Yapilacaklar: [Docs/todo.md](Docs/todo.md)
- Ornek programlar: [examples/README.md](examples/README.md)
