# Anadil V0.1 Local Release Validation

Tarih: 2026-05-10
Ortam: Windows, Visual Studio Build Tools mevcut

## Paketleme

- `pwsh -File .\package.ps1` lokal makinede calistirilamadi: `pwsh`
  PATH icinde yok.
- `powershell -NoProfile -ExecutionPolicy Bypass -File .\package.ps1`
  basariyla calisti.
- Uretilen ZIP:
  `target/dist/Anadil-v0.1.0-windows-x64.zip`
- SHA256:
  `71BE933CFDD339047188DC68AA65D1245524B69A641807AC59895C7A75EE256E`

## Icerik Kontrolu

`target/dist/Anadil-v0.1.0-windows-x64/` altinda beklenen dosyalar
dogrulandi:

- `anadil.exe`
- `anadil-ide.exe`
- `runtime/anadil_runtime.lib`
- `runtime/anadil_runtime.asm`
- `examples/*.ana`
- `docs/*.md`
- `KURULUM.txt`
- `CHANGELOG.txt`
- `LICENSE.txt`
- `README.txt`

## ZIP Smoke Test

ZIP ayri bir `target/release-validation-*` klasorune acildi.

- `anadil.exe yardim` basariyla calisti.
- `anadil.exe yorumla examples\topla.ana` basariyla calisti ve `30`
  yazdirdi.

## Installer

`makensis.exe` PATH icinde bulunamadi; lokal `-Installer` dogrulamasi
atlanmistir. CI workflow NSIS'i kendi kurdugu icin bu lokal ortam
eksikligi release blocker olarak isaretlenmedi.
