# Anadil Todo

## Hemen Sonraki Adim

- ~~Native IDE kullanici akisini interpreter modundan arindir; `Yap`/`F5`
  native EXE derleyip hemen calistirmali.~~
- ~~CLI `calistir` ve ciplak dosya akisini native derle-ve-calistir yap;
  interpreter'i `yorumla` komutuna tasi.~~
- Runtime object cache icin native testleri calistir ve commit/push sonrasi ilk build davranisini izle.
- Runtime `.lib` modelinin commit/push sonrasi ilk build davranisini izle.
- Memory management notunu push sonrasi gozden gecir.

## Sonraki Native Compiler Sprinti

1. Runtime object cache ekle.
2. Cache invalidation icin runtime asm timestamp kontrolu yap.
3. Link hattini cached runtime objesiyle calistir ve mevcut native testleri koru.
4. ~~V0.1 compiler tamam kriterlerini yaz.~~ (project_status.md `V0.1 Tamam Kriterleri`)
5. Runtime library yeterince sabitlenince paketleme modelini degerlendir.

## Native IDE

- [x] `Yap`/`F5` akisini native derle-ve-calistir yap.
- [x] Smoke test sonucunu kaydet. (2026-05-10 otomatik build/test sonucu `Docs/ide_smoke_test.md` icinde)
- [ ] Manuel GUI smoke akisini uygula.
- [x] Satir numarasi gutter'ini editor yazma erisimini bozmadan geri getir.
- [ ] Build panelindeki hata metinlerini kart/bolum olarak daha okunur yap.
- [ ] Dosya explorer'da uzun proje ve alt klasor deneyimini tekrar kontrol et.
- [ ] Son acilan proje/dosya state'inin bozuk path durumunda sessizce toparlandigini test et.

## Test Bosluklari

V0.1 kapsam tarama sonuclari `Docs/test_coverage.md` icinde. Yuksek oncelik
ek testler:

- [x] Void (donus tipsiz) fonksiyon icin interpreter+native parity ornegi.
- [x] Recursive fonksiyon icin interpreter+native parity ornegi.
- [x] 6 parametreli fonksiyon icin interpreter+native parity ornegi.
- [x] `mantik` esitlik ve esitsizlik (`==`, `!=`) icin sema karari ve testi.

Orta/dusuk oncelik bosluklar (sema diagnostic testleri, CLI alt komut testleri,
parantez/yorum/unary edge'ler) `Docs/test_coverage.md` "Bulunan Bosluklar"
basligi altinda listelenmistir.

## Native Compiler

- [ ] Native compiler regresyon orneklerini interpreter disi beklenen cikti modeliyle genislet.
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
- [ ] Runtime helper objesini tekrar kullanilabilir kutuphane modeline tasarla.

## Dil Tasarimi

- [ ] Turkce karakterli ana builtin yazimi ve `yazdir` ASCII alias kararini dokumanlarda tutarli tut.
- [ ] Dizi/struct/modul icin MVP kapsam kararini yaz.
- [x] Memory management modelini kisa tasarim notuna dok.

## Sonra

- [ ] Outline paneli: fonksiyon listesi.
- [ ] Basit autocomplete: anahtar kelimeler, `Ana`, `yazdir`.
- [ ] Gorsel tasarim polish: smoke test temiz gecmeden baslama.
