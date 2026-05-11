# Ornek Programlar

Bu klasor Anadil V1'in calisan ve bilerek hatali orneklerini icerir.

Tum ornekleri CLI uzerinden listelemek icin:

```powershell
cargo run -- ornekler
```

Bir ornegi native executable olarak derlemek icin:

```powershell
cargo run -- derle examples\topla.ana
examples\topla.exe
```

Native derleme Windows x64 hedefler ve Visual Studio Build Tools C++ araclarini kullanir.

## Calisan Ornekler

Toplama:

```powershell
cargo run -- calistir examples\topla.ana
```

Beklenen cikti:

```text
30
```

Sayacli dongu, `devam` ve `kir`:

```powershell
cargo run -- calistir examples\dongu.ana
```

Beklenen cikti:

```text
0
2
3
```

Kosul:

```powershell
cargo run -- calistir examples\kosul.ana
```

Beklenen cikti:

```text
15
```

Ic ice fonksiyon cagrilari:

```powershell
cargo run -- calistir examples\fonksiyon.ana
```

Beklenen cikti:

```text
25
```

Mantik degerleri:

```powershell
cargo run -- calistir examples\mantik.ana
```

Beklenen cikti:

```text
doğru
yanlış
doğru
yanlış
```

Metin degerleri:

```powershell
cargo run -- calistir examples\metin.ana
```

Beklenen cikti:

```text
Merhaba Anadil
Yerel Derleyici
doğru
```

V0.2 dinamik metin ve `uzunluk`:

```powershell
cargo run -- calistir examples\metin_v02.ana
```

Beklenen cikti:

```text
Merhaba Anadil
14
0
2
15
doğru
```

Kosullu dongu:

```powershell
cargo run -- calistir examples\kosullu_dongu.ana
```

Beklenen cikti:

```text
0
1
2
```

Sonsuz dongu ve `kir`:

```powershell
cargo run -- calistir examples\sonsuz_dongu.ana
```

Beklenen cikti:

```text
0
1
2
```

Scope/kapsam:

```powershell
cargo run -- calistir examples\kapsam.ana
```

Beklenen cikti:

```text
2
1
```

Negatif sayilar:

```powershell
cargo run -- calistir examples\negatif.ana
```

Beklenen cikti:

```text
-10
7
10
```

Native MVP demo programi:

```powershell
cargo run -- calistir examples\native_mvp.ana
```

Beklenen cikti:

```text
7
15
doÄŸru
native
```

## Hatali Ornekler

Tip hatasi:

```powershell
cargo run -- kontrol examples\hata_tip.ana
```

Beklenen davranis: `sayı` degiskene `mantık` atanamayacagini soyleyen semantic hata.

`Ana()` eksik:

```powershell
cargo run -- kontrol examples\hata_ana_yok.ana
```

Beklenen davranis: program giris noktasi icin `Ana()` gerektigini soyleyen semantic hata.
