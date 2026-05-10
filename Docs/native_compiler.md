# Anadil Native Compiler MVP

Bu belge Anadil'in Windows x64 native compiler MVP'sinin nasil calistigini ve bilinen sinirlarini ozetler.

## Pipeline

Native derleme hatti su sekildedir:

```text
.ana kaynak
  -> Lexer
  -> Parser
  -> Semantic analiz
  -> Typed AST
  -> Windows x64 MASM assembly
  -> ml64 ile .obj
  -> link ile .exe
```

Ilgili dosyalar:

- `src/native.rs`: Typed AST'den Windows x64 MASM assembly uretir.
- `runtime/anadil_runtime.asm`: Anadil runtime helper'larini ayri MASM modulu olarak saglar.
- `src/lib.rs`: `emit_native_asm_source` API'sini disari acar.
- `src/main.rs`: `asm`, `asm-yaz` ve `derle` CLI komutlarini calistirir.
- `tests/native_examples.rs`: Ornek programlari native executable olarak derler ve interpreter ciktisiyla karsilastirir.
- `tests/native_edge_cases.rs`: Fonksiyon cagrisi, stack arguman, nested call, karsilastirma ve runtime hata edge case'lerini native/interpreter davranisiyla karsilastirir.
- `tests/cli_diagnostics.rs`: CLI hata ciktisinin IDE tarafindan okunabilir satir/sutun ve caret bilgisi tasidigini kontrol eder.

## CLI Komutlari

Programi insan okunur diagnostic ile kontrol etmek:

```powershell
cargo run -- kontrol examples\topla.ana
```

Programi IDE entegrasyonu icin JSON diagnostic ile kontrol etmek:

```powershell
cargo run -- kontrol --json examples\hata_tip.ana
```

JSON protokolu:

```json
{"ok":false,"diagnostics":[{"severity":"error","stage":"semantic","message":"...","line":2,"column":1}]}
```

Programi IDE entegrasyonu icin JSON output ile calistirmak:

```powershell
cargo run -- calistir --json examples\topla.ana
```

JSON protokolu:

```json
{"ok":true,"output":"30","diagnostics":[]}
```

Runtime hatalari ayni diagnostic listesine `runtime` stage'iyle duser.

Native executable'i IDE entegrasyonu icin JSON build sonucu ile derlemek:

```powershell
cargo run -- derle --json examples\topla.ana
```

JSON protokolu:

```json
{"ok":true,"exe":"examples\\topla.exe","diagnostics":[]}
```

Build veya toolchain hatalari ayni diagnostic listesine `native` stage'iyle duser.

Assembly'yi ekrana basmak:

```powershell
cargo run -- asm examples\topla.ana
```

Assembly dosyasi yazmak:

```powershell
cargo run -- asm-yaz examples\topla.ana
```

Native executable uretmek:

```powershell
cargo run -- derle examples\topla.ana
examples\topla.exe
```

`derle` komutu once program assembly'sini uretir. Sonra `ml64` ile program object file'ini ve `runtime/anadil_runtime.asm` dosyasindan Anadil runtime object file'ini olusturur. `link`, program objesi ile runtime objesini birlestirerek executable olusturur. `ml64` ve `link` PATH icinde yoksa Visual Studio Build Tools altindaki `vcvars64.bat` dosyasini otomatik bulmaya calisir. Uretilen executable program sonunda runtime helper uzerinden Enter bekler; bu, dosyaya Explorer'dan cift tiklandiginda terminal penceresinin hemen kapanmamasini saglar.

## Hedef Platform

Su anki MVP yalnizca Windows x64 hedefler.

Kullanilan araclar:

- Microsoft Macro Assembler: `ml64`
- Microsoft linker: `link`
- Visual Studio Build Tools C++ toolchain
- Windows import library: `kernel32.lib`

## Assembly Modeli

Backend MASM uyumlu assembly uretir.

Program girisi:

- Assembly icinde `main PROC` uretilir.
- `main`, Anadil giris noktasi olan `Ana()` fonksiyonunu cagirir.
- Linker'a `/ENTRY:main` verilir.

Fonksiyon adlari:

```text
Anadil: Topla
Assembly: anadil_fn_Topla
```

Label'lar backend tarafinda uretilir:

```text
L_return_0
L_else_1
L_loop_2
```

## Calling Convention

Backend Windows x64 calling convention'i takip eder.

Ilk 4 parametre register ile tasinir:

```text
1. parametre -> rcx
2. parametre -> rdx
3. parametre -> r8
4. parametre -> r9
```

5. parametreden sonrasi caller tarafindan call alanindaki stack argument bolgesine yazilir. Callee, fonksiyon girisinde bu stack argument'lari kendi local slot'larina kopyalar.

Fonksiyon donus degeri `rax` register'i ile tasinir.

Argumanlar interpreter ile ayni yan etki sirasi icin soldan saga degerlendirilir. Backend, ic ice fonksiyon cagrilarinda gecici degerleri korumak icin her call oncesinde Windows x64 shadow space ve stack arguman alanini gecici olarak ayirir.

## Stack Frame

Her Anadil fonksiyonu kendi stack frame'ini kurar:

```asm
push rbp
mov rbp, rsp
sub rsp, <frame_size>
```

Local degerler `LocalId` uzerinden stack slot'lara yerlestirilir:

```text
LocalId(0) -> [rbp-8]
LocalId(1) -> [rbp-16]
LocalId(2) -> [rbp-24]
```

Frame size, local sayisina ve fonksiyon icindeki en genis call'un arguman scratch ihtiyacina gore hesaplanir. Sonuc 16 byte hizalamaya yuvarlanir. Windows x64 icin gerekli shadow space her call oncesinde ayrilir.

## Deger Temsili

`sayi`:

- 64-bit signed integer
- Register/stack temsili: `i64`

`mantik`:

- `false`: `0`
- `true`: `1`

`metin`:

- Su an yalnizca string literal desteklenir.
- Literal'lar assembly `.data` bolumune null-terminated byte dizisi olarak yazilir.
- Runtime'da yeni string allocation yoktur.

Ornek:

```ana
mesaj: metin = "Merhaba";
```

Assembly tarafinda:

```asm
str_0 db "Merhaba", 0
lea rax, str_0
```

## yazdır Runtime Modeli

`yazdır` native backend'de Anadil runtime helper'larina dusurulur. `yazdir` ASCII alias'i da ayni builtin'e baglanir. Helper'lar `runtime/anadil_runtime.asm` icinde ayri assemble edilir ve Windows `WriteFile` uzerinden stdout'a byte yazar.

Kullanilan formatlar:

```text
sayi  -> "%lld\n"
metin -> "%s\n"
```

`mantik` degerleri once static metinlere cevrilir:

```text
true  -> "dogru"  UTF-8: doğru
false -> "yanlis" UTF-8: yanlış
```

## Metin Karsilastirma

`metin == metin` ve `metin != metin` islemleri `anadil_runtime_strcmp` helper'i ile uretilir. Bu helper runtime icinde byte byte karsilastirma yapar; C runtime `strcmp` cagrisi kullanmaz.

```text
anadil_runtime_strcmp(a, b) == 0 -> esit
anadil_runtime_strcmp(a, b) != 0 -> esit degil
```

## Runtime Hatalari

Native MVP sifira bolme icin interpreter'a benzer kontrollu hata davranisi uretir. Kodgen bu durumda `anadil_runtime_panic` helper'ini cagirir:

```text
Sifira bolme hatasi
```

Bu durumda executable Enter bekledikten sonra Windows `ExitProcess(1)` ile biter. Kaynak satir/sutun bilgisi su an native executable icine gomulmez; IDE entegrasyonu icin compile-time lexer/parser/semantic hatalari CLI tarafinda caret'li diagnostic ve `kontrol --json` ile structured diagnostic olarak kalir.

## Runtime Helper ABI

Compiler kullanici kodundan dogrudan platform API'si veya C runtime cagrisi uretmek yerine kendi runtime helper'larini cagirir:

```text
anadil_runtime_print_sayi(rcx=sayi)
anadil_runtime_print_metin(rcx=metin_ptr)
anadil_runtime_print_mantik(rcx=0/1)
anadil_runtime_strcmp(rcx=left_ptr, rdx=right_ptr) -> eax
anadil_runtime_wait_before_exit()
anadil_runtime_panic(rcx=message_ptr) -> process exit 1
```

Bu helper'lar program assembly'sinden ayri bir runtime object file olarak linklenir. Runtime I/O, bekleme ve process sonlandirma Windows `kernel32` API'lerine baglidir; C runtime import'u artik native executable link hattinda gerekli degildir.

## Memory Management

Su an native backend'de heap allocation yoktur.

Bu nedenle:

- Garbage collector yoktur.
- Reference counting yoktur.
- Manual `free`/`delete` modeli yoktur.
- String literal'lar static `.data` bolumunde yasar.
- `sayi`, `mantik` ve local `metin` referanslari stack slot'larda tutulur.

Bu model mevcut dil alt kumesi icin yeterlidir. `metin` birlestirme, dizi, struct veya dinamik obje destegi eklendiginde runtime allocator veya GC tasarimi gerekecektir.

Onerilen sonraki bellek modeli:

```text
anadil_alloc(size)
anadil_runtime_shutdown()
```

Ilk asamada arena allocator yeterli olabilir. Mark-and-sweep GC daha sonra eklenebilir.

## Test Stratejisi

Native ornek testi su karsilastirmayi yapar:

```text
interpreter ciktisi == native executable ciktisi
```

Test edilen ornekler:

- `topla`
- `negatif`
- `kosul`
- `fonksiyon`
- `mantik`
- `metin`
- `kosullu_dongu`
- `dongu`
- `sonsuz_dongu`
- `kapsam`
- `native_mvp`

Visual Studio native toolchain bulunamazsa native integration testi kendini skip eder.

## Bilinen Sinirlar

- Sadece Windows x64 hedeflenir.
- Heap allocation yoktur.
- Garbage collector yoktur.
- String literal disinda runtime metin uretimi yoktur.
- Native runtime hatalari interpreter kadar ayrintili raporlanmaz.
- Optimizasyon yoktur.
- Debug info uretilmez.

## Sonraki Hedefler

Kisa vadeli hedefler:

- Native derleme komutunu daha temiz hata mesajlariyla zenginlestirmek.
- IDE entegrasyonu icin compiler API'lerini netlestirmek.

Orta vadeli hedefler:

- Runtime allocator eklemek.
- Dinamik `metin` islemlerini desteklemek.
- Daha temiz bir IR katmani tasarlamak.
- Windows disi hedefleri degerlendirmek.
