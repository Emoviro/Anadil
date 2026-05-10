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

## Su Ana Kadar Yapilanlar

### Dil Cekirdegi

- Lexer, parser, semantic analiz ve typed AST hatti kuruldu.
- Interpreter mevcut dil alt kumesini calistiriyor.
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
- Editor, dosya explorer, build paneli ve interpreter/native karsilastirma
  akislari uzerinde calisildi.
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
5. README ve native compiler dokumanlarini son kapsamla hizalamak.

Bu isler bittikten sonra compiler MVP V0.1 icin sabit kabul edilebilir.

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
Anadil V0.1 = lokal IDE + interpreter + native Windows executable compiler
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

