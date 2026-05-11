# Anadil Bellek Modeli

Bu belge Anadil'in bellek yonetim modelini, niye bu yolun secildigini, runtime
ve compiler tarafinin somut sorumluluklarini, ve dikkat edilmesi gereken
tuzaklari toplar. Hedef: gelecekteki gelistirme oturumlari icin tek dogruluk
kaynagi olmak.

## V0.1 Kapsami

V0.1 hedefi heap/GC implementasyonu degil, mevcut native compiler MVP'nin
bellek sinirlarini acikca tanimlamaktir.

V0.1'de:

- `sayi` ve `mantik` register/stack degerleridir.
- `metin` sadece compiler'in `.data` segmentine koydugu statik literal veya
  runtime sabitidir.
- Dinamik heap allocation yoktur.
- Garbage collector yoktur.
- Reference counting runtime'i yoktur; `paylas`, `birak`, `tahsis`
  fonksiyonlari V0.1 disidir.
- `dizi`, `yapi`, dinamik `metin` birlestirme ve heap string V0.1 disidir.
- Native runtime sadece I/O, metin karsilastirma, program bekleme ve hata
  cikisi helper'larini saglar.

V0.1 icin bu karar bilincli olarak dar tutulur: compiler omurgasi,
runtime `.lib` modeli ve `.exe` uretme akisi sabitlenmeden heap/RC
implementasyonuna girilmez.

## 1. Karar Ozeti

V0.2+ heap modeli icin Anadil **referans sayma (reference counting, RC)**
modelini hedefler. Tepe notlar:

- Heap nesneleri header'inda bir `refcount` tasir.
- Compiler her atama, parametre gecisi ve donus icin uygun yerlere
  `paylas` (refcount++) ve `birak` (refcount--) cagrilari uretir.
- **Last-use analizi** ile cogu paylas/birak cifti elenir; sahiplik tasinir.
- Runtime helper'lar `runtime/anadil_runtime.asm` icindeki ayni modulde
  yasar; hicbir C runtime bagimliligi yoktur.
- **RC Faz 1 tek-thread varsayimi ile non-atomic sayac kullanir.** Threading
  geldiginde sadece runtime PROC'lari `lock`'a gecer; compiler emit'i ve
  ABI degismez (bkz. Bolum 9).
- **Cycle handling RC Faz 1'e GIRMEZ.** Ilk surumde cycle yaparsan leak
  olusur, dokumantasyonla uyari verilir. `zayif<T>` ve weak counter
  RC Faz 7'de eklenir; o noktada header layout'u 16 byte'tan 24 byte'a
  genisler (kabul edilen ABI break, henuz shipped binary yok).

Bu kararin gerekceleri:

- **Native compile ile uyumlu**: GC iskelesi gerekmez; RC primitive'leri
  runtime'a kademeli eklenir.
- **Deterministik yikim**: kullanici "bu nesne ne zaman olur?" sorusuna
  net cevap verebilir.
- **Egitim degerine uygun**: Turkce terimler dogal oturur, kullanici
  `tahsis()` / `birak()` yazmak zorunda degildir.
- **Olcekli isgucu**: RC + heap + metin/dizi/yapi hatti tek parca degil,
  asagidaki V0.2+ fazlara bolunerek uygulanir.

## 2. Mimari Resim

```
.ana kaynak
  -> Lexer / Parser / Semantic analiz / Typed AST
  -> Native backend (src/native.rs)
       * V0.2+ icin RC cagrilarini emit eder
       * V0.2+ icin last-use analizi calistirir
  -> Generated user.asm  (ml64) -> user.obj
                                                ─┐
       runtime/anadil_runtime.asm  (ml64) -> rt.obj
                                              -> lib  -> anadil_runtime.lib
                                                ─┘
  -> link user.obj + anadil_runtime.lib + kernel32.lib  -> .exe
```

Compiler RC mantigini, runtime ise butun heap operasyonlarini bilir. Bu
ayrim disiplinli kalmali, ama compiler ile runtime arasinda **acikca
yazilmis bir ABI sozlesmesi** vardir; compiler bu sozlesmenin dortagi
sayisini varsayar, daha fazlasini degil.

### ABI Sozlesmesi (Compiler ↔ Runtime)

Compiler'in **bilmek zorunda oldugu** seyler:

1. **Header layout ve offset'ler**:
   - `[ptr - 16]` → 8 byte refcount.
   - `[ptr - 8]`  → 8 byte tip_id.
   - RC Faz 7'de header 24 byte'a buyur (weakcount eklenir): `[ptr - 24]`
     refcount, `[ptr - 16]` weakcount, `[ptr - 8]` tip_id.

2. **Runtime fonksiyon imzalari** (Bolum 5 ve sonrasinda dokumante):
   - `anadil_runtime_paylas(rcx=ptr) -> void`
   - `anadil_runtime_birak(rcx=ptr) -> void`
   - `anadil_runtime_tahsis(rcx=size, rdx=tip_id) -> rax=ptr`
   - vb.

3. **Tip ID degerleri**: `METIN`, `DIZI`, `YAPI_X` icin atanmis sabit
   sayilar. Compile-time'da derlenecek bir tablo.

4. **Statik literal sentinel**: refcount alaninda `0x7FFFFFFFFFFFFFFF`
   degeri statik (HeapFree edilmemesi gereken) anlamina gelir. Bu
   degeri **hem compiler** statik literal emit ederken yazar, **hem
   runtime `birak`** kontrol ederek atlar.

Compiler'in **bilmek zorunda OLMADIGI** seyler:

- HeapAlloc/HeapFree cagrisinin nasil yapildigi.
- Header'in hangi pencere fonksiyonuyla allocate edildigi.
- Destructor dispatch tablosunun ic veri yapisi (sadece tip_id'nin
  benzersiz oldugunu bilir; tablonun map mi switch mi oldugunu bilmez).
- Iteratif destructor worklist'in implementasyonu.
- Atomic vs non-atomic instruction secimi (Bolum 9).

### Sozlesme degisiklikleri

Header offset'i veya runtime imzasi degisirse bu **breaking change**'tir
ve hem compiler hem runtime ayni patch'te guncellenir. Boyle degisiklikler
bu belgeye eklenir; layout versiyon numarasi tasimaz, sadece "mevcut
faz" bilgisi yeterli (Anadil shipped binary tutmuyor, ABI bozulabilir).

## 3. Tip Kategorileri

V0.2+ heap modeliyle Anadil tipleri iki sinifa ayrilir:

| Kategori | Ornekler | Saklama | RC dahil mi |
|---|---|---|---|
| Value (deger) | `sayi`, `mantik` | Stack veya register | Hayir |
| Reference (gonderge) | `metin`, `dizi`, `yapi`, `zayif<T>` | Heap | Evet |

Compiler her ifade icin tipi bildiginden, RC cagrisi gerekip gerekmedigi
derleme sirasinda belirlenir. Value tipler icin paylas/birak ASLA
emit edilmez.

V0.1'de `metin` henuz heap reference degildir; compiler'in `.data`
segmentinde tuttugu NUL-terminated statik literal pointer'i olarak
calisir. V0.2+ migration'i tamamlandiginda `metin` kullanicinin gozunden
tek tip kalir, ama runtime temsili length-prefixed heap/static reference
modeline gecer.

V0.2 branch'inde bu gecis icin runtime ABI hazirligi basladi:
`anadil_runtime_metin_uzunluk`, `anadil_runtime_print_metin_nesne` ve
`anadil_runtime_metin_esit` length-prefixed nesne layout'unu bekler. Native
backend henuz bunlari emit etmez; V0.1 uyumlu NUL-terminated literal yolu
korunur.

## 4. Heap Nesne Layout'u

Tum heap nesneleri ayni 16 byte header ile baslar:

```
Offset  Boyut  Alan
------  -----  ----
-16      8     refcount (u64, RC Faz 1 non-atomic)
 -8      8     tip_id   (u64)
  0      ...   nesne verisi
```

Kullanicinin elindeki pointer **veriyi** gosterir, header'i degil. Bu
sayede `mov rax, [ptr]` gibi alan erisimi offset hesabi gerektirmez.
Header'a erismek icin `[ptr - 16]` yeterlidir.

Tip ornekleri:

### `metin`

**Hedef layout (RC Faz 4 sonu):**

```
[refcount][tip_id=METIN][len: u64][bytes: u8...]
                        ^ ptr (kullaniciya verilir)
```

`uzunluk(m)` builtin'i `mov rax, [m]` ile len'i okur (ptr len'i
gosteriyor; bytes hemen ardindan).

**Mevcut runtime ile catisma:**

`runtime/anadil_runtime.asm` su an `metin`'i NUL-terminated kabul ediyor:

- `anadil_runtime_print_metin` → `anadil_runtime_write_cstr`'i cagiriyor
  → byte byte `0`'a kadar tarayip uzunluk hesapliyor.
- `anadil_runtime_strcmp` byte byte `0`'a kadar karsilastiriyor.
- `text_dogru`, `text_yanlis` sabit byte dizileri NUL-terminated.

Length-prefixed layout'a gecince bu fonksiyonlarin **hepsi degismek
zorunda**. Bu compiler ve runtime arasinda esleştirilmis bir gecistir,
ayni anda yapilmali. RC Faz 4 ozel olarak bu migration'a ayrilmis. Detay
icin Bolum 13.4'e bakiniz.

NUL-terminated'tan length-prefixed'e gecmek su kazanimlari getirir:

- `uzunluk(m)` O(1) (su an O(n) tarama).
- `metin` icinde NUL byte tasiyabilir (binary-safe).
- strcmp/strlen tarzi tarama bug'lari (out-of-bounds read) yok.

Goturecekleri:

- Sabit byte dizileri (`text_dogru`, `text_yanlis`, runtime hata
  mesajlari) header + len ile kaynak kodda yazilmasi gerekir.
- `anadil_runtime_print_metin`, `anadil_runtime_strcmp`,
  `anadil_runtime_write_cstr` (kaldirilir veya icsel kullanim icin
  kalir) yeniden yazilir.

### `dizi`

```
[refcount][tip_id=DIZI][len: u64][cap: u64][eleman_pointerlari...]
                       ^ ptr
```

Elemanlar **kendileri RC nesneleri** ise burada pointer'lari tutulur,
her ekleme/cikma kendi paylas/birak cagrilarini tetikler.

### `yapi`

```
[refcount][tip_id=YAPI_X][field0][field1]...
                         ^ ptr
```

`tip_id` her struct icin uniktir, destructor dispatch tablosunu
indeksler.

## 5. Runtime Primitifleri

`runtime/anadil_runtime.asm` icine eklenecek fonksiyonlar:

### `anadil_runtime_tahsis`

```
input:  rcx = data_size (header haric)
        rdx = tip_id
output: rax = data pointer (header'in 16 byte sonrasi)
```

`HeapAlloc(GetProcessHeap(), 0, data_size + 16)` cagirir, header'i
doldurur (refcount=1, tip_id=rdx), data ptr'yi doner.

### `anadil_runtime_paylas`

```
input:  rcx = data pointer
output: yok
```

```asm
inc qword ptr [rcx - 16]
ret
```

RC Faz 1'de tek-thread varsayimiyla non-atomic kullanilir. Threading
dile girdiginde bu instruction `lock inc` olur; fonksiyon imzasi degismez.

### `anadil_runtime_birak`

```
input:  rcx = data pointer
output: yok
```

```asm
dec qword ptr [rcx - 16]
jnz .alive
; refcount sifira indi → yikim
mov rdx, [rcx - 8]              ; tip_id
call destructor_dispatch        ; alt nesneleri birak
sub rcx, 16                     ; header'a sar
mov rdx, rcx
mov rcx, GetProcessHeap_handle
xor r8, r8
call HeapFree
.alive:
ret
```

### `anadil_runtime_kopyala`

```
input:  rcx = data pointer
output: rax = derin kopya pointer
```

`tip_id`'ye gore dispatch eder, yeni heap nesnesi yaratir, alt nesneleri
de kopyalar (recursive).

### `destructor_dispatch`

```
input:  rcx = data pointer
        rdx = tip_id
```

Tip_id'ye gore tablo bakar, ilgili destructor'u cagirir. Her tip
icin "alt referanslari birak" prosedurudir. Iteratif worklist ile
calisir (asagida 10. bolume bakiniz).

## 6. Compiler Emit Kurallari

Compiler her syntactic noktada belirli kurallar isletir:

### 6.1 Atama (`x = y`)

```ana
y: metin = "selam"
x: metin = y
```

`x = y`:
- Once `paylas(y)` (yeni referans dogdu).
- Sonra mevcut `x` varsa `birak(x_eski)` (eski deger oldu).
- Sonra `x = y` mov.

Ozel durum: `x` yeni tanimlanmissa eski deger yoktur, birak atilir.

### 6.2 Parametre Gecisi

```ana
Yazdir(s: metin) { yazdir(s) }
Ana() {
    m: metin = "merhaba"
    Yazdir(m)
}
```

Cagiran tarafta:
- `paylas(m)` → callee'ye giderken refcount artar.

Cagrilan tarafta:
- Fonksiyon kapsami sonunda `birak(s)`.

Sonuc: `m` cagri sonunda hala canli, callee yerel referansi temizlendi.

### 6.3 Donus Degeri

```ana
Olustur() -> metin {
    s: metin = "yeni"
    don s
}
```

- Kapsamda `s` yaratilirken refcount=1.
- `don s` ile sahiplik caller'a tasinir; **`birak(s)` ATILMAZ**.
- Caller'da don deger yeni bir baglama atanir veya kullanilir; gerekirse
  `birak` orada eklenir.

Bu "return is move" kurali compiler'in en kritik yerlerinden biridir.

### 6.4 Kapsam Cikisi

Her blok `{ ... }` cikisinda, blokta yaratilan veya alinan referans
tipindeki yerel degiskenler icin `birak` cagrisi eklenir. Sira ters:
**son tanimlanan once birakilir** (RAII benzeri).

### 6.5 Kosullu Yikim

```ana
eger kosul {
    s: metin = "evet"
    yazdir(s)
}
// burada s artik yok
```

`s` sadece if-true dalinda yasar. Her iki dal icin de o dalin sonunda
`birak` eklenir; merge noktasinda yerel referans yoktur.

### 6.6 Atama Operatoru ve Geri Sayim Sirasi

```ana
x = y
```

Dogru sira:
1. `paylas(y)` ← Yeni sahiplik.
2. `tmp = x_eski`
3. `x = y` (mov)
4. `birak(tmp)` ← Eski deger.

**Bu sirada KESINLIKLE birak'i once yapmamak**. Ornegin `x = x.alt`
durumu: `x.alt`'i once birakirsan, `x` yikilir, `alt` da yikilir,
sonra atadigin pointer dangling olur.

## 7. Last-Use Optimization

Saf RC her atama icin iki cagri uretir; bu pratikte cok pahali. **Last-use
analysis** ile pek cogu silinir.

### 7.1 Algoritma

Compiler her temel blok icin geriye-dogru bir gecisle her degisken
icin "son kullanim noktasini" isaretler. Last-use noktasinda
*sahiplik tasiniyorsa* (yani degisken bir baska yere atanyor veya
fonksiyona veriliyor ve sonra erisilmiyor), paylas/birak cifti elenir.

### 7.2 Ornek

```ana
isim: metin = "Ali"
selam: metin = isim       // isim son kez burada kullaniliyor
yazdir(selam)
```

Saf RC:
- isim: tahsis (rc=1)
- paylas(isim) → rc=2
- selam = isim
- yazdir cagrisi
- birak(selam) → rc=1
- birak(isim) → rc=0, free

Last-use sonrasi:
- isim: tahsis (rc=1)
- selam = isim (mov, sahiplik tasindi)
- isim artik gecersiz isaretlendi
- yazdir cagrisi
- birak(selam) → rc=0, free

Bir paylas + bir birak silindi. Sayac islemi yarisi gitti.

### 7.3 Karmasik Durumlar

- **Donguler**: dongu icindeki son kullanim her iterasyonda gecerlidir,
  paylas/birak korunmali (veya iterasyon disinda taslanmali).
- **Sube'li akis**: bir dalda son-kullanim, digerinde degil → en kotu
  durumu varsay, paylas/birak koru.
- **Esleme/destructuring**: ileride struct field destructuring eklenirse
  her field icin ayri last-use analizi gerekir.

Ilk surumde basit, doğrusal akis icin yapan bir analiz yeterlidir.
Cikarim: en sik kullanim (yerel, son-kullanimda atanan) yakalansin.

## 8. Cycle Problemi ve Cozumler

### 8.1 Cycle Nedir

```ana
yapi Dugum {
    veri: sayi,
    sonraki: Dugum,
}

a: Dugum = Dugum{veri: 1}
b: Dugum = Dugum{veri: 2}
a.sonraki = b   // a → b (b.rc=2)
b.sonraki = a   // b → a (a.rc=2)
```

Kapsam cikisinda `birak(a)` ve `birak(b)`. Ikisi de sayaci 1'e dusurur,
sifir olmaz. Her ikisi de hayatta, ama disardan ulasilamaz → bellek
sizintisi.

### 8.2 RC Faz 1'de Cycle Handling YOK

**Karar**: ilk RC surumu (RC Faz 1-6) cycle problemini cozmez. Kullanici cycle
yaratirsa nesneler leak olur. Bu kabul edilen bir kisitlama; faz
roadmap'inin guvenlik fasinda (RC Faz 8 testleri) cycle leak'i bir test
case olarak gosterilir, dokumantasyonda **bilinen sinir** olarak
duyurulur.

RC Faz 1'de:
- Header 16 byte (refcount + tip_id), weakcount yok.
- `zayif<T>` tipi yok.
- Self-referencing struct (`yapi Dugum { sonraki: Dugum }`) leak yapar.

Bu tasariminda mantikli; cunku:

1. Anadil'in mevcut tipleri (`sayi`, `mantik`, `metin`) cycle uretmiyor.
2. Faz 5'te struct geldikten sonra cycle olusturulabilir, ama implicit
   cycle pratikte nadir; programcinin bilerek yapmasi gerekir.
3. RC Faz 7'de cycle cozumu eklenir, ABI break (header 24 byte'a buyur)
   kabul edilebilir bir maliyettir.

### 8.3 Cozum: `zayif<T>` (RC Faz 7+)

`zayif<T>` tipi, refcount artirmadan bir nesneye gondergedir. Sahiplik
yok, sadece "bakma izni" var.

```ana
yapi Dugum {
    veri: sayi,
    sonraki: Dugum,
    onceki: zayif<Dugum>,
}
```

`onceki` field'i `Dugum`'a baglidir ama refcount'a katki vermez. Cycle
olusmaz.

#### Runtime davranisi

Zayif gonderge dereference edilirken kontrol gerekir:

```ana
eger eski := w.guclendir() {
    // eski hala canli, normal Dugum gibi kullan
} degilse {
    // eski olmus, yapacak bir sey yok
}
```

`guclendir()` (analog: Rust'in `Weak::upgrade`) zayif gondergeyi guclu
gondergeye yukseltmeyi dener. Nesne hala canliysa refcount artar ve
guclu gonderge doner. Yikilmissa `degilse` daline duser.

#### Implementation

Iki yaklasim:

**Yaklasim A (basit):** Ayri zayif sayac.

Header genisletilir:
```
[refcount][weakcount][tip_id] (24 byte)
```

`birak`'ta refcount sifira inerse data yikilir, ama header HeapFree
yapilmaz; weakcount sifira inerken yapilir. Zayif gonderge dereference'i:

```asm
test [refcount], 0
jz nesne_olmus
; canli, paylas et
inc [refcount]
```

**Yaklasim B (referans nesnesi):** Zayif gonderge ayri bir kucuk nesnedir,
"kontrol bloku" tutar. Apple'in ARC'i benzer.

Ilk surum icin **A** yeterli ve daha hafif.

### 8.4 Cycle Collector?

Python'un yaklasimi: belirli aralikla heap'i tara, ulasilamaz cycle'lari
bul. Karmasik (~1500-2000 satir), Anadil'in mevcut hedefleri icin
gereksiz.

**Karar**: hicbir RC fazinda planli degil. Kullanici `zayif` ile cycle'lari
elle yonetir. Ileride ihtiyac netleşirse RC Faz 8'in disinda modul olarak
eklenir.

## 9. Atomic Sayaclar ve Threading Hazirligi

### 9.1 Karar

**RC Faz 1 implementasyonu non-atomic baslar.** Cunku:

- Anadil bugun tam anlamiyla tek-thread (interpreter, native exe, hicbiri
  thread spawn etmiyor).
- Non-atomic `inc/dec` her cagrida 2-3 cycle; `lock inc/dec` uncontended
  durumda 15-25 cycle. Yaklasik 5-10x fark.
- Pratikte her atama paylas+birak ureteceginden, sayac islemi sicak
  yolun onemli bir parcasi olur. RC Faz 1'de gereksiz overhead'den kacinmak
  istiyoruz.

### 9.2 Threading geldiginde gecis plani

Threading dile girdiginde:

1. Runtime'da `paylas` ve `birak` `lock`'lu versiyonlarla degisir.
2. Bu **runtime-only degisiklik**; compiler emit'i farkli kalmaz.
3. ABI bozulmaz — program tipi (kullanim) ayni, sadece icteki instruction
   degisir. Yeniden derleme yeterli.

Yani "atomic'i bastan koy" kaygisi gerceksiz: sayac birim islemi runtime'in
icinde mahremdir, compiler tarafindan gorulmez. ABI yuzeyinde sadece
"cagri arayuzu" var, atomicity degil.

### 9.3 RC Faz 1 Implementasyonu

```asm
; paylas (RC Faz 1, non-atomic)
anadil_runtime_paylas PROC
    inc qword ptr [rcx - 16]
    ret
anadil_runtime_paylas ENDP

; birak (RC Faz 1, non-atomic)
anadil_runtime_birak PROC
    dec qword ptr [rcx - 16]
    jnz .alive
    ; ... yikim ...
.alive:
    ret
anadil_runtime_birak ENDP
```

Threading geldiginde sadece bu iki PROC icindeki `inc`/`dec` `lock inc`/
`lock dec` olur.

### 9.4 Tek-thread Varsayimi Yorumu

Bu karar, `runtime/anadil_runtime.asm` icinde acik bir yorum bandiyla
isaretlenmeli:

```asm
; NOT: RC Faz 1 tek-thread varsayimiyla non-atomic sayac kullaniyor.
; Threading dile girdiginde bu PROC'lar lock'lu versiyona gecer.
; Bkz Docs/memory_model.md, Bolum 9.
```

## 10. Iteratif Destructor

Naive destructor recursive'dir:

```
free(node):
    free(node.sonraki)   ; recursive!
    HeapFree(node)
```

100,000 elemanli linked list yikimi 100,000 derin stack call → stack overflow.

Cozum: **iteratif worklist**.

```asm
; Pseudo-asm
worklist (sabit boyutlu yerel buffer veya buyuyen heap dizi)
worklist.push(initial_ptr)

while worklist not empty:
    ptr = worklist.pop()
    dec [ptr-16]
    if not zero:
        continue
    ; tip_id'ye gore alt referanslari toplat
    for each alt_ref in destructor_table[tip_id]:
        worklist.push(alt_ref)
    ; nesneyi free et
    HeapFree(ptr - 16)
```

Bu kalip her dilde standart, runtime'da ~50-80 satir.

## 11. Statik String Literal'lar

Compile-time bilinen string literal'leri (`"merhaba"`) statik `.data`
segmentindedir, HeapAlloc edilmemistir. `birak` cagrisi `HeapFree`'ye
indirgerse statik adresi free etmeye calisir → crash.

### Cozum: "olumsuz" sayac

Statik literal'in header'i `refcount = 0x7FFFFFFFFFFFFFFF` (i64 max yarisi)
ile uretilir. `birak`:

```asm
mov rax, [rcx - 16]
cmp rax, 0x4000000000000000
jge .static_literal_skip      ; ust bit set ise statik, skip
dec [rcx - 16]
...
.static_literal_skip:
ret
```

Sayac asla anlamli sayilara inmez. Statik nesne icin paylas/birak no-op'tur.

Alternatif: tum literal'leri runtime baslangicinda heap'e kopyala.
Daha temiz ama startup maliyeti. Anadil icin **olumsuz sayac** yaklasimi
hafif ve yeterli.

`runtime/anadil_runtime.asm` icindeki mevcut `text_dogru` ve `text_yanlis`
sabit byte dizileri bu kategoride; suanki layout'a uydurulmasi gerekir.

## 12. Tuzaklar

Implementation sirasinda kacinilmasi sart hatalar:

### 12.1 Cift birak

Ayni pointer'a iki kez `birak` cagrisi → use-after-free.

**Onleme**: Compiler her yerel degisken icin TEK bir `birak` cikti
sirasinda emit eder. Last-use sonrasi degisken "dead" isaretlenir; ikinci
birak gelmez.

### 12.2 Birak Sirasinin Yanlisliği

```ana
x = x.alt    // YANLIS sirayla yapilirsa katastrof
```

Dogru sira: Yeni deger paylas → mov → eski deger birak. Atama operatoru
emit'i bunu garanti etmeli.

### 12.3 Static Olmayan Default Field'lar

```ana
yapi Cubuk { isim: metin = "varsayilan" }
```

Constructor her cagrida `"varsayilan"` literal'ini paylas eder. Statik
literal `birak`'i no-op oldugu icin guvenli, ama tutarli ele alinmali.

### 12.4 Self-Referencing Struct

```ana
yapi Dugum { sonraki: Dugum }     // tehlikeli
yapi Dugum { sonraki: zayif<Dugum> }  // guvenli
```

Compiler bu durumda warning verebilir; yasaklamak gerekmez ama dile
"zayif kullan" mesaji egitime uygun.

### 12.5 Donguler ve Iterator

```ana
dongu i: sayi = 0; i < dizi_uzunluk(d); i = i + 1 {
    e: metin = d[i]    // her iterasyonda paylas/birak
    yazdir(e)
}
```

Her iterasyon paylas + birak. Optimize: `e` borrow gibi davransa
(referans, sahiplik almasin) cifti silinebilir. Ileri optimizasyon, MVP
disi.

### 12.6 Recursive Structuras Builder

`metin + metin + metin + ...` ifadesi her ara sonucta yeni heap nesnesi
yaratir. Kullanici buyuk birlestirmelerde yavasligi gorur. Cozum: ileride
`metin_yapici` (StringBuilder) tipi. MVP'de `+` yeterli.

### 12.7 Heap Allocation Hatasi

`HeapAlloc` NULL donerse `anadil_runtime_panic` cagir. "Bellek tahsisi
basarisiz" mesaji ile programi sonlandir. Kullaniciya graceful try/catch
verilmez (Anadil exception'siz).

### 12.8 Async/Closure (Gelecek)

Closures heap allocation gerektirir; capture edilen degiskenler RC ile
sahiplenmeli. Bu konu gelecekte dile closure eklenince tasarlanir.

## 13. V0.2+ Yol Haritasi

Bu yol haritasi V0.1 compiler MVP tamamlandiktan sonra uygulanir. Fazlar
bagimsiz olarak isleyebilir ve test edilebilir; hicbiri V0.1 kapatma
kriteri degildir.

### RC Faz 1 — Runtime Primitifleri (~1 gun)

- `runtime/anadil_runtime.asm` icine ekle:
  - `anadil_runtime_tahsis`
  - `anadil_runtime_paylas`
  - `anadil_runtime_birak`
  - `anadil_runtime_kopyala_yuzeysel`
  - `destructor_dispatch` (tip tablosu yer tutucu)
- **16 byte header** (refcount + tip_id). Weakcount yok. Cycle handling yok.
- **Non-atomic sayaclar** (`inc/dec`). Threading geldiginde ayri bir fazda
  `lock`'a gecilir; bkz. Bolum 9.
- Iteratif destructor altyapisi.
- Statik literal "olumsuz sayac" markeri.
- Tahsis hatasinda panic.

### RC Faz 2 — Compiler RC Emit (~2 gun)

- `src/native.rs` icinde reference tipler icin paylas/birak emit.
- Atama, parametre gecisi, donus, kapsam cikis kurallari.
- Rust tarafinda yardimci: tip kategorisi (`is_ref_type(t)`).
- Henuz last-use optimizasyonu yok; saf RC ile baslar.

### RC Faz 3 — Last-Use Analizi (~2 gun)

- Bloklarda geriye-dogru yasam analizi.
- Last-use ve "sahiplik tasiniyor" durumlarini isaretle.
- Emit sirasinda paylas/birak cifti elenebiliyorsa el.
- Test: paylas/birak sayilarinin saf RC'ye gore dustugu unit testler.

### RC Faz 4 — `metin` NUL-Terminated'tan Length-Prefixed'e Migration (~2 gun)

Bu faz **tek atomik degisiklik** olarak ele alinmali; ne compiler ne
runtime arada tutarsiz kalabilir.

Yapilacaklar listesi:

1. Header + length-prefixed layout'u (Bolum 4) `runtime/anadil_runtime.asm`
   icinde sabit byte dizileri icin uygula:
   - `text_dogru`, `text_yanlis` her birinin `[ölümsüz_sayaç][tip_id=METIN][len][bytes]`
     formatina cevir.
   - Runtime hata mesajlari (`runtime_error_prefix` vb.) ayni sekilde.
2. Print yolunu degistir:
   - `anadil_runtime_write_cstr` ya kaldirilir ya da iç kullanim icin
     baska isimle kalir (NUL'a guven duymayan write_bytes versiyonu).
   - `anadil_runtime_print_metin` artik `mov rdx, [rcx]` ile len'i
     okur, `lea rdx, [rcx + 8]` ile bytes'a gecer, `write_bytes`
     cagirir.
3. Compare yolunu degistir:
   - `anadil_runtime_strcmp` len + memcmp tarzina cevrilir (veya yeni
     ad: `anadil_runtime_metin_esit`).
   - `native.rs` icinde `==`/`!=` emit'i bu yeni signature'i cagirir.
4. Compiler degisikligi:
   - `metin` literal'leri `.data`'da yeni layout'la emit edilir.
   - `anadil_runtime_metin_birlestir` (yeni runtime fn): iki metin alir,
     yeni heap nesnesi yaratir, ikisinin bytes'ini birlestirir, refcount=1.
   - `+` operatoru bu cagriyi uretir.
   - `uzunluk(metin)` builtin'i: `mov rax, [arg]`. Tek instruction.
5. Test:
   - Mevcut `tests/native_examples.rs` ve `tests/native_edge_cases.rs`
     metin operasyonlarini koru, layout'a guven `len[bytes]` olduguna
     dair regression test ekle.
   - `metin` icinde NUL byte (binary-safe test) bir test programi.
   - Bos `metin` ("") tahsis ve birak.

### RC Faz 5 — `yapi` MVP (~3 gun)

- Lexer/parser: `yapi Ad { field: tip, ... }`.
- Semantic: tip kaydi, field offset hesaplama.
- Native emit: `yeni_yapi(...)` ile heap'te yarat, field erişimi
  `mov [ptr + offset]`.
- Destructor tablosu: her yapi icin alt referans listesi.
- Test: gomulu yapi, recursive yapi (zayif ile).

### RC Faz 6 — `dizi` MVP (~2 gun)

- `dizi<T>` tipi, parser/semantic destegi.
- Runtime: `anadil_runtime_dizi_olustur`, `_ekle`, `_eriş`, `_uzunluk`.
- Bounds check: out-of-range → `anadil_runtime_panic("Dizi siniri")`.
- Destructor: tum elemanlari tek tek birak.
- Test: tahsis/free, eleman tipleri, bounds.

### RC Faz 7 — `zayif<T>` ve Cycle Cozumu (~2 gun, ABI break)

Bu faz **iki tarafi birden bozar**: header layout 16 → 24 byte buyur,
hem compiler hem runtime ayni patch'te guncellenir. Daha onceki tum
heap nesneleri yeni layout'la yeniden derlenmek zorundadir.

- Header layout: `[refcount][weakcount][tip_id]`, 24 byte.
- Compiler tarafinda offset'ler guncellenir: refcount `[ptr-24]`,
  weakcount `[ptr-16]`, tip_id `[ptr-8]`.
- Runtime fonksiyonlari:
  - `anadil_runtime_zayif_olustur(rcx=guclu_ptr) -> rax=zayif_handle`
  - `anadil_runtime_zayif_guclendir(rcx=zayif_handle) -> rax=guclu_ptr veya 0`
  - `anadil_runtime_zayif_birak(rcx=zayif_handle) -> void`
- `birak` fonksiyonu guncellenir: refcount=0 olunca data destruct edilir
  ama HeapFree weakcount=0 olana kadar ertelenir.
- Compiler: `zayif<T>` tipi destegi, parser/sema, dereference syntax
  (`w.guclendir()` veya benzeri).
- Test: cycle senaryosu (zayif olmadan leak gosterimi, zayif ile temiz),
  weak yikilan nesneye erisim, weakcount lifecycle.

### RC Faz 8 — Test ve Edge Case (~2-3 gun)

- Stress test: 1M nesne tahsis/free.
- Linked list yikimi (iteratif destructor dogrulama).
- Cycle senaryosu (zayif kullanmadan leak, zayif ile temiz).
- Statik literal birak no-op dogrulama.
- Multi-thread atomic sayac sanity (manuel).
- Mevcut `tests/native_examples.rs` ve `tests/native_edge_cases.rs`
  dosyalarini dizi/yapi/metin senaryolariyla genislet.

**Toplam:** Bu is V0.1 kapsami degildir. RC + heap + metin/dizi/yapi
hatti testlerle birlikte yaklasik 2-4 haftalik ayri bir V0.2+ calismasi
olarak ele alinmalidir.

## 14. Test Stratejisi

### Unit testler (Rust tarafi)

- Compiler last-use analizi: belirli AST'ler icin paylas/birak sayisi
  beklenenle esit mi.
- Tip kategorisi: hangi tipler reference, hangileri value.
- Destructor dispatch tablosu olusturma.

### Integration testler (cargo test)

- `tests/native_examples.rs`: dizi, yapi, dinamik metin ornek programlari
  hem interpreter hem native cikti uretiyor mu, ayni mi.
- `tests/native_edge_cases.rs`: derin nested struct, uzun linked list
  yikimi, cycle senaryosu (zayif olmadan leak gosterimi, zayif ile temiz).

### Memory testler

- Native exe'yi bir `valgrind`/Windows benzerinde calistirma (Application
  Verifier veya Dr. Memory). Ilk asamalarda elle, sonra CI'da.
- Test programi: 100,000 metin yarat ve kapsam disina cikar; HeapFree
  cagri sayisi tahsis sayisina esit olmali.

### Stress testler

- 10M paylas/birak dongusu - performans regresyon.
- 1M nesneli linked list - iteratif destructor stack overflow yapmamali.

## 15. Turkce Terminoloji

Dil ic ve disinda tutarliligi koru:

| Ingilizce | Anadil |
|---|---|
| reference | gonderge |
| owner | sahip |
| ownership | sahiplik |
| heap | obek |
| stack | yigit |
| allocate | tahsis et |
| free / drop / release | birak |
| reference count | gonderge sayaci |
| share / retain | paylas |
| weak reference | zayif gonderge |
| deep copy | derin kopya |
| destructor | yikici |
| memory leak | bellek sizintisi |
| cycle | dongusel referans |
| panic | yikim (runtime hatasi) |

Runtime fonksiyon adlari `anadil_runtime_<turkce_ascii>` desenini izler:
`anadil_runtime_paylas`, `anadil_runtime_birak`, `anadil_runtime_tahsis`.
ASCII'de kalmasi cunku MASM Unicode identifier kabul etmiyor.

## 16. Gelecekte Dusunulebilecekler

Su an scope disinda; ilerleyen surumlerde acilabilir.

### Cycle collector

Python tarzi devamli zaman zaman heap tarayan toplayici. ~1500-2000 satir.
Anadil "ileri" kullanim icin gerek olunca eklenir.

### Escape analysis (stack-or-heap)

Compiler "bu nesne kapsamdan disari cikiyor mu?" sorusunu cevaplar.
Cikmayanlari stack'te ayirir, RC cagrisi uretmez. Performans icin etkili,
implementation orta zorlukta. RC Faz 5-6 tamamlandiktan sonra eklenebilir.

### Move-only types

Bazi tipler `paylas` edilemesin (sahiplik mutlaka tasinsin). Dosya
handle'lari, kilit nesneleri vb. icin uygun. Rust'ta default; Anadil'de
opt-in olabilir.

### `metin_yapici` (StringBuilder)

Buyuk metin birlestirmeleri icin amortized O(n) tip. Runtime'da liste
tutar, `bitir()` ile final `metin` uretir.

### Custom allocators

Arena/region allocators bazi durumlarda RC'den hizli olur. Ileride
opsiyonel olarak `bolge { ... }` blok syntax'i ile kullanici belirtebilir.

### Async / kapali fonksiyonlar (closures)

Capture edilen degiskenler heap'e tasinmali, RC ile sahiplenilmeli.
Closure tasarimi yapilirken bu modulun guncellenmesi gerek.

## 17. Karar Tarihi

Bu belge ilk olarak Anadil RC tasariminin baslangicinda yazildi. Karar
sahibi: Emir Canbaz. Implementation sirasinda burada belirlenmis kurallar
yol gosterici olarak kullanilir; sapma gerekiyorsa belge guncellenmeli.
