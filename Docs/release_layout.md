# Anadil V0.1 Release Layout

Bu belge `Anadil-v0.1.0-windows-x64.zip` arsivinin ic yapisini, hangi
dosyanin nereden geldigini ve hangi adimda uretildigini tanimlar. Hem
elle paketleme hem GitHub Actions otomatik release icin tek dogruluk
kaynagidir.

## Hedefler

- Tek `.zip` arsivi indir → cikart → calistir.
- Alternatif olarak NSIS Setup sihirbazi (`Anadil-Setup-vX.Y.Z.exe`):
  per-user kurulum, opsiyonel PATH ekleme, file association, kaldirma
  sihirbazi.
- Microsoft Build Tools dogrudan kullaniciya yuk; redistribute edilmez.
- ZIP icindeki dosyalar self-explanatory; KURULUM.txt yeterli olmali.
- ZIP boyutu makul (~10-20 MB hedef). Pre-built `.lib` dahil; `.exe`
  iki tane.

## Hedef yol ve dosya yapisi

```
Anadil-v0.1.0/
├── anadil.exe                       (CLI: yorumla / derle / kontrol / ide / repl)
├── anadil-ide.exe                   (Native IDE)
├── runtime/
│   ├── anadil_runtime.asm           (kaynak; sefaflik icin)
│   └── anadil_runtime.lib           (pre-built; ml64+lib gerekmesin)
├── examples/
│   ├── topla.ana
│   ├── fonksiyon.ana
│   ├── dongu.ana
│   ├── kosul.ana
│   ├── kosullu_dongu.ana
│   ├── sonsuz_dongu.ana
│   ├── kapsam.ana
│   ├── mantik.ana
│   ├── metin.ana
│   ├── negatif.ana
│   ├── native_mvp.ana
│   ├── hata_tip.ana                 (bilerek hatali)
│   └── hata_ana_yok.ana             (bilerek hatali)
├── docs/
│   ├── PROJE_RAPORU.md              (Docs/proje_raporu.md kopyasi)
│   ├── DIL_REFERANSI.md             (Docs/dil_referansi.md kopyasi)
│   └── NATIVE_COMPILER.md           (Docs/native_compiler.md kopyasi)
├── KURULUM.txt                      (kullanici icin kurulum talimati)
├── CHANGELOG.txt                    (release notes)
├── README.txt                       (5-6 satirlik baslangic)
└── LICENSE.txt                      (MIT lisansi)
```

## Dosya kaynaklari

| Hedef dosya | Kaynak | Adim |
|---|---|---|
| `anadil.exe` | `cargo build --release --bin anadil` | package adim 1 |
| `anadil-ide.exe` | `cargo build --release --bin anadil-ide` | package adim 1 |
| `runtime/anadil_runtime.asm` | `runtime/anadil_runtime.asm` | direkt kopya |
| `runtime/anadil_runtime.lib` | `ml64 + lib` ile pre-build | package adim 2 |
| `examples/*.ana` | `examples/*.ana` | direkt kopya |
| `docs/PROJE_RAPORU.md` | `Docs/proje_raporu.md` | rename + kopya |
| `docs/DIL_REFERANSI.md` | `Docs/dil_referansi.md` | rename + kopya |
| `docs/NATIVE_COMPILER.md` | `Docs/native_compiler.md` | rename + kopya |
| `KURULUM.txt` | `KURULUM.txt` (repo root, packaging icin yazildi) | direkt kopya |
| `CHANGELOG.txt` | `CHANGELOG.txt` (repo root) | direkt kopya |
| `README.txt` | inline package script icinde uretilir | dinamik |
| `LICENSE.txt` | `LICENSE` | direkt kopya |

## Bagimliliklar

ZIP'in production icin ihtiyac duyduklari:

- `cargo` (Rust toolchain) — repo derlemesi icin, kullaniciya gerekmez.
- `ml64.exe` ve `lib.exe` (Visual Studio Build Tools) — runtime `.lib`
  pre-build icin, kullaniciya gerekmez (zaten pre-built ship ediliyor).

ZIP'i indirip kullanan son kullanicinin ihtiyaclari:

- Windows 10/11 x64.
- Visual Studio Build Tools (`link.exe` icin) — **ZORUNLU**, sadece
  `derle` komutu icin. Interpreter ve IDE Build Tools olmadan calisir.

## Pre-built `.lib` mantigi

`anadil.exe`'nin Build Tools kontrolu sirasi:

1. `ml64.exe` ve `link.exe` PATH'te varsa veya `vcvars64.bat`
   bulunabiliyorsa `derle` calisir.
2. Paketlenmis release'de exe yanindaki `runtime/anadil_runtime.lib`
   dogrudan kullanilir; runtime yeniden assemble edilmez.
3. Kaynak agacindan gelistirme calismasinda pre-built lib yoksa runtime
   cache'i `ml64 + lib` ile yeniden uretilir.
4. Build Tools yoksa kullaniciya net hata mesaji + Build Tools indirme
   linki.

Pre-built `.lib` shipping runtime icin `ml64`/`lib` adimini atlatir.
Program assembly'si icin `ml64.exe`, final executable icin `link.exe`
gerekir. Kullanici opsiyonel olarak `runtime_asm` source'undan yeniden
build etmek isterse `runtime/anadil_runtime.asm`'a erisimi var.

## Paketleme akisi (`package.ps1`)

```
1. cargo build --release --bin anadil
2. cargo build --release --bin anadil-ide
3. ml64 + lib ile target/release-runtime/anadil_runtime.lib uret
4. target/dist/Anadil-v0.1.0/ klasorunu temizle ve olustur
5. Tum dosyalari yukaridaki layout'a kopyala
6. README.txt'yi ic icerikten uret (sürüm, tarih, kisa baslangic)
7. target/dist/Anadil-v0.1.0-windows-x64.zip olarak sikistir
8. SHA256 hash uret (release sayfasinda gosterilir)
```

### Installer (opsiyonel, `-Installer` flag'i)

`-Installer` flag'i ile script ek olarak NSIS setup sihirbazi uretir:

```
9. installer.nsi'yi makensis ile derle
10. target/dist/Anadil-Setup-v0.1.0.exe olarak yaz
11. Setup SHA256 hash uret
```

NSIS, Anadil'in `dist` klasorunu okur ve dosyalari bu klasorden alir;
yani Setup uretimi her zaman ZIP uretiminden sonra calistirilmali. Setup
hedefi: per-user kurulum, opsiyonel PATH/Start menu/file association,
standart kaldirma sihirbazi.

## CI/CD akisi (`.github/workflows/release.yml`)

GitHub Actions tag push'unda (`v*`) tetiklenir:

```
1. windows-2022 runner
2. Rust toolchain (dtolnay/rust-toolchain@stable)
3. MSVC env (ilammy/msvc-dev-cmd@v1) — ml64/lib/link aktif
4. cargo test --release
5. choco install nsis -y
6. pwsh -File .\package.ps1 -Installer
7. ZIP ve Setup yollarini step output'una yaz
8. softprops/action-gh-release@v2 ile release olustur
   - hem ZIP hem Setup yuklenir
   - release notes ZIP+Setup boyutu ve SHA256 ile uretilir
```

## Sürüm numarasi semantigi

- `v0.1.0` — ilk kamuya acik release
- `v0.1.x` — bug fix'leri (semver patch)
- `v0.2.0` — heap modeli ve dinamik metin (semver minor)
- `v1.0.0` — V0.1 + V0.2 + V0.3 dil ve runtime'inin sabitlendigi tarih

`Cargo.toml` `version` alani ZIP isminin tek dogruluk kaynagidir.

## Test akisi

Release oncesi temiz makinede dogrulanmasi gerekenler:

- [ ] ZIP'i indirme + cikartma sorunsuz
- [ ] `anadil.exe yardim` calisir
- [ ] `anadil.exe yorumla examples\topla.ana` calisir (interpreter)
- [ ] `anadil.exe derle examples\topla.ana` calisir (Build Tools varsa)
- [ ] Uretilen `.exe` dosyasi calisir
- [ ] `anadil-ide.exe` acilir
- [ ] IDE'de ornek dosya acilir, kontrol/derle/calistir butonlari calisir
- [ ] Build Tools yok ise `derle` net hata mesajini gosterir
- [ ] Build Tools yok ise IDE'de "EXE Derle" disable + tooltip aciklar

## Acik Konular

Release oncesi kalan dogrulama isi temiz makine uzerinde ZIP ve Setup
smoke testidir. Kod tarafinda release'i bloklayan bilinen paketleme
konusu yoktur.
