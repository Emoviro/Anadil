# Anadil

Anadil, Turkce anahtar kelimelerle yazilan kucuk bir programlama dili denemesidir.
V1 hedefi sade, statik tipli ve genisletilebilir bir cekirdek olusturmaktir.

Proje su anda kaynak dosyayi okuyabilen, lexer/parser/semantic analiz yapan ve typed AST uzerinden programi calistiran bir interpreter icerir.

## Durum

Yapilanlar:

- `sayı` ve `mantık` temel tipleri
- Degisken tanimlama ve atama
- Aritmetik islemler: `+`, `-`, `*`, `/`
- Unary eksi: `-10`, `-x`, `10 + -3`
- Karsilastirma islemleri: `==`, `!=`, `<`, `>`, `<=`, `>=`
- `eğer` / `değilse`
- Sonsuz, kosullu ve sayacli `döngü`
- `kır`, `devam`, `dön`
- Fonksiyon tanimlama ve fonksiyon cagirma
- `Ana()` giris noktasi
- `yazdir` yerlesik fonksiyonu
- `//` satir yorumlari
- CLI komutlari: `calistir`, `kontrol`, `ast`, `typed`, `ornekler`, `surum`, `yardim`
- Etkilesimli REPL komutu: `repl`

Henuz yapilmayanlar:

- Native codegen
- String/metin tipi
- Dizi, struct, class, modul sistemi
- Dosya paketleme veya kurulum araci

## Calistirma

Varsayilan calistirma:

```powershell
cargo run -- examples\topla.ana
```

Acik komutla calistirma:

```powershell
cargo run -- calistir examples\topla.ana
```

Program gecerli mi kontrol etme:

```powershell
cargo run -- kontrol examples\topla.ana
```

Parse edilmis AST'yi yazdirma:

```powershell
cargo run -- ast examples\topla.ana
```

Semantic analizden sonraki typed AST'yi yazdirma:

```powershell
cargo run -- typed examples\topla.ana
```

Ornek dosyalari listeleme:

```powershell
cargo run -- ornekler
```

Surum bilgisi:

```powershell
cargo run -- surum
```

Yardim:

```powershell
cargo run -- yardim
```

Etkilesimli REPL:

```powershell
cargo run -- repl
```

REPL icinde:

```text
> yazdir(10);
10
> yazdir(10 + -3);
7
> Kare(x: sayı) -> sayı {
|     dön x * x;
| }
Fonksiyon kaydedildi.
> yazdir(Kare(5));
25
> :cik
```

Not: REPL cok satirli girisi destekler ve fonksiyon tanimlarini oturum boyunca saklar. Degiskenler satirlar arasinda saklanmaz.

## Ornek

```ana
// Iki sayıyı toplar.
Topla(a: sayı, b: sayı) -> sayı {
    dön a + b;
}

Ana() {
    sonuc: sayı = Topla(10, 20);
    yazdir(sonuc);
}
```

## Gelistirme

### Mimari

Proje iki parcaya ayrilmistir:

- `src/lib.rs`: Dil motoru. Lexer, parser, semantic analiz, typed AST ve interpreter burada kutuphane olarak disari acilir.
- `src/main.rs`: CLI katmani. Dosya okur, komutlari yorumlar ve `lib.rs` icindeki pipeline fonksiyonlarini cagirir.

Kutuphane tarafinda uc ana giris fonksiyonu vardir:

```rust
anadil::parse_source(source)
anadil::compile_source(source)
anadil::run_source(source)
```

### Komutlar

Format kontrolu:

```powershell
cargo fmt --check
```

Testler:

```powershell
cargo test
```

Bu komut unit testleri, CLI testlerini ve `examples/` altindaki ornek programlarin integration testlerini calistirir.

Clippy:

```powershell
cargo clippy --all-targets --all-features -- -D warnings
```

## Dokumantasyon

- Guncel dil referansi: [Docs/dil_referansi.md](Docs/dil_referansi.md)
- Ornek programlar: [examples/README.md](examples/README.md)
