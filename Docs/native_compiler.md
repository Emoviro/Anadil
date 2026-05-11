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
  -> Typed AST optimizer
  -> Windows x64 MASM assembly
  -> ml64 ile .obj
  -> lib ile runtime .lib
  -> link ile .exe
```

Ilgili dosyalar:

- `src/native.rs`: Typed AST'den Windows x64 MASM assembly uretir.
- `src/optimizer.rs`: Typed AST uzerinde sabit katlama ve basit cebirsel
  sadelestirme gecisi uygular.
- `src/ir.rs`: V0.2 ara temsilini typed AST'den dusurur ve okunabilir
  metin formatiyla yazdirir.
- `runtime/anadil_runtime.asm`: Anadil runtime helper'larini ayri MASM modulu olarak saglar.
- `src/lib.rs`: `emit_native_asm_source` API'sini disari acar.
- `src/main.rs`: `ir`, `asm`, `asm-yaz` ve `derle` CLI komutlarini calistirir.
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

`calistir`, kaynak dosyayi native executable olarak derler ve uretilen
programi hemen calistirir. Native runtime hatalari su an kaynak satir/sutun
tasimaz; diagnostic stage'i `native` olur ve program stdout'u `output`
alaninda korunur.

Interpreter/debug yolu gerekiyorsa:

```powershell
cargo run -- yorumla --json examples\topla.ana
```

`yorumla --json` runtime hatalarini satir/sutun bilgili `runtime` stage'iyle
raporlar; bu yol V0.1'de dogrulama/test araci olarak kalir.

V0.2 ara temsilini goruntulemek:

```powershell
cargo run -- ir examples\topla.ana
```

Bu komut typed AST optimizer sonrasi programi okunabilir Anadil IR
formatinda yazar. V0.2'de IR henuz native backend'in girdisi degildir;
backend switch icin hazirlik ve test yuzeyi olarak tutulur.
Runtime'a dusen islemler IR'de acik isimlerle gorunur; ornegin
`yazdir(metin)` -> `runtime.yazdir_metin(...)`, `metin == metin` ->
`runtime.metin_esit(...)`. Bu, dinamik `metin` migration'i sirasinda
backend'in hangi runtime ABI'sine baglanacagini gorunur kilar.

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

`derle` komutu once program assembly'sini uretir ve `ml64` ile program object file'ini olusturur. Anadil runtime object file'i `target/native-runtime/anadil_runtime.obj` altinda cache'lenir; obje yoksa veya `runtime/anadil_runtime.asm` daha yeniyse yeniden assemble edilir. Ardindan `lib`, `target/native-runtime/anadil_runtime.lib` dosyasini olusturur veya object file daha yeniyse gunceller. `link`, program objesi ile cached runtime kutuphanesini birlestirerek executable olusturur. `ml64`, `lib` ve `link` PATH icinde yoksa Visual Studio Build Tools altindaki `vcvars64.bat` dosyasini otomatik bulmaya calisir. Uretilen executable program sonunda runtime helper uzerinden Enter bekler; bu, dosyaya Explorer'dan cift tiklandiginda terminal penceresinin hemen kapanmamasini saglar.

## Hedef Platform

Su anki MVP yalnizca Windows x64 hedefler.

Kullanilan araclar:

- Microsoft Macro Assembler: `ml64`
- Microsoft Library Manager: `lib`
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
- Literal'lar assembly `.data` bolumune statik length-prefixed Anadil metin
  nesnesi olarak yazilir: `[refcount][tip_id][len][bytes...]`.
- Literal data pointer'i length alanini gosterir; runtime bytes alanina
  `ptr + 8` ile ulasir.
- Runtime'da yeni string allocation yoktur.

Ornek:

```ana
mesaj: metin = "Merhaba";
```

Assembly tarafinda:

```asm
str_0_refcount dq 4000000000000000h
str_0_tip dq 1
str_0 dq 7
str_0_bytes db "Merhaba"
lea rax, str_0
```

## yazdır Runtime Modeli

`yazdır` native backend'de Anadil runtime helper'larina dusurulur. `yazdir` ASCII alias'i da ayni builtin'e baglanir. Helper'lar `runtime/anadil_runtime.asm` icinde ayri assemble edilir ve Windows `WriteFile` uzerinden stdout'a byte yazar.

Kullanilan formatlar:

```text
sayi  -> decimal + newline
metin -> length-prefixed bytes + newline
```

`mantik` degerleri once static metinlere cevrilir:

```text
true  -> "dogru"  UTF-8: doğru
false -> "yanlis" UTF-8: yanlış
```

## Metin Karsilastirma

`metin == metin` ve `metin != metin` islemleri `anadil_runtime_metin_esit` helper'i ile uretilir. Bu helper once length alanlarini, sonra byte'lari karsilastirir; C runtime `strcmp` cagrisi kullanmaz.

```text
anadil_runtime_metin_esit(a, b) != 0 -> esit
anadil_runtime_metin_esit(a, b) == 0 -> esit degil
```

## Metin Birlestirme

V0.2 branch'inde `metin + metin` MVP olarak desteklenir. Native backend
iki operand'i degerlendirir ve `anadil_runtime_metin_birlestir` helper'ini
cagirir. Helper `anadil_runtime_tahsis` ile yeni length-prefixed heap metin
nesnesi olusturur, sol ve sag operand byte'larini arka arkaya kopyalar ve
yeni data pointer'i dondurur.

Bu ilk dilimden sonra RC cleanup emit'i kademeli olarak genisletildi:
fonksiyon cikisi, atama replacement, local paylasimi, parametre sahipligi,
return sahipligi, if/else branch scope cikisi ve loop body scope cikisi
artik native backend tarafindan ele alinir.

Ilk cleanup dilimi olarak native backend, donus degeri olmayan fonksiyonlarin
ust seviye `metin` local'leri icin fonksiyon cikisinda
`anadil_runtime_birak` emit eder. Static literal'lar refcount sentinel'i
tasidigi icin bu cagri no-op olur; `metin + metin` sonucu heap nesnesi ise
serbest birakilir.

Bu henuz tam RC degildir: last-use optimizasyonu ve daha karmasik ownership
indirgemeleri sonraki RC emit fazlarina kalir.

### Metin Ownership Matrisi

Native backend `metin` ifadelerini kodgen sirasinda su dar siniflara ayirir:

| Ifade formu | Ownership sinifi | Kodgen kuralı |
| --- | --- | --- |
| `"sabit"` | Static literal | Refcount sentinel tasir; `birak` no-op olur. |
| `local_metin` | Shared reference | Yeni slot/callee/caller referansi gerekiyorsa `paylas` edilir. |
| `a + b` | Owned temporary | Sonucu heap nesnesidir; hedefe devredilmezse caller temizler. |
| `Uret()` -> `metin` | Owned temporary | Return ownership caller'a gecer; hedefe devredilmezse caller temizler. |
| `sayi`, `mantik`, void | Not string | RC emit edilmez. |

Concat ifadesi baska bir concat veya user-defined fonksiyon return degerini
operand olarak kullaniyorsa, native backend `anadil_runtime_metin_birlestir`
sonucunu korur ve owned temporary operandlari `anadil_runtime_birak` ile
temizler. Boylece `"A" + "B" + Uret()` gibi zincirlerde ara heap metinler
program sonuna kadar tasinmaz.

`yazdir("A" + "B")` gibi builtin cagri argumanlarinda callee param cleanup
yolu olmadigi icin caller, yazdirma bittikten sonra owned temporary'yi
`birak` eder. Benzer sekilde `Uret();` veya `"A" + "B";` gibi sonucu
kullanilmayan owned expression statement'lari da hemen temizlenir.

`uzunluk(metin)` V0.2'de length-prefixed layout'un ilk kullaniciya acik
builtin'idir. Native backend bunu `anadil_runtime_metin_uzunluk` helper'ina
dusurur ve `sayi` dondurur. Ilk MVP'de deger runtime nesnesindeki byte length
alanidir; Unicode grapheme/karakter sayma sonraki metin API katmanina kalir.
Owned temporary argumanlar sonuc korunduktan sonra caller tarafinda `birak`
edilir.

Atama tarafinda ilk guvenli daralma eklendi: `metin` local'i literal veya
`metin + metin` gibi owned/static bir ifadeyle yeniden atanirken eski slot
degeri yeni deger korunarak `anadil_runtime_birak` ile birakilir.
`x = y` gibi baska local'den paylas gerektiren assignment'lar henuz tam RC
kurali bekler; bu durumda compiler ekstra `paylas`/`birak` emit etmez.

Sonraki dar RC adimi olarak local `metin` paylasimi da eklendi:
`b: metin = a` ve `b = a` durumlarinda RHS local pointer'i
`anadil_runtime_paylas` ile retain edilir. Assignment formunda yeni referans
retain edildikten sonra eski target degeri `birak` edilir ve slot yeni
pointer ile guncellenir. Bu sira self-assignment ve ayni heap objesini
paylasma durumlarinda refcount'un erken sifira inmesini engeller.

Void fonksiyonlarda `metin` parametreleri de fonksiyon cikisinda
`anadil_runtime_birak` ile temizlenir. Caller tarafinda local `metin`
argumani user-defined fonksiyona verilirken `anadil_runtime_paylas` emit
edilir; boylece caller local'i ve callee parametresi ayni heap nesnesini
guvenli sekilde paylasir. Inline owned concat argumani retain edilmez,
callee cikisinda birakilarak sahiplik transferi gibi davranir. Ayni kural
user-defined fonksiyon return degeri dogrudan baska user-defined fonksiyona
arguman olarak verildiginde de gecerlidir.

Return value icin epilogue return pointer'ini cleanup oncesi stack slot'unda
saklar, ref cleanup tamamlandiktan sonra `rax`'a geri yukler. Return edilen
deger local `metin` ise `paylas` emit edilir; boylece function-scope cleanup
local'i birakirken caller'a donen referans canli kalir. Owned concat return
degeri zaten yeni refcount=1 nesne oldugu icin retain edilmeden caller'a
gecer.

If/else branch'lerinde normal akisla branch sonuna ulasilirse, o branch'in
ust seviye `metin` local'leri ters sirayla `birak` edilir. Erken `return`
aktif nested scope'lari temizledikten sonra fonksiyon epilogue'una atlar.
Loop body scope'u da normal iterasyon sonunda temizlenir; `kır` ve `devam`
akislarinda yalnizca cikilan loop'un aktif scope'lari temizlenir, dis loop
scope'lari canli kalir.

## Runtime Hatalari

Native MVP sifira bolme icin interpreter'a benzer kontrollu hata davranisi uretir. Kodgen bu durumda `anadil_runtime_panic` helper'ini cagirir:

```text
Anadil runtime hatasi: Sifira bolme hatasi
```

Bu durumda executable Enter bekledikten sonra Windows `ExitProcess(1)` ile biter. Kaynak satir/sutun bilgisi su an native executable icine gomulmez; IDE entegrasyonu icin compile-time lexer/parser/semantic hatalari CLI tarafinda caret'li diagnostic ve `kontrol --json` ile structured diagnostic olarak kalir.

## Runtime Helper ABI

Compiler kullanici kodundan dogrudan platform API'si veya C runtime cagrisi uretmek yerine kendi runtime helper'larini cagirir:

```text
anadil_runtime_print_sayi(rcx=sayi)
anadil_runtime_print_metin(rcx=cstr_ptr)
anadil_runtime_print_metin_nesne(rcx=metin_obj_ptr)
anadil_runtime_print_mantik(rcx=0/1)
anadil_runtime_strcmp(rcx=left_ptr, rdx=right_ptr) -> eax
anadil_runtime_metin_uzunluk(rcx=metin_obj_ptr) -> rax
anadil_runtime_metin_esit(rcx=left_obj_ptr, rdx=right_obj_ptr) -> eax 0/1
anadil_runtime_metin_birlestir(rcx=left_obj_ptr, rdx=right_obj_ptr) -> rax
anadil_runtime_tahsis(rcx=data_size, rdx=tip_id) -> rax=data_ptr
anadil_runtime_paylas(rcx=data_ptr) -> void
anadil_runtime_birak(rcx=data_ptr) -> void
anadil_runtime_wait_before_exit()
anadil_runtime_panic(rcx=message_ptr) -> process exit 1
```

Bu helper'lar program assembly'sinden ayri bir cached runtime library olarak linklenir. Runtime I/O, bekleme ve process sonlandirma Windows `kernel32` API'lerine baglidir; C runtime import'u artik native executable link hattinda gerekli degildir.
V0.2 baslangicinda heap primitive sembolleri de runtime'a eklendi; mevcut
compiler henuz bu primitive'leri emit etmez, dinamik `metin`/`dizi`/`yapi`
fazlari icin ABI hazirligi olarak tutulur.
Length-prefixed `metin` nesnesi icin hazirlanan helper'lar native backend
tarafindan static literal yazdirma ve karsilastirma icin kullanilir. Eski
C-string helper'lari runtime'in kendi hata ve `mantik` metinleri icin ic
uyumluluk yuzeyi olarak kalir.

## Runtime Library Paketleme

Anadil runtime'i her `derle` cagrisinda yeniden assemble edilmez. Compiler
runtime'i ayri bir `.lib` dosyasi olarak cache'ler ve programlari bu
kutuphane ile linkler.

### Dosya Yerlesimi

| Yol | Aciklama |
|---|---|
| `runtime/anadil_runtime.asm` | Tek dogruluk kaynagi olan runtime asm modulu. Repo icinde versiyonlanir. |
| `target/native-runtime/anadil_runtime.obj` | Cache'lenen runtime object file. `ml64` ciktisi. |
| `target/native-runtime/anadil_runtime.lib` | Cache'lenen runtime library. `lib` ciktisi. Programa link edilen artifact. |
| `target/native-runtime/anadil_runtime.lock` | `mkdir`-bazli cache lock klasoru. Paralel build'leri serilemek icin. |

### Build Adimlari

Compiler `derle` cagrisinda su sirayi takip eder:

1. Cache lock'unu al (`mkdir target/native-runtime/anadil_runtime.lock`).
   Klasor zaten varsa baska bir build koşuyor demektir; 25 ms araliklarla
   200 deneme yapilir (~5 sn timeout).
2. `runtime/anadil_runtime.asm` mtime'i `anadil_runtime.obj` mtime'inden
   yeniyse veya `obj` yoksa `ml64 /c /Fo<obj> <asm>` ile yeniden assemble
   edilir.
3. `anadil_runtime.obj` mtime'i `anadil_runtime.lib` mtime'inden yeniyse
   veya `lib` yoksa `lib /OUT:<lib> <obj>` ile library yeniden uretilir.
4. Lock klasoru `Drop` ile silinir; yarisli paralel build sonraki adima
   gecer.

Cache temiz ve kaynak `.asm` degismediyse hicbir `ml64`/`lib` cagrisi
yapilmaz; sadece program `.obj`'si uretilir ve dogrudan linkleme yapilir.

### Linker Cagrisi

Program object'i ve cached runtime library'si su komutla birlestirilir:

```text
link /NOLOGO /SUBSYSTEM:CONSOLE /ENTRY:main /OUT:<exe>
     <program.obj>
     target/native-runtime/anadil_runtime.lib
     kernel32.lib
```

Link satirinda **yalnizca** Anadil runtime library ve `kernel32.lib`
gozukur. Eski hattaki `msvcrt.lib`, `ucrt.lib`, `vcruntime.lib` ve
`legacy_stdio_definitions.lib` artik gerekli degildir; eklenmesi
istenmeyen davranis olarak kabul edilir.

### Build Tool Gereksinimleri

`derle` icin PATH icinde veya Visual Studio Build Tools `vcvars64.bat`
icinden ulasilabilir olmasi gerekenler:

- `ml64` (MASM)
- `lib` (Library Manager)
- `link` (Microsoft linker)

Hicbiri PATH icinde bulunamazsa compiler `vcvars64.bat`'i otomatik bulup
ucunu birden ayni shell ortamindan cagirir. Ucu de bulunamazsa
diagnostic stage `native` ile su mesaji raporlar:

```text
Native derleme icin Visual Studio Build Tools C++ araclari gerekli.
`ml64`, `link`, `lib` veya `vcvars64.bat` bulunamadi.
```

### Cache Invalidation Davranisi

Cache su olaylar disinda mtime'a guvenir:

- Lock acquire timeout: tek mesaj olarak `Native runtime cache lock
  beklerken zaman asimi` ile diagnostic'e duser.
- Mtime okunamazsa (izinler, silme yarisi vb.): cache invalid kabul
  edilir, yeniden uretilir.
- `runtime/anadil_runtime.asm` repo'da yoksa derleme hata ile durur:
  `Anadil runtime assembly dosyasi bulunamadi`.

Cache'i elle sifirlamak icin `target/native-runtime/` klasorunu silmek
yeterlidir; bir sonraki `derle` cagrisinda runtime yeniden uretilir.

### Test Kapsami

`tests/native_examples.rs` ve `tests/native_edge_cases.rs` `derle` komutunu
gercek `ml64`/`lib`/`link` zincirinde calistirir; cache hem ilk yaratim
hem reuse senaryosunu kapsar. `tests/cli_diagnostics.rs` runtime
artifact'leri olmadan da CLI hata yolunun calistigini dogrular.

## Memory Management

Su an native backend'de heap allocation yoktur.

Bu nedenle:

- Garbage collector yoktur.
- Reference counting helper'lari vardir; compiler yalnizca ilk MVP olarak
  void fonksiyon ust seviye `metin` local cleanup'i emit eder.
- Manual `free`/`delete` modeli yoktur.
- String literal'lar static length-prefixed `.data` nesneleri olarak yasar.
- `sayi`, `mantik` ve local `metin` referanslari stack slot'larda tutulur.

Bu model mevcut V0.1 dil alt kumesi icin yeterlidir. V0.2 branch'inde
`metin + metin` heap allocation kullanmaya basladi; dizi, struct ve
otomatik RC cleanup icin heap modeli genisletilecektir.

Karar belgesi: [memory_model.md](memory_model.md)

Onerilen V0.2+ bellek modeli reference counting uzerine kuruludur:

- `anadil_runtime_tahsis`
- `anadil_runtime_paylas`
- `anadil_runtime_birak`
- length-prefixed heap/static `metin`
- daha sonra `dizi`, `yapi` ve `zayif<T>`

V0.1 icin GC, manual `free/delete`, heap allocator veya RC emit hedeflenmez.

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
- `metin + metin` disinda runtime metin uretimi yoktur.
- Dinamik metinler icin otomatik `birak` emit'i fonksiyon cikisi, if/loop
  scope cikisi ve owned/static RHS ile guvenli `metin` assignment replacement
  icin vardir.
- Local `metin` paylasimi, user-defined fonksiyona local `metin` arguman
  gecisi ve local `metin` return degeri icin `paylas` emit edilir.
- If/else branch ve loop body scope'larindaki `metin` local'leri normal
  cikista ve ilgili erken akis cikislarinda `birak` edilir.
- Native runtime hatalari tek satir `Anadil runtime hatasi: ...` formatindadir, ancak henuz kaynak satir/sutun bilgisi tasimaz.
- Optimizasyon su an yalnizca typed AST uzerinde sabit katlama ve basit
  cebirsel sadelestirme seviyesindedir; IR/CFG tabanli optimizer yoktur.
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
