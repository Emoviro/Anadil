# Anadil V1 Dil Referansi

Bu dosya projenin su anki calisan davranisini ozetler.

## Program Yapisi

Her program parametresiz bir `Ana()` fonksiyonu icermelidir.

```ana
Ana() {
    yazdir(10);
}
```

## Tipler

V1'de iki temel tip vardir:

```ana
sayı
mantık
```

## Degisken Tanimlama

```ana
x: sayı = 10;
durum: mantık = doğru;
```

Degisken taniminda tip zorunludur.

## Atama

```ana
x = 30;
durum = yanlış;
```

Atanan deger degiskenin tipiyle ayni olmalidir.

## Sayilar ve Mantik Degerleri

```ana
10
-10
doğru
yanlış
```

Unary eksi sayilar icin gecerlidir:

```ana
x: sayı = -10;
yazdir(-x);
yazdir(10 + -3);
```

## Yorum Satirlari

`//` satir sonuna kadar yorum kabul edilir.

```ana
// Bu satir calistirilmaz.
yazdir(10);
```

## Aritmetik Operatorler

```ana
+
-
*
/
```

Ornek:

```ana
sonuc: sayı = (10 + 20) * 2;
```

## Karsilastirma Operatorleri

```ana
==
!=
<
>
<=
>=
```

Karsilastirmalar `mantık` degeri uretir.

```ana
yazdir(10 > 5);
```

## Kosul

Kosul parantez icinde yazilir.

```ana
eğer (x > 10) {
    yazdir(x);
} değilse {
    yazdir(0);
}
```

## Donguler

Sonsuz dongu:

```ana
döngü {
    yazdir(1);
}
```

Kosullu dongu:

```ana
döngü (x < 10) {
    x = x + 1;
}
```

Sayacli dongu:

```ana
döngü (i: sayı = 0; i < 10; i = i + 1) {
    yazdir(i);
}
```

Dongu kontrol ifadeleri:

```ana
kır;
devam;
```

## Fonksiyonlar

Fonksiyon tanimlamak icin ayri bir anahtar kelime yoktur.

```ana
Topla(a: sayı, b: sayı) -> sayı {
    dön a + b;
}
```

Dönüş tipi olmayan fonksiyon:

```ana
YazdirDeger(x: sayı) {
    yazdir(x);
}
```

Dönüş tipi belirtilirse tum kontrol yolları deger dondurmelidir.

## Yerlesik Fonksiyonlar

Su anda tek yerlesik fonksiyon vardir:

```ana
yazdir(deger);
```

`yazdir` deger dondurmez. Bu yuzden su gecersizdir:

```ana
x: sayı = yazdir(10);
```

## Hata Kontrolleri

Semantic analiz su durumlari yakalar:

- `Ana()` eksikligi
- `Ana()` fonksiyonunun parametre almasi
- `Ana()` fonksiyonunun donus tipi belirtmesi
- Tip uyumsuzlugu
- Tanimlanmamis degisken kullanimi
- Tanimlanmamis fonksiyon cagrisi
- Yanlis arguman sayisi veya tipi
- `kir` / `devam` ifadelerinin dongu disinda kullanilmasi
- `yazdir` sonucunun deger gibi kullanilmasi

