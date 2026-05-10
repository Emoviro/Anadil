# Anadil V0.1 Release Layout

Bu belge `Anadil-v0.1.0-windows-x64.zip` arsiviniin ic yapisini, hangi
dosyanin nereden geldigini ve hangi adimda uretildigini tanimlar. Hem
elle paketleme hem GitHub Actions otomatik release icin tek dogruluk
kaynagidir.

## Hedefler

- Tek `.zip` arsivi indir → cikart → calistir.
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
└── LICENSE.txt                      (TODO: lisans karari verilmeli)
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
| `LICENSE.txt` | `LICENSE` (henuz yok, TODO) | direkt kopya |

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

1. `link.exe` PATH'te varsa veya `vcvars64.bat` bulunabiliyorsa → `derle`
   calisir, pre-built `runtime/anadil_runtime.lib`'i kullanir.
2. Yoksa kullaniciya net hata mesaji + Build Tools indirme linki.

Pre-built `.lib` shipping `ml64`/`lib` adimini atlatir. Sadece `link.exe`
yeterlidir. Kullanici opsiyonel olarak `runtime_asm` source'undan
yeniden build etmek isterse `runtime/anadil_runtime.asm`'a erisimi var.

## Paketleme akisi (`package.ps1`)

```
1. cargo build --release --bin anadil
2. cargo build --release --bin anadil-ide
3. ml64 + lib ile target/release/anadil_runtime.lib uret
4. target/dist/Anadil-v0.1.0/ klasorunu temizle ve olustur
5. Tum dosyalari yukaridaki layout'a kopyala
6. README.txt'yi ic icerikten uret (sürüm, tarih, kisa baslangic)
7. target/dist/Anadil-v0.1.0-windows-x64.zip olarak sikistir
8. SHA256 hash uret (release sayfasinda gosterilir)
```

## CI/CD akisi (`.github/workflows/release.yml`)

GitHub Actions tag push'unda (`v*`) tetiklenir:

```
1. windows-latest runner kullan (Build Tools varsayilan kurulu)
2. Rust toolchain setup (`actions-rs` veya `dtolnay/rust-toolchain`)
3. package.ps1 calistir
4. Cikan ZIP'i actions/upload-release-asset ile release'e ekle
5. SHA256 hash'i release notes'a ekle
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

Bu dosya yazildiginda bekleyen konular:

1. **LICENSE** secimi — MIT, Apache-2.0, GPL-3.0 vb. arasindan secilmeli.
   ZIP icinde olmasi gereken bir dosya; release'den once eklenmeli.
2. **`runtime_asm_path()` exe-relative arama** — Codex tarafinda
   bekleyen iş; bu yapilmadan ZIP shipping anlamli olmaz (`derle`
   komutu kullanicinin makinesinde yanlis path aratir).
3. **Build Tools algilama mesaji iyilestirmesi** — Codex tarafinda
   bekleyen iş; KURULUM.txt'in soracagi metin ile uyumlu olmali.

Bu uc madde release tarihini belirler. `LICENSE` lisans tercihi olarak
kullanicidan beklenir; diger ikisi Codex'in development track'ine
baglidir.
