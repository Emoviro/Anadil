# Anadil Runtime Platform Soyutlamasi

Bu kisa tasarim notu, Anadil native runtime'inin Windows-spesifik
katmanini ilerideki Linux/macOS hedefleri icin nasil soyutlayabilecegini
ozetler. V0.1 hedefi degildir; karari kayitli tutmak ve ileride buyuk
refactor'a girmeden ekleme yapilabilmesi icin yazilmistir.

## Amac

Su anki runtime (`runtime/anadil_runtime.asm`) Windows `kernel32` API'lerine
dogrudan bagli. Bu sayede C runtime kaldirildi ve native `.exe` cikti
hattinin tek dis bagimliligi `kernel32.lib`. Ileride Linux/macOS hedefi
acilirsa, ayni runtime hatti syscall ABI'lari uzerinden yeniden uretilebilir
olmali; bunun icin runtime'in Windows'a ozel kismi acikca isaretli ve
yerinde degisken olmali.

## Mevcut Windows Yuzeyi

Tum Win32 bagimliligi su dort fonksiyon ve iki sabittir:

| Fonksiyon / Sabit | Kullanim |
|---|---|
| `GetStdHandle(dwStdHandle)` | stdin/stdout handle'i alir |
| `STD_INPUT_HANDLE` (-10) | stdin secimi |
| `STD_OUTPUT_HANDLE` (-11) | stdout secimi |
| `WriteFile(hFile, lpBuffer, nNumberOfBytesToWrite, lpNumberOfBytesWritten, NULL)` | byte yazimi |
| `ReadFile(hFile, lpBuffer, nNumberOfBytesToRead, lpNumberOfBytesRead, NULL)` | tek byte input (Enter bekleme) |
| `ExitProcess(uExitCode)` | runtime hata cikisi |

Cevirilen fonksiyonel yuzey daha kucuk:

```text
- write_bytes(buf, len) -> void
- read_one_byte() -> void
- process_exit(code) -> never
- get_stdin_handle() -> opaque
- get_stdout_handle() -> opaque
```

Geriye kalan tum runtime mantigi (sayi-to-decimal cevirim, byte byte
metin karsilastirma, integer abs, panic mesaji yazma) **platform
bagimsizdir** ve oldugu gibi tasinabilir.

## Soyutlama Prensibi

Runtime'i iki katmana ayirmak yeterlidir:

```text
runtime/anadil_runtime.asm           (platform bagimsiz)
runtime/anadil_runtime_windows.asm   (Win32 syscalls)
runtime/anadil_runtime_linux.s       (Linux syscall ABI; gelecek)
runtime/anadil_runtime_macos.s       (macOS syscall ABI; gelecek)
```

`anadil_runtime.asm` icindeki ortak helper'lar (yazi formatlama,
karsilastirma, panic) sadece soyut platform fonksiyonlarini cagirir:

```text
extrn anadil_platform_write_bytes:proc
extrn anadil_platform_read_byte:proc
extrn anadil_platform_exit:proc
extrn anadil_platform_get_stdin:proc
extrn anadil_platform_get_stdout:proc
```

Her platform `.asm`/`.s` dosyasi bu sembollerin somut ozelligini
saglar:

- Windows: `GetStdHandle` + `WriteFile`/`ReadFile` + `ExitProcess`
  uzerinden.
- Linux: `syscall` instruction ile `write`(1), `read`(0),
  `exit_group`(231) syscall numaralari uzerinden.
- macOS: `syscall` instruction ile BSD numaralari (write=4, read=3,
  exit=1) uzerinden.

Build hatti host platforma gore dogru `_<platform>.{asm,s}` dosyasini
secip ortak `anadil_runtime.asm` ile birlestirir. `src/main.rs`
icindeki `ensure_runtime_lib` fonksiyonuna platform secimi eklenir;
diger paketleme adimlari (cache, lock, mtime) ayni kalir.

## Asm-Vs-Rust Soru Isareti

Iki yaklasim mumkun:

### Yaklasim A: Tum platformlar icin ayri assembly

- Avantaj: hicbir bagimlilik yok, runtime kucuk kalir, mevcut MASM
  modeline simetrik.
- Dezavantaj: ucuncu syntax (GAS/AT&T) tanimak gerek; her syscall
  numarasi platform basina manuel.

### Yaklasim B: Platform katmani icin minimal Rust crate

- Avantaj: Rust `std::os` modulleri syscall'lari soyut sunar;
  `core::arch::asm!` ile gerekirse syscall'a inilir; tek tip Rust
  derleme adimi.
- Dezavantaj: Rust runtime'in panic/abort yolu C runtime'a dokunabilir;
  `#![no_std]` + `panic = "abort"` profil disiplini gerek.

V0.1 sonrasi ilk Linux/macOS denemeleri icin **Yaklasim A** daha
guvenli; mevcut "C runtime'siz native binary" hedefini bozmaz. Yaklasim
B ileride opsiyonel hale gelebilir.

## Linker Tarafindaki Etki

Windows hedefinde linker satiri ayni kalir:

```text
link ... program.obj anadil_runtime.lib kernel32.lib
```

Linux hedefinde `lib` yerine `ar` ile `.a` arsiv uretilir; `link`
yerine `ld` cagrisi yapilir; `kernel32.lib` referansi yoktur.

```text
ld -o program program.o libanadil_runtime.a
```

macOS hedefinde benzer ama `ld64` ve `-lSystem` (libSystem.dylib
syscall stub'lari icin) bagimliligi olur. macOS syscall'larini
dogrudan kullanmak da mumkun ancak Apple resmen onermez.

## Test Stratejisi

Cross-platform implementation ilk eklendiginde:

- `tests/native_examples.rs` her hedef icin platform-uygun derleyici
  zincirini bulup atlama davranisi gosterir (Visual Studio yoksa
  Windows test atlanmasi gibi).
- Yeni eklenen platforma ozel runtime test seti ekleyerek
  `cargo test --target <triple>` calistirilir.

## V0.1 Karari

V0.1 yalnizca Windows x64 hedefler. Bu soyutlama V0.1 kapsami
disindadir; uygulanmasi icin kisa adimlar:

1. `anadil_platform_*` semboller tanimlanir; mevcut runtime ic
   cagrilarini bu sembollere yonlendirir.
2. Mevcut Win32 cagrilari `runtime/anadil_runtime_windows.asm`
   dosyasina tasinir.
3. `src/main.rs` `ensure_runtime_lib` Windows kolu mevcut davranisi
   korur.
4. Ortak runtime ile platform asm'inin ayni `.lib`'e arsivlendigi
   regression testi eklenir.

Linux/macOS ekleme kararlari ileri tasarim notlarina baglidir; bu belge
yalnizca yapinin onunde engel olmadigini gosterir.
