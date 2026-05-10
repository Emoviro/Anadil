# Anadil Proje Durumu

Bu belge Anadil'in su anki teknik durumunu, alinmis kararlari ve gidilmek
istenen yonu tek yerde toplar. Daha detayli notlar icin `native_compiler.md`,
`memory_model.md`, `local_ide.md` ve `todo.md` dosyalarina bakilmalidir.

## Genel Yon

Anadil'in hedefi Turkce sozdizimine sahip, yerel executable uretebilen bir
dil ve lokal IDE deneyimi sunmaktir. Kisa vadeli hedef web/HTML tabanli bir
oyuncak demo degil, Windows uzerinde kendi native compiler hattina sahip
bir V0.1 cikarmaktir.

Compiler tarafinda tercih edilen yon:

```text
.ana kaynak
-> lexer / parser / semantic analiz / typed AST
-> Windows x64 MASM assembly
-> program.obj
+ Anadil runtime library
-> program.exe
```

Anadil C'ye transpile etmez. V0.1 native backend Windows x64 assembly uretir
ve Microsoft Build Tools (`ml64`, `lib`, `link`) ile executable olusturur.
V0.1 kullanici akisi interpreter'a degil, dogrudan native derlemeye dayanir;
interpreter gecici dogrulama ve test oracle'i olarak tutulur.

## Su Ana Kadar Yapilanlar

### Dil Cekirdegi

- Lexer, parser, semantic analiz ve typed AST hatti kuruldu.
- Interpreter mevcut dil alt kumesini calistiriyor; V0.1'de kullanici akisi
  degil, dogrulama/test araci olarak konumlanir.
- `sayi`, `mantik`, `metin` temel tipleri destekleniyor.
- Degisken tanimlama, atama, fonksiyon, `don`, `eger/degilse`, dongu,
  `kir`, `devam` gibi temel yapilar mevcut.
- `yazdir` ASCII alias'i destekleniyor; Turkce ana builtin yazimi karari
  dokumanda tutarli hale getirilmeye devam edecek.

### Native Compiler

- `.ana -> assembly -> obj -> exe` hatti calisir durumda.
- Native backend `src/native.rs` icinde typed AST'den Windows x64 MASM
  assembly uretiyor.
- Program entrypoint'i `Ana()` olarak kabul ediliyor.
- Fonksiyon cagirma, stack argument gecisi ve nested call senaryolari test
  altinda.
- Native/interpreter cikti karsilastirmalari `tests/native_examples.rs` ve
  `tests/native_edge_cases.rs` ile korunuyor.
- Sifira bolme native runtime hatasi olarak raporlaniyor.

### Native Runtime

- Runtime helper'lari generated program assembly'sinden ayrildi.
- Runtime kodu `runtime/anadil_runtime.asm` icinde tutuluyor.
- C runtime bagimliligi kaldirildi:
  - `printf` yok.
  - `getchar` yok.
  - `strcmp` yok.
  - C `exit` yok.
- Runtime Windows `kernel32` API'leri ile calisiyor:
  - `WriteFile`
  - `ReadFile`
  - `ExitProcess`
  - `GetStdHandle`
- Metin karsilastirma runtime icinde byte byte yapiliyor.
- Sayi yazdirma runtime icinde integer-to-decimal donusumu ile yapiliyor.
- Runtime hata formati standart:

```text
Anadil runtime hatasi: Sifira bolme hatasi
```

### Runtime Library Modeli

- Runtime once cached object olarak ayrildi.
- Sonra `.lib` modeline tasindi.
- Derleme hattinda `target/native-runtime/anadil_runtime.obj` ve
  `target/native-runtime/anadil_runtime.lib` uretiliyor.
- Programlar artik runtime object yerine Anadil runtime library ile
  linkleniyor.
- Paralel native derlemelerde runtime library ayni anda yazilip okunmasin
  diye cache lock eklendi.

### Path ve Windows Dayanikliligi

- MASM/link path sorunlarina karsi regression test eklendi.
- Bosluk ve Turkce karakter iceren kaynak yolu test ediliyor:

```text
target/native path cases/Turkce Klasor/deneme dosyasi.ana
```

- OneDrive/Masaustu/Turkce path senaryolari native build riskleri arasinda
  ozel olarak takip ediliyor.

### Lokal IDE

- Lokal desktop IDE mevcut.
- Editor, dosya explorer ve native build paneli uzerinde calisildi.
- Satir numarasi, otomatik girinti, parantez/quote tamamlama gibi temel
  editor kolayliklari eklendi.
- IDE gorsel polish su an ikinci planda. Oncelik compiler ve runtime
  omurgasinin V0.1 icin sabitlenmesi.

## Alinan Kararlar

### C'ye Transpile Yok

Anadil'in native compiler hatti C cikarmayacak. C runtime'dan da cikildi.
Windows tarafinda su an dogrudan MASM assembly ve Anadil runtime library
modeli kullaniliyor.

### V0.1 Memory Modeli Dar Tutulacak

V0.1'de heap allocation, GC ve reference counting yoktur. Mevcut model:

- `sayi` ve `mantik`: register/stack degerleri.
- `metin`: static `.data` literal pointer'i veya runtime sabiti.
- Dinamik string, dizi, struct ve heap obje yok.

Detayli karar belgesi: `Docs/memory_model.md`.

### V0.2+ Icin RC Yonunde Ilerlenecek

Heap gerektiren ozellikler gelince hedef model reference counting'dir:

- `anadil_runtime_tahsis`
- `anadil_runtime_paylas`
- `anadil_runtime_birak`
- length-prefixed `metin`
- daha sonra `dizi`, `yapi`, `zayif<T>`

GC ilk hedef degildir. Cycle problemi ileride `zayif<T>` ile ele alinacak.

### IDE Polish Bekletilecek

IDE V0.1 icin kullanilabilir seviyede tutulacak, ancak gorsel tasarim ve
buyuk UI revizyonlari compiler omurgasi tamamlanmadan ana oncelik olmayacak.

## V0.1 Icin Kalan Ana Isler

Compiler tarafinda V0.1'e "tamam" diyebilmek icin kalan ana isler:

1. V0.1 compiler tamam kriterlerini yazmak.
2. Native/interpreter test kapsaminda son bir bosluk taramasi yapmak.
3. Runtime library modelinin ilk push sonrasi davranisini izlemek.
4. Windows API bagimli runtime katmaninin ileride nasil soyutlanacagini
   kisa not olarak eklemek.
5. ~~README ve native compiler dokumanlarini son kapsamla hizalamak.~~

Bu isler bittikten sonra compiler MVP V0.1 icin sabit kabul edilebilir.

## V0.1 Tamam Kriterleri

V0.1 compiler MVP'sini "tamam" kabul edebilmek icin asagidaki kriterler
saglanmali. Cogu kriter su anda saglanmis durumda; isaretsiz olanlar
yukaridaki "V0.1 Icin Kalan Ana Isler" basliginda listelenen kalan
isler veya `Docs/handoff.md` icindeki sonraki is listesiyle ortusur.

### Dil ve compiler

- [x] `sayi`, `mantik`, `metin` tipleri parser, sema, interpreter ve
  native backend'de destekleniyor.
- [x] Atama, kosullu (`eger`/`degilse`), dongu (sonsuz, kosullu,
  sayacli), `kir`, `devam`, `don`, fonksiyon tanim/cagri ve `Ana()`
  entry point hem interpreter hem native tarafinda calisiyor.
- [x] `yazdir` yerlesik fonksiyonu (Turkce ve ASCII alias) `sayi`,
  `mantik`, `metin` tiplerini yazdiriyor.
- [x] Lexer/parser/sema hatalari satir/sutun ve caret bilgisiyle
  raporlaniyor.

### Native build hatti

- [x] `cargo run -- derle <dosya>.ana` calisan `.exe` uretiyor.
- [x] Native exe beklenen `stdout`'u uretiyor; gecici olarak interpreter
  oracle'i kullanan parite testleri `tests/native_examples.rs` ile korunuyor.
- [x] Linker satiri yalnizca `kernel32.lib` ve `anadil_runtime.lib`
  kullaniyor; CRT (`msvcrt`, `ucrt`, `vcruntime`,
  `legacy_stdio_definitions`) bagimliligi yok.
- [x] Runtime `.obj` ve `.lib` cache, kaynak `.asm` mtime ile
  invalidate oluyor.
- [x] Paralel `derle` cagrilarinda runtime cache yarisi cache lock ile
  cozuluyor.

### Path ve Windows dayanikliligi

- [x] Bosluklu kaynak yolu native build'de calisiyor.
- [x] Turkce karakterli kaynak yolu native build'de calisiyor.
- [x] OneDrive/Masaustu tarzi uzun yollar build sirasinda probleme
  yol acmiyor.

### Test guvencesi

- [x] `tests/native_examples.rs` `examples/` altindaki tum `.ana`
  dosyalarini interpreter/native parity ile dogruluyor.
- [x] `tests/native_edge_cases.rs` runtime hatasi, fonksiyon argumani,
  nested call ve I/O edge case'lerini iceriyor.
- [x] V0.1 oncesi son bosluk tarama gecisi: her dil kurali icin en az
  bir interpreter+native parity ornegi var mi? (sonuclar
  `Docs/test_coverage.md` icindedir; `Docs/todo.md` "Test Bosluklari"
  basligi altinda izlenecek dort yuksek oncelik test maddesi var.)
- [x] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`
  ve `cargo test` temiz makinede yesil.

### CLI ve diagnostics

- [x] CLI komut yuzeyi sabit: `calistir`, `kontrol`, `ast`, `typed`,
  `asm`, `asm-yaz`, `derle`, `ide`, `ornekler`, `surum`, `yardim`,
  `repl`.
- [x] `kontrol --json` ve `derle --json` IDE icin kararli diagnostic semasi
  uretiyor. `calistir --json` gecici interpreter/test protokolu olarak kalir.
- [x] CLI hata ciktisi `tests/cli_diagnostics.rs` ile regresyondan
  korunuyor.

### Dokumantasyon hizalamasi

- [x] `Docs/memory_model.md` V0.1'in heap/RC icermedigini ve V0.2+
  yolunu acikca anlatiyor.
- [x] `README.md` mevcut runtime library modelini, kaldirilmis CRT
  bagimliligini ve sabitlenmis CLI komutlarini yansitiyor.
  (tamamlandi)
- [x] `Docs/native_compiler.md` runtime cache ve `.lib` paketleme
  modelini son haliyle iceriyor (`Runtime Library Paketleme` basligi).
- [x] Windows API bagimli runtime katmaninin platform soyutlamasi icin
  kisa tasarim notu yazilmis (`Docs/runtime_platform_abstraction.md`).

### IDE V0.1 minimum

- [x] Native IDE acilip kapaniyor; klasor ve dosya acilabiliyor.
- [x] Build paneli native exe yolunu, exit code'u ve `stdout`/`stderr`
  raporluyor.
- [x] Diagnostics karti tikla-git ile editor konumuna gidiyor.
- [ ] Native IDE smoke test (`Docs/ide_smoke_test.md`) kontrolden
  gecmis ve sonuclar kaydedilmis. Otomatik build/test sonucu kaydedildi;
  manuel GUI akisi bekliyor. (`Docs/todo.md` Native IDE madde 1)

## V0.1 Disi Birakilanlar

Asagidakiler bilerek V0.1 disinda tutulur:

- Heap allocator
- Garbage collector
- Reference counting emit'i
- Dynamic string allocation
- String concat
- Dizi
- Struct / yapi
- Modul sistemi
- Debugger
- Optimizer
- Cross-platform backend

Bu kararlar V0.1'i kucultmek icin degil, once native compiler omurgasini
guvenilir hale getirmek icin alindi.

## Gidilmek Istenen Yon

Kisa vadede hedef:

```text
Anadil V0.1 = lokal IDE + native Windows executable compiler
```

Orta vadede hedef:

```text
Anadil V0.2 = heap modeli + RC runtime + dinamik metin + dizi/yapi temeli
```

Daha uzun vadede hedef:

```text
Anadil = Turkce sozdizimli, kendi runtime'i olan, IDE destekli native dil
```

Bu yolda su prensipler korunacak:

- Kucuk, testli, commitlenebilir adimlar.
- Compiler ve runtime kararlarini dokumanla sabitlemek.
- IDE polish'i compiler kararliligindan sonra yapmak.
- C'ye donmeden native toolchain modelini buyutmek.
- V0.1 kapsam disini net soylemek, V0.2+ yolunu ise tasarimla hazirlamak.
