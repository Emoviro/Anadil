# Anadil Todo

## Hemen Sonraki Adim

- Native runtime helper'larini ayri runtime objesinden kutuphane modeline tasimayi planla.
- C benzeri `derle -> exe -> calistir` akisini IDE/CLI tarafinda ana yol yap.
- Runtime hata mesajlarini native executable ciktilarinda daha net raporla.

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
- [ ] Windows path/OneDrive/Turkce karakter senaryolarini build testlerine ekle.
- [x] Runtime helper'larini ayri obje olarak linkleme yolunu tasarla.
- [x] Metin karsilastirmayi C `strcmp` yerine Anadil runtime icinde uygula.
- [x] Runtime panic cikisini C `exit` yerine Windows process cikisina bagla.
- [x] `printf` ve `getchar` bagimliliklarini runtime I/O katmaniyla azalt.
- [x] Native runtime I/O icin sayi, bos metin ve UTF-8 edge testleri ekle.
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
