# Anadil Todo

## Hemen Sonraki Adim

- Runtime object cache icin native testleri calistir ve commit/push sonrasi ilk build davranisini izle.
- Runtime `.lib` modelinin commit/push sonrasi ilk build davranisini izle.
- Native runtime hata mesajlarini tek formatta netlestir.

## Sonraki Native Compiler Sprinti

1. Runtime object cache ekle.
2. Cache invalidation icin runtime asm timestamp kontrolu yap.
3. Link hattini cached runtime objesiyle calistir ve mevcut native testleri koru.
4. Runtime hata ciktilarini tek formatta netlestir.
5. Memory management notunu yaz: MVP'de stack/static, sonraki hedefte heap allocator.
6. Runtime library yeterince sabitlenince paketleme modelini degerlendir.

## Native IDE

- [ ] Smoke test sonucunu kaydet.
- [x] Satir numarasi gutter'ini editor yazma erisimini bozmadan geri getir.
- [ ] Build panelindeki hata metinlerini kart/bolum olarak daha okunur yap.
- [ ] Dosya explorer'da uzun proje ve alt klasor deneyimini tekrar kontrol et.
- [ ] Son acilan proje/dosya state'inin bozuk path durumunda sessizce toparlandigini test et.

## Native Compiler

- [ ] Interpreter/native karsilastirmasini daha fazla ornekle genislet.
- [x] String ve bool `yazdir` davranisini native tarafta smoke test et.
- [ ] Runtime hatalarini native executable ciktilarinda daha net raporla.
- [x] Windows path/OneDrive/Turkce karakter senaryolarini build testlerine ekle.
- [x] Runtime helper'larini ayri obje olarak linkleme yolunu tasarla.
- [x] Metin karsilastirmayi C `strcmp` yerine Anadil runtime icinde uygula.
- [x] Runtime panic cikisini C `exit` yerine Windows process cikisina bagla.
- [x] `printf` ve `getchar` bagimliliklarini runtime I/O katmaniyla azalt.
- [x] Native runtime I/O icin sayi, bos metin ve UTF-8 edge testleri ekle.
- [x] Runtime object cache ekle.
- [x] Runtime objesini `.lib` modeline tasi.
- [ ] Windows API bagimli runtime katmanini ileride platform soyutlamasina bol.
- [ ] Runtime helper objesini tekrar kullanilabilir kutuphane modeline tasarla.

## Dil Tasarimi

- [ ] Turkce karakterli ana builtin yazimi ve `yazdir` ASCII alias kararini dokumanlarda tutarli tut.
- [ ] Dizi/struct/modul icin MVP kapsam kararini yaz.
- [ ] Memory management modelini kisa tasarim notuna dok.

## Sonra

- [ ] Outline paneli: fonksiyon listesi.
- [ ] Basit autocomplete: anahtar kelimeler, `Ana`, `yazdir`.
- [ ] Gorsel tasarim polish: smoke test temiz gecmeden baslama.
