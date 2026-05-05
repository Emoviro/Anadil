# Ornek Programlar

Bu klasor Anadil V1'in calisan ve bilerek hatali orneklerini icerir.

Tum ornekleri CLI uzerinden listelemek icin:

```powershell
cargo run -- ornekler
```

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
