# Anadil — Proje Raporu

**Ders:** Bilgisayar Mühendisliği Proje 2
**Üniversite:** Kırıkkale Üniversitesi — Bilgisayar Mühendisliği Bölümü
**Hazırlayan:** Emir Canbaz [ve proje ortağı]
**Sürüm:** Anadil V0.1
**Tarih:** Mayıs 2026

---

## 1. Proje Özeti

**Anadil**, Türkçe söz dizimine sahip, kendi yerel derleyicisini, kendi
çalışma zamanını (runtime) ve kendi tümleşik geliştirme ortamını (IDE)
barındıran küçük ölçekli bir programlama dilidir. Bu raporda kapsanan
**V0.1 sürümü** kaynak kodu doğrudan Windows x64 makine koduna çeviren
tam bir derleyici hattı içerir; üretilen `.exe` dosyaları C çalışma
zamanı kütüphanelerine bağımlı değildir, yalnızca Windows işletim
sisteminin çekirdek API'leriyle çalışır.

Tek cümlelik tanım: **Anadil, kaynak Türkçe sözdizimi okur, kendi
derleyici hattından geçirir ve C bağımlılığı olmayan bağımsız bir
Windows yürütülebilirine dönüştürür.**

---

## 2. Konunun Seçilme Nedeni

Bu projenin konusu danışman hocamız tarafından verilmedi; biz seçtik.
Hocamızın kendi sözleriyle:

> "Bu sektörde işletim sistemi yazmak en zor iş, programlama dili yazmak
> ikinci en zor iş."

**Proje 1** dersinde, başka bir hoca için Rust ile küçük bir işletim
sistemi yazma çalışmasını tamamlamış olduk. Bu nedenle ikinci proje
dönemine adım attığımızda, doğal bir devam adımı olarak ikinci en zor
işi — bir programlama dilini ve onun derleyicisini — sıfırdan ele
almayı seçtik. Konu aynı zamanda derin sistem programlama, dil
tasarımı, yazılım mühendisliği disiplini ve test stratejisi gibi
birden çok alanı bir araya getirdiği için müfredattaki birikimimizi
toplu hâlde uygulayabileceğimiz bir konu olarak değerlendirildi.

Konuyu hocamıza ilk açtığımızda, normalde bölümde iki kişilik proje
yapma izninin verilmediği söylendi. Ancak "kendi derleyicimizi sıfırdan
yazacağız" dediğimizde hocamız onay verdi.

---

## 3. Neyi İnşa Ettik

V0.1 sürümünde aşağıdaki bileşenler tamamlanmıştır:

### 3.1 Dil Çekirdeği

- **Lexer** — kaynak metni belirteçlere (token) ayırır; Türkçe
  karakterli tanımlayıcıları (`sayı`, `mantık`, `metin`, `eğer`,
  `değilse`, `döngü`, `kır`, `devam`, `dön`, `doğru`, `yanlış`,
  `yazdır`) doğru şekilde tanır.
- **Parser** — belirteç dizisinden soyut sözdizim ağacı (AST) üretir.
- **Semantic Analiz** — tip kontrolü, kapsam çözümlemesi, `Ana()`
  giriş noktası kontrolü, fonksiyon parametre uyumu denetimi yapar.
- **Tip-li AST** (Typed AST) — semantik bilgi gömülü ara temsil.
- **Interpreter** — programı doğrudan çalıştırır; geliştirme sırasında
  hızlı doğrulama amacıyla ve native derleyiciyle çıktı karşılaştırma
  için kullanılır.

### 3.2 Yerel Derleyici (Native Compiler)

- Tip-li AST'den Windows x64 MASM assembly kodu üretir.
- Microsoft toolchain'i (`ml64`, `lib`, `link`) ile `.obj` ve `.exe`
  üretir.
- Aritmetik, karşılaştırma, koşul, döngü (üç çeşit), fonksiyon
  tanım/çağrı, yedi parametreye kadar parametre geçişi, kapsam
  yönetimi gibi tüm dil yapılarını native koda çevirir.
- Sıfıra bölme gibi çalışma zamanı hatalarını standart formatta
  raporlar.

### 3.3 Çalışma Zamanı (Runtime)

- `runtime/anadil_runtime.asm` dosyasında ayrı bir MASM modülü olarak
  yazılmıştır.
- Doğrudan Windows `kernel32` API'leri (`WriteFile`, `ReadFile`,
  `GetStdHandle`, `ExitProcess`) üzerinden çalışır.
- C çalışma zamanı kütüphanelerine **bağımlı değildir** (`printf`,
  `strcmp`, `getchar`, C `exit` gibi fonksiyonlar kullanılmaz).
- Kendi `print_sayi`, `print_metin`, `print_mantık`, `strcmp`, `panic`,
  `wait_before_exit` yardımcılarını sağlar.
- Ayrı bir `.lib` kütüphanesi olarak paketlenir; programlar bu
  kütüphane ile bağlanır.

### 3.4 Komut Satırı Arayüzü

`anadil` çalıştırılabilir komut satırı arayüzü 11 alt komut sağlar:

| Komut | İşlev |
|---|---|
| `calistir` | Programı interpreter ile çalıştırır |
| `kontrol` | Programı derler ama çalıştırmaz; hata kontrolü |
| `ast` | Soyut sözdizim ağacını yazdırır |
| `typed` | Tip-li AST'yi yazdırır |
| `asm` | Üretilen Windows x64 assembly kodunu yazdırır |
| `asm-yaz` | Assembly kodunu `.asm` dosyasına yazar |
| `derle` | Programı `.exe` olarak derler |
| `ide` | Tarayıcı tabanlı yerel IDE'yi başlatır |
| `ornekler` | `examples/` altındaki örnekleri listeler |
| `surum` | Sürüm bilgisini yazdırır |
| `yardim` | Yardım metnini yazdırır |
| `repl` | Etkileşimli okuma-değerlendirme döngüsü |

`kontrol`, `calistir` ve `derle` komutlarının `--json` varyantı, IDE
entegrasyonu için kararlı bir tanılama protokolü sağlar.

### 3.5 Yerel IDE

`anadil-ide` adında ayrı bir yerel masaüstü uygulaması:

- Editor, dosya gezgini, derleme paneli ve tanılama paneli içerir.
- Sözdizim renklendirme, satır numarası, kapanmış parantez/tırnak
  tamamlama gibi temel düzenleyici kolaylıkları sağlar.
- "Yorumla", "Derle" ve "Karşılaştır" modlarını tek arayüzden çalıştırır.
- Tanılama hatalarına tıklayarak doğrudan kaynak kod konumuna gider.
- Modern koyu tema ve Türkçe arayüz dilinde çalışır.

---

## 4. Sistemin Genel İşleyişi

Anadil kaynak kodunun yürütülebilire dönüşüm hattı:

```
.ana kaynak dosyası
    │
    ▼
  Lexer
    │  (token dizisi)
    ▼
  Parser
    │  (AST)
    ▼
  Semantic Analiz
    │  (Typed AST)
    ▼
  Native Code Generator
    │  (Windows x64 MASM assembly)
    ▼
  ml64 (Microsoft assembler)
    │  (program.obj)
    ▼
  link (Microsoft linker)  +  anadil_runtime.lib  +  kernel32.lib
    │
    ▼
  program.exe  (bağımsız Windows yürütülebilir)
```

Her aşama bir öncekinden bilgi alır ve bir sonrakine sade bir veri
yapısı verir. Bu klasik derleyici hattı; ders kitaplarında anlatılan
yapının pratik bir uygulamasıdır. Anadil'i diğer öğrenci projelerinden
ayıran nokta, bu hattın **hiçbir aşamasında dış bir derleyici altyapısının
(LLVM, GCC, transpilation hedefi olarak C dili) kullanılmamış olmasıdır**:
makine kodu doğrudan üretilmektedir.

---

## 5. Dilin Görünümü

Anadil sözdizimi C ailesinden ilham alır ama anahtar kelimeleri ve tipleri
Türkçedir. Bir kaç tipik program:

### Toplama

```
Topla(a: sayı, b: sayı) -> sayı {
    dön a + b;
}

Ana() {
    sonuc: sayı = Topla(10, 20);
    yazdır(sonuc);
}
```

Çıktı: `30`

### Koşul ve Döngü

```
Ana() {
    döngü (i: sayı = 0; i < 5; i = i + 1) {
        eğer (i == 3) {
            devam;
        }
        yazdır(i);
    }
}
```

Çıktı: `0`, `1`, `2`, `4`

### Boole ve Metin

```
Ana() {
    yazdır(doğru);
    yazdır("Merhaba, dünya");
    yazdır(10 > 5);
}
```

Çıktı: `doğru`, `Merhaba, dünya`, `doğru`

Dil; `sayı` (64-bit tam sayı), `mantık` (boole) ve `metin` (UTF-8
string sabiti) olmak üzere üç temel tipi destekler. Türkçe `yazdır`
yerleşik fonksiyonu üç tipi de yazdırabilir.

---

## 6. Tasarım Kararları ve Gerekçeleri

Bu bölüm, projeyi bugünkü hâline getiren en önemli teknik kararları
içerir. Her karar için "alternatif", "biz neyi seçtik" ve "niçin bu yol"
şeklinde yapılandırıldı.

### 6.1 Yorumlamak (interpret) yerine derlemek (compile)

**Alternatifler:**
- Sadece interpreter yazmak (Python, JavaScript benzeri)
- Bytecode + sanal makine (Java, C# benzeri)
- Doğrudan native kod üretmek

**Seçim:** Hem interpreter hem native derleyici yazıldı.

**Gerekçeler:**
- Sadece interpreter, bir programlama dili dersinin teknik derinliğini
  tam yansıtmaz; native code generation, calling convention, stack
  frame yönetimi gibi sistem-seviyesi konuları kapsamadan dil tasarımı
  eksik kalır.
- Hem interpreter hem native derleyici yazılması iki ek fayda sağlar:
  ilki interpreter'ın geliştirme sırasında hızlı doğrulama ortamı
  olması; ikincisi her iki çıktının birbirini test ederek otomatik
  doğrulama vermesi (ileride 9. bölümde açıklanacak parity testleri).

### 6.2 LLVM/Cranelift gibi mevcut backend'i kullanmamak

**Alternatifler:**
- LLVM IR üretmek; LLVM bizim yerimize optimizasyon ve makine kodu üretirdi.
- Cranelift gibi daha küçük bir Rust ekosistemi backend'ini kullanmak.
- Doğrudan makine koduna yazmak.

**Seçim:** Doğrudan Windows x64 MASM assembly'si üretildi.

**Gerekçeler:**
- Projenin eğitim hedefi "bir derleyici nasıl çalışır" sorusunun
  cevabını doğrudan deneyimlemekti. LLVM ya da Cranelift kullansaydık,
  derleyicinin en öğretici kısmı (kod üretimi) bizim alanımızın dışında
  kalırdı.
- Calling convention, stack frame, register kullanımı, parametre
  geçişi gibi düşük seviye konuların elden uygulanması, üst düzey bir
  çerçeve kullanmaktan daha öğretici bir yol oldu.
- Bağımlılık olarak yalnızca Microsoft toolchain'in olması, projenin
  taşınabilirliği konusunda da kontrolümüzü artırdı.

### 6.3 C diline transpile etmemek

**Alternatif:** Birçok küçük dil (Vala, Nim'in eski sürümleri, C++ benzeri
dilller) derleyicilerini C dili çıkışı üreten "kaynak kodu çevirici"
(transpiler) olarak kurar; sonrasında gcc/clang ile derler.

**Seçim:** C kodu üretilmedi; doğrudan assembly üretildi.

**Gerekçeler:**
- C'ye transpile etmek "asıl derleyiciyi başkasına yaptırmak" anlamına
  gelir. Eğitim hedefi bu değildi.
- Üretilen kodun davranışını birebir kontrol edebilmek için kaynak
  seviyesinde değil, makine seviyesinde çalışmak istedik.
- C kod üretimi, dilimizin bellek modeli, çalışma zamanı davranışı ve
  hata raporlama konularında C'nin sınırlarına saygı göstermek demek
  olurdu; biz bunlardan bağımsız bir model kurabilmek istedik.

### 6.4 C çalışma zamanı kütüphanesine bağımlı olmamak

**Alternatif:** Üretilen `.exe` dosyaları `printf`, `strcmp`, `exit`
gibi C standart kütüphane fonksiyonlarına çağrı yapabilirdi (msvcrt /
ucrt). Bu en kolay yoldur.

**Seçim:** C runtime'ı tamamen devre dışı bırakıldı. Üretilen
yürütülebilir yalnızca `kernel32.dll` (Windows çekirdek API'leri)
üzerinden çalışır.

**Gerekçeler:**
- Üretilen `.exe` dosyalarının "gerçekten kendi çalışma zamanımıza
  sahip olması" istendi. Eğer C runtime kullansaydık, "Anadil
  derleyici" değil "Anadil → C-runtime sarmalayıcısı" üretmiş olurduk.
- Düşük seviye sistem çağrılarını doğrudan kullanmak (Win32 API),
  öğrenme açısından daha derin bir sonuç verdi.
- Üretilen `.exe`'nin daha küçük ve daha bağımsız olması yan kazanım.
- Bu karar projenin teknik vurgularından biridir; aynı kararı veren
  dil sayısı (gerçek anlamda) sınırlıdır.

### 6.5 Türkçe sözdizim

**Alternatif:** İngilizce anahtar kelimeler kullanmak (`if`, `else`,
`while`, `return`, `print`, vs.) — sektör standardı.

**Seçim:** Tüm anahtar kelimeler ve yerleşik fonksiyon isimleri Türkçe
yazıldı (`eğer`, `değilse`, `döngü`, `dön`, `yazdır`, `sayı`, `mantık`,
`metin`).

**Gerekçeler:**
- Programlama eğitiminde anadilin etkisi tartışılan bir konudur. Türkçe
  bir dil ile, kavramların sözcüklerle eşleşmesi (örn. `sayı`,
  `eğer/değilse`) öğrenme sürecinde ek bir bilişsel adımı azaltır.
- Türkçe karakterlerin (ç, ğ, ı, ö, ş, ü) kaynak kod düzeyinde
  desteklenmesi, lexer ve dosya yolu yönetiminde özel UTF-8 dikkati
  gerektirdi; bu da projenin teknik derinliğine katkıda bulunan bir
  kararsdır.
- Bu yön, projeyi sıradan bir öğrenci derleyicisi olmaktan çıkarıp
  belirgin bir kimliğe büründürdü.

### 6.6 Yalnızca Windows x64 hedefi

**Alternatif:** Linux ve macOS desteği de eklemek.

**Seçim:** V0.1 yalnızca Windows x64 hedefler.

**Gerekçeler:**
- Bir dönemlik proje süresinde her platform için ayrı çalışma zamanı
  yazmak, derleyicinin kendisinin sağlamlığını gölgede bırakırdı.
- Üç farklı assembly sözdizimi (MASM, GAS, AT&T) öğrenmek ve üç farklı
  syscall ABI'sı ile çalışmak zaman maliyeti yüksek bir genişleme
  olurdu.
- Bunun yerine Windows üzerinde derinlik elde edildi; gelecekteki
  platform genişlemesi için tasarım notu hazırlandı (çalışma zamanı
  soyutlama planı `Docs/runtime_platform_abstraction.md` içinde).

---

## 7. Teknik Detaylar (Öne Çıkanlar)

Aşağıdaki teknik özellikler projenin "öğrenci ödevi" değil, "ciddi
mühendislik" boyutunda olduğunu gösteren noktaları temsil eder.

### 7.1 Doğrudan x64 Assembly Üretimi

Derleyicinin kod üretici katmanı (`src/native.rs`), tip-li AST'yi
gezerek karakter karakter MASM assembly kodu üretir. Üretilen kod:

- Windows x64 calling convention'ına uyar (ilk dört parametre
  `rcx`/`rdx`/`r8`/`r9`, sonrası stack üzerinden).
- Her fonksiyon için kendi stack frame'ini kurar (`push rbp`, `mov rbp,
  rsp`, `sub rsp, frame_size`).
- 16 byte hizalamayı korur (Windows ABI gereği).
- Fonksiyon çağrılarından önce shadow space (32 byte) ayırır.
- Yedi parametreye kadar fonksiyon çağrısı, parametrelerin doğru
  sırada hem register hem stack'e yerleştirilmesini doğru yapar.

### 7.2 Win32 Syscall'ları ile Çalışma Zamanı

Çalışma zamanı (`runtime/anadil_runtime.asm`) yalnızca dört Windows
sistem fonksiyonu kullanır:

| Win32 API | Kullanım |
|---|---|
| `GetStdHandle` | stdin/stdout dosya tutamacı alır |
| `WriteFile` | byte dizisini stdout'a yazar |
| `ReadFile` | program sonunda Enter beklemek için |
| `ExitProcess` | runtime hatasında programı sonlandırır |

Sayıdan ondalık metne çevirim, byte byte metin karşılaştırma,
boole-to-metin dönüşümü gibi tüm yardımcı işlevler **assembly içinde**
elden yazıldı; herhangi bir hazır kütüphane çağrılmadı.

### 7.3 Çalışma Zamanı Kütüphanesi Modeli

Her derleme isteğinde runtime'ı yeniden assemble etmek yerine,
runtime ayrı bir `.lib` dosyası olarak cache'lenir:

- `runtime/anadil_runtime.asm` (kaynak)
- `target/native-runtime/anadil_runtime.obj` (cache)
- `target/native-runtime/anadil_runtime.lib` (cache)

Cache geçersizleştirme dosya değiştirilme zamanına (`mtime`) göre
yapılır; kaynak `.asm` daha yeniyse veya cache yoksa yeniden üretilir.
Paralel derleme isteklerinde cache dosyalarının yarış durumlarına
düşmesini önlemek için dosya tabanlı kilit mekanizması (`mkdir`-bazlı
atomic lock) kuruldu.

Linker komut satırı yalnızca üç bağımlılığa sahiptir:

```
program.obj  +  anadil_runtime.lib  +  kernel32.lib  →  program.exe
```

C runtime kütüphaneleri (`msvcrt.lib`, `ucrt.lib`, `vcruntime.lib`,
`legacy_stdio_definitions.lib`) bu zincirde **yer almaz**.

### 7.4 Tanılama Protokolü

Derleyici hata mesajları iki katmanda tutarlıdır:

- **İnsan okur format:** satır, sütun ve caret ("^") gösterimi içerir,
  C/Rust derleyicilerinin kullandığı stilde:

  ```
  Semantic hata, satır 3, sütun 12: Tip uyumsuzluğu: sayı ve mantık.
       sonuc: sayı = doğru;
                     ^^^^^
  ```

- **JSON format:** IDE entegrasyonu için makine okur. Şu sema:

  ```json
  {
    "ok": false,
    "diagnostics": [
      {"severity": "error", "stage": "semantic",
       "message": "...", "line": 3, "column": 12}
    ]
  }
  ```

`kontrol --json`, `calistir --json` ve `derle --json` komutları bu
protokolü kararlı şekilde sağlar.

### 7.5 Test Stratejisi

Anadil dört bağımsız test paketi içerir:

| Test paketi | Ne doğrular |
|---|---|
| `tests/examples.rs` | Örnek programlar interpreter ile beklenen çıktıyı veriyor mu |
| `tests/native_examples.rs` | Aynı örnekler native `.exe` ile **aynı** çıktıyı veriyor mu (parity) |
| `tests/native_edge_cases.rs` | Sıfıra bölme, dört/beş/yedi parametreli fonksiyon, scope shadowing, UTF-8, Türkçe yol gibi kenar durumlar |
| `tests/cli_diagnostics.rs` | CLI hata çıktısının formatı bozulmamış mı (regression) |

Bu yaklaşımda **interpreter, native derleyicinin doğruluğu için
referans** rolünde kullanılır; her örnek programın iki bağımsız yolla
aynı sonucu vermesi gerekir. Bu disiplin, native code generation
katmanında oluşacak sessiz hataları yakalamayı kolaylaştırır.

### 7.6 Türkçe Karakter ve Yol Dayanıklılığı

Aşağıdaki senaryolar test paketinde kalıcı olarak korunur:

- Boşluk içeren kaynak yolları (`target/native path cases/...`).
- Türkçe karakterli klasör isimleri (`Türkçe Klasör`, `deneme dosyası.ana`).
- Windows OneDrive/Masaüstü gibi uzun yollar.
- Kaynak kod içinde UTF-8 karakter dizileri (`"Merhaba, dünya"`).
- `metin` içinde NULL byte içermeyen UTF-8 dizilerinin doğru aktarımı.

Bu, dilin yalnızca İngilizce ASCII ortamlarda değil, gerçek bir Türkçe
geliştirici makinesinde de güvenilir çalışmasını sağlar.

---

## 8. Bilinçli Sınırlar

V0.1 sürümü dar tutulmuştur. Aşağıdakiler **bilerek** kapsam dışında
bırakılmıştır:

- **Heap allocation yok.** `metin`, `dizi`, `yapı` (struct) gibi
  dinamik veri tipleri için runtime tahsis altyapısı V0.1'de yer almaz.
  Mevcut `metin` desteği yalnızca derleme zamanında bilinen sabit
  metinler için geçerlidir.
- **Çöp toplayıcı (GC) yok.** Heap olmadığı için gerek de yoktur.
- **Referans sayma yok.** V0.2 sürümü için tasarım notu hazırlanmıştır
  ancak V0.1 kapsamında uygulanmamıştır.
- **Optimizasyon geçişi yok.** Kod üretici doğrudan AST gezerek kod
  yazar; optimize edilmiş ara temsil katmanı yoktur.
- **Hata ayıklama bilgisi (debug info) üretilmez.** `.exe` çıktıları
  Visual Studio debugger'a açıldığında kaynak satır eşleşmesi olmaz.
- **Sadece Windows x64.** Linux ve macOS hedefleri planlanmıştır
  ancak V0.1'de yoktur.
- **Modül sistemi yok.** Tüm program tek bir `.ana` dosyasından oluşur.

Bu kararlar projenin "yetersiz" kalması için değil, **bir dönem
süresince ulaşılabilir bir hedefin sağlam bir şekilde tamamlanması**
için alındı. Kapsamı geniş tutmak yerine kapsamı dar tutup her
seçileni sağlam yapmak, mühendislik açısından daha değerli bir tercih
olarak kabul edildi.

---

## 9. Doğrulama ve Test Yaklaşımı

Anadil V0.1 sürümünün doğruluğu üç katmanda doğrulanır:

### 9.1 Birim Testleri (Unit)

Lexer, parser, semantik analiz ve native kod üretici Rust birim
testleri ile kapsanır. Compiler iç fonksiyonları (örn. dosya yolu
güvenliği, IDE durum dosyası ayrıştırma, runtime kaynağı bulma)
bağımsız test edilir.

### 9.2 Entegrasyon Testleri

`examples/` altındaki her örnek program iki bağımsız yoldan çalıştırılır:

1. Interpreter ile (`anadil calistir`)
2. Native derleyici ile (`anadil derle && examples/x.exe`)

İki çıktının **birebir aynı** olması beklenir. Bu yöntem, native code
generation katmanında oluşacak çoğu hatayı sessiz kalmadan yakalar.

Test edilen programlar:

`topla`, `negatif`, `kosul`, `fonksiyon`, `mantik`, `metin`,
`kosullu_dongu`, `dongu`, `sonsuz_dongu`, `kapsam`, `native_mvp`.

### 9.3 Kenar Durum Testleri

`tests/native_edge_cases.rs` özel olarak şu durumları kapsar:

- Dört, beş ve yedi parametreli fonksiyonlar (calling convention sınırı).
- İç içe fonksiyon çağrılarında parametre koruma.
- Sayısal karşılaştırma operatörleri (`==`, `!=`, `<`, `<=`, `>`, `>=`).
- Tam sayı yazdırma kenar değerleri (sıfır, negatif, on dokuz haneli sayı).
- Boş metin, UTF-8 metin, Türkçe karakterli metin.
- Sıfıra bölme runtime hatası ve doğru hata mesajı.
- İç içe `eğer`/`döngü` ve `kır`/`devam` etkileşimi.
- Kapsam gölgeleme (scope shadowing).
- Boşluk ve Türkçe karakter içeren kaynak yolları.

Tüm test paketleri her commit'ten önce yeşil tutulur (`cargo test`).

---

## 10. Olası Sorular ve Cevapları

Bu bölüm, raporun teknik tarafına yönelik beklenebilecek soruların
cevaplarını toplamak için hazırlandı.

### S1. "Neden bir programlama dili yazıyorsunuz? Kullanıcısı olacak mı?"

Anadil'in pratik kullanım için bir dil olarak rakiplere alternatif
sunduğu iddiasında değiliz. Projenin amacı eğitsel: bir derleyicinin
nasıl çalıştığını ders kitabından okumak yerine bizzat inşa ederek
öğrenmek. "Derleyici nasıl çalışır" sorusunun en doğru cevabı, basit
de olsa kendi derleyicisini yazıp test edenden gelir.

### S2. "Neden LLVM kullanmadınız? Şirketler de LLVM kullanıyor."

LLVM kullansaydık, derleyicinin **en öğretici kısmı (kod üretimi)**
bizim alanımızın dışında kalırdı. LLVM'in ardındaki kavramları —
calling convention, stack frame yönetimi, register kullanımı, syscall
arayüzü — kendimiz uygulayarak öğrenmek istedik. LLVM bir araçtır;
projenin amacı bu aracı kullanmak değil, bu aracı oluşturan kavramları
deneyimlemekti.

### S3. "C diline transpile edebilirdiniz, çok daha kolay olurdu."

C'ye transpile etmek, "asıl derleyiciyi başkasına yaptırmak" anlamına
gelir; sonuçta kullanıcının `.exe` dosyasını üreten gcc ya da clang
olurdu. Biz makine kodu seviyesinde çalışmak istedik. Ayrıca C'ye
çevirmek, dilimizin bellek modeli, çalışma zamanı davranışı ve hata
raporlama açısından C'nin kısıtlarına saygı göstermek demek olurdu.
Bağımsız tasarım kararları alabilmek için C'yi araya sokmadık.

### S4. "Neden C runtime kullanmadınız? `printf` zaten var."

`printf`, `strcmp`, `getchar` gibi C runtime fonksiyonları kullansaydık,
ürettiğimiz `.exe` aslında "Anadil program + C runtime" karışımı
olurdu. Bunun yerine doğrudan Windows `kernel32` API'leri kullanan
kendi çalışma zamanımızı yazdık. Üretilen `.exe` yalnızca işletim
sistemine bağımlıdır, başka hiçbir kütüphaneye değil. Bu karar
projenin en belirleyici teknik özelliklerinden biridir.

### S5. "Sayıyı ondalık metne nasıl çeviriyorsunuz, `printf` olmadan?"

Çalışma zamanında byte byte yazılmış bir tam sayı → ondalık metin
çevirici fonksiyon var. 64-bit tam sayı 10'a tekrarlanan bölmeyle
ondalık basamaklara ayrılır, basamaklar ASCII'ye dönüştürülerek bir
geçici tampona yazılır, ardından `WriteFile` ile stdout'a basılır.
Tüm mantık `runtime/anadil_runtime.asm` içinde, doğrudan x64
talimatlarıyla yazılmıştır.

### S6. "Türkçe sözdizim sadece estetik mi?"

Kısmen estetik, kısmen pratik. Estetik tarafı: Anadil'i sıradan bir
öğrenci derleyicisinden ayıran ve ona kimlik katan tasarım kararı.
Pratik tarafı: Türkçe karakter desteği lexer, dosya yolu yönetimi ve
UTF-8 işleme konularında ek mühendislik gerektirdi (sadece İngilizce
ASCII ile çalışsaydık çok daha kolay olurdu). Ayrıca dilin Türkçe
olması, programlama eğitiminde anadilin kavramsal yükü azaltmaya
katkı yapan tartışmalı ama ilginç bir dil tasarımı denemesi.

### S7. "Neden sadece Windows? Linux daha doğal değil mi?"

Bir dönemlik bir projede üç işletim sistemi için ayrı çalışma zamanı
yazmak, derleyicinin kendisinin derinliğini gölgeleyecekti. Tek bir
hedefe odaklanıp orada derinlik elde etmek tercih edildi. Linux ve
macOS desteğinin nasıl eklenebileceğine dair tasarım notu hazırlandı
(`Docs/runtime_platform_abstraction.md`); mevcut çalışma zamanının
soyutlanabilir olduğu ve genişlemenin **mimari değişiklik gerektirmediği**
gösterilmiştir.

### S8. "Heap allocation, dizi, yapı yok. Dilin gerçek bir kullanım için
yeterli mi?"

V0.1 için yeterli **değildir** ve bunu açıkça belirtiyoruz. V0.1'in
hedefi yetkin bir genel amaçlı dil değil, derleyici hattının ve
çalışma zamanı modelinin sağlam temelinin atılmasıdır. Bu temel
üzerine V0.2'de heap allocator, referans sayma, dinamik metin, dizi
ve yapı eklenmesi planlanmıştır. Tasarım notu hazırlanmış durumdadır
(`Docs/memory_model.md`); ancak V0.2'nin uygulaması proje süresine
sığmadığı için bu raporda değerlendirme dışıdır.

### S9. "Native ve interpreter aynı çıktıyı vermesini nasıl garanti
ediyorsunuz?"

`tests/native_examples.rs` her örnek için iki ayrı yoldan çalıştırma
yapıp çıktıları karşılaştırır. Eğer native derleyici bir hata
yaparsa, otomatik test paketi başarısız olur. Bu ikili-doğrulama
yaklaşımı, native code generation katmanında oluşacak hataları manuel
inceleme olmadan yakalamamızı sağlar.

### S10. "Optimizasyon yok, üretilen kod yavaş değil mi?"

Üretilen kod basit ama doğrudur. Optimizasyon (sabit katlama, ölü kod
silme, register allocation, sıralama değişiklikleri) bilinçli olarak
V0.1 kapsamı dışında bırakıldı. Eğitim hedefimiz "doğru kod üreten
bir derleyici"ydi; "hızlı kod üreten" sonraki bir hedef. Pratikte
eğitim amaçlı küçük programlar için performans yeterlidir.

### S11. "Bir IDE niye yazdınız? Compiler'ın bir parçası değil ki."

Derleyicinin tanılama (diagnostic) çıktısının **kullanılabilir**
olduğunu göstermek için bir tüketici tarafı gerekiyordu. IDE bu
tüketicidir: derleyicinin JSON tanılama protokolünü çağırır, hataları
satır numarası ile editörde gösterir, tıklanabilir hâle getirir.
IDE projenin "yan ürünü" değil, **derleyicinin gerçek dünyada
nasıl kullanıldığının canlı kanıtıdır**. Native derleyici bir IDE
tarafından sürüklenebiliyorsa, başka bir editörden de sürüklenebilir
demektir.

### S12. "Bu projeyi neden Rust ile yazdınız?"

Üç gerekçe: (1) Proje 1'de işletim sistemi yazımında Rust
kullanmıştık, ekosisteme aşinaydık. (2) Rust'ın güçlü tip sistemi ve
ownership modeli, bir derleyici yazarken AST/IR/codegen
katmanlarındaki yapısal hataları derleme zamanında yakaladı. (3)
Anadil'in çalışma zamanı assembly'de yazılmasına rağmen, derleyicinin
kendisi yüksek seviyede tutulmalıydı; Rust bu ikisini güvenli bir
şekilde köprülüyor.

### S13. "Üzerinde ne kadar süre çalıştınız?"

Proje Nisan 2026 ilk haftasından itibaren aktif olarak yürütüldü;
yaklaşık beş haftalık bir dönem boyunca düzenli geliştirme yapıldı.
Bu süre içinde toplam yaklaşık 12.500 satırlık kod ve dokümantasyon
üretildi (kaynak kodlar, çalışma zamanı assembly'si, test paketleri,
tasarım belgeleri dahil).

### S14. "Proje 1'de işletim sistemi yazdınız. Bu proje onunla nasıl
ilişkili?"

Proje 1, Rust ile küçük bir işletim sistemi çekirdeği yazma
çalışmasıydı. Bu projede sistem çağrılarının arkasındaki dünyayı
gördük. Anadil'de bu birikim doğrudan kullanıldı: çalışma zamanı
fonksiyonlarının `WriteFile`/`ReadFile` üzerinden yazılması, calling
convention'a uyum, stack frame yönetimi gibi konular Proje 1'in
devamıydı. Hocamızın "OS yazmak en zor, dil yazmak ikinci en zor"
sözüne uygun bir akademik ilerleme.

### S15. "Bu projeden sonra ne yapacaksınız?"

V0.2 için tasarım notu hazırlandı: heap, referans sayma çalışma zamanı,
dinamik metin, dizi ve yapı. Ardından modül sistemi, daha geniş
optimizasyon, ve Linux/macOS desteği planlanan yön. Ancak bu raporun
kapsamı V0.1 sürümüdür.

### S16. "Yapılan iş gerçekten 'kendi derleyicisi' mi? Ne kadar
özgün?"

Proje şu bileşenlerin hepsini sıfırdan içerir:

- Lexer, parser, semantik analiz (Rust ile elden yazıldı)
- Tip-li ara temsil
- Native code generator (Windows x64 assembly üretici)
- Çalışma zamanı kütüphanesi (MASM assembly ile elden yazıldı)
- Test paketi
- IDE
- Komut satırı arayüzü

Hiçbir kod üretici altyapısı (LLVM, GCC, transpiler hedefi) ödünç
alınmadı. Microsoft toolchain (`ml64`, `lib`, `link`) yalnızca
"assembler ve linker" rolünde kullanıldı; bu programların yaptığı iş
kavramsal olarak makine kodu üreteci ile karıştırılmamalıdır.

---

## 11. Sonuç

Anadil V0.1 sürümü ile aşağıdakiler başarıldı:

- Türkçe söz dizimine sahip küçük bir programlama dili tasarlandı.
- Lexer, parser, semantik analiz, interpreter ve Windows x64 native
  derleyici sıfırdan yazıldı.
- C çalışma zamanına bağımlı olmayan, doğrudan Windows API'leri
  üzerinden çalışan kendi çalışma zamanı oluşturuldu.
- Çalışma zamanı ayrı bir `.lib` kütüphanesi olarak paketlendi;
  cache ve eşzamanlılık altyapısı eklendi.
- IDE entegrasyonu için JSON tanılama protokolü ve örnek IDE yazıldı.
- İnterpreter ve native derleyici çıktılarını otomatik karşılaştıran
  test paketi kuruldu.
- Türkçe karakter ve yol dayanıklılığı testlerle korundu.

Projenin **eğitim hedefi**, dilin "kullanıcı kazanması" değil, bir
derleyicinin yapı taşlarını birinci elden öğrenmekti. Bu hedef tam
anlamıyla yerine getirildi: lexer ve parser, semantik tip sistemi,
calling convention, runtime tasarımı, syscall arayüzleri, Windows
linker mantığı gibi konular sıfırdan elden uygulandı.

Hocamızın değerlendirmesine sunulan iş, bu raporda anlatılan tüm
bileşenleri ve `Docs/` klasörü içindeki ek tasarım notlarını
kapsamaktadır.

---

## Ekler

Daha derin teknik detay için aşağıdaki belgeler proje içindeki
`Docs/` klasöründe bulunmaktadır:

- `Docs/dil_referansi.md` — Anadil dil referansı (kullanıcı odaklı).
- `Docs/native_compiler.md` — Native derleyici teknik dokümanı.
- `Docs/memory_model.md` — V0.2+ için bellek modeli tasarım notu.
- `Docs/runtime_platform_abstraction.md` — Cross-platform soyutlama
  planı.
- `Docs/test_coverage.md` — Test kapsam matrisi.
- `Docs/project_status.md` — V0.1 tamam kriterleri ve genel durum.
- `Docs/local_ide.md` — IDE tasarım notları.

Kaynak kod yapısı:

- `src/` — derleyici (Rust)
- `runtime/` — çalışma zamanı (MASM assembly)
- `tests/` — test paketleri
- `examples/` — örnek `.ana` programlar

---

*Bu rapor Anadil V0.1 sürümü için hazırlandı.*
