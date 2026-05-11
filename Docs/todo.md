# Anadil Todo

## Hemen Sonraki Adim

- V0.1 RC paketini ve release yayini sonucunu izle.

## Sonraki Native Compiler Sprinti

1. ~~Runtime object cache ekle.~~
2. ~~Cache invalidation icin runtime asm timestamp kontrolu yap.~~
3. ~~Link hattini cached runtime objesiyle calistir ve mevcut native testleri koru.~~
4. ~~V0.1 compiler tamam kriterlerini yaz.~~ (project_status.md `V0.1 Tamam Kriterleri`)
5. Runtime library yeterince sabitlenince paketleme modelini degerlendir.

## Native IDE

- [x] `Yap`/`F5` akisini native derle-ve-calistir yap.
- [x] Smoke test sonucunu kaydet. (2026-05-10 otomatik build/test sonucu `Docs/ide_smoke_test.md` icinde)
- [x] Manuel GUI smoke akisini uygula.
- [x] Satir numarasi gutter'ini editor yazma erisimini bozmadan geri getir.
- [x] Build panelindeki hata metinlerini kart/bolum olarak daha okunur yap.
- [x] Dosya explorer'da uzun proje ve alt klasor deneyimini tekrar kontrol et.
- [x] Son acilan proje/dosya state'inin bozuk path durumunda sessizce toparlandigini test et.

## Test Bosluklari

V0.1 kapsam tarama sonuclari `Docs/test_coverage.md` icinde. Yuksek oncelik
ek testler:

- [x] Void (donus tipsiz) fonksiyon icin interpreter+native parity ornegi.
- [x] Recursive fonksiyon icin interpreter+native parity ornegi.
- [x] 6 parametreli fonksiyon icin interpreter+native parity ornegi.
- [x] `mantik` esitlik ve esitsizlik (`==`, `!=`) icin sema karari ve testi.

V0.2 baslangicinda P1 test borcu kapatildi: sema diagnostic ornekleri,
eksik dosya CLI diagnostic'i ve ic ice dongu native parity testi eklendi.
Kalan P2 bosluklar `Docs/test_gap_analizi.md` altinda izlenir.

## Native Compiler

- [ ] Native compiler regresyon orneklerini interpreter disi beklenen cikti modeliyle genislet.
- [x] Static `metin` literal layout'unu length-prefixed nesne bicimine tasi.
- [x] `metin + metin` icin dinamik heap metin birlestirme MVP'si ekle.
- [x] Void fonksiyon ust seviye `metin` local'leri icin temel RC cleanup emit et.
- [x] Owned/static RHS ile `metin` assignment replacement icin eski degeri birak.
- [x] Local `metin` paylasiminda `paylas` emit et.
- [x] User-defined fonksiyonlara local `metin` argumani gecirilirken `paylas` emit et.
- [x] Local `metin` return degeri icin return ownership emit et.
- [x] If/else branch normal cikisinda branch-scope `metin` local cleanup emit et.
- [x] Loop body scope cleanup ve `kır`/`devam` RC cleanup emit et.
- [x] Nested concat ve function-return concat operand temporary cleanup emit et.
- [x] `yazdir` owned temporary argumani ve kullanilmayan owned expression cleanup emit et.
- [x] User-defined fonksiyonlara inline owned `metin` arguman transferini regresyonla sabitle.
- [x] Native backend `metin` ownership siniflandirmasini tek helper'a toparla.
- [x] `uzunluk(metin) -> sayi` builtin'ini interpreter, IR ve native backend'e ekle.
- [x] `uzunluk` edge testlerini ve `examples/metin_v02.ana` ornegini ekle.
- [x] V0.2 RC audit checklist'ini `Docs/native_compiler.md` icine isle.
- [x] Native string literal emit'ini length-prefixed migration'a hazir soyutlamaya al.
- [x] IR cikti katmaninda runtime metin operasyonlarini acik temsil et.
- [x] Length-prefixed `metin` nesnesi icin runtime ABI helper'larini ekle.
- [x] V0.2 heap primitive runtime sembollerini stub olarak ekle.
- [x] String ve bool `yazdir` davranisini native tarafta smoke test et.
- [x] Runtime hatalarini native executable ciktilarinda daha net raporla.
- [x] Windows path/OneDrive/Turkce karakter senaryolarini build testlerine ekle.
- [x] Runtime helper'larini ayri obje olarak linkleme yolunu tasarla.
- [x] Metin karsilastirmayi C `strcmp` yerine Anadil runtime icinde uygula.
- [x] Runtime panic cikisini C `exit` yerine Windows process cikisina bagla.
- [x] `printf` ve `getchar` bagimliliklarini runtime I/O katmaniyla azalt.
- [x] Native runtime I/O icin sayi, bos metin ve UTF-8 edge testleri ekle.
- [x] Runtime object cache ekle.
- [x] Runtime objesini `.lib` modeline tasi.
- [x] Windows API bagimli runtime katmanini ileride platform soyutlamasina bol. (tasarim notu: `Docs/runtime_platform_abstraction.md`)

## Dil Tasarimi

- [ ] Turkce karakterli ana builtin yazimi ve `yazdir` ASCII alias kararini dokumanlarda tutarli tut.
- [ ] Dizi/struct/modul icin MVP kapsam kararini yaz.
- [x] Memory management modelini kisa tasarim notuna dok.

## Sonra

- [ ] Outline paneli: fonksiyon listesi.
- [ ] Basit autocomplete: anahtar kelimeler, `Ana`, `yazdir`.
- [ ] Gorsel tasarim polish: smoke test temiz gecti; artik baslanabilir.
