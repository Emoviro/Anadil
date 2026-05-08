# Anadil Local IDE

Anadil local IDE, `anadil ide` komutuyla calisan hafif bir yerel web arayuzudur.

## Calistirma

```powershell
cargo run -- ide
```

Komut bos bir port bulur ve terminale su formatta adres yazar:

```text
Anadil IDE hazir: http://127.0.0.1:5817
```

## Ozellikler

- Ornek `.ana` dosyalarini sol panelden yukleme
- Yerel `.ana` dosyasi acma
- Dosya kaydetme
- Kod editoru, satir numaralari ve cursor konumu
- `Kontrol` butonu: `/api/check` uzerinden compiler diagnostic
- `Calistir` butonu: `/api/run` uzerinden interpreter output
- `EXE Derle` butonu: `/api/build` uzerinden native build sonucu
- Alt panelde output ve diagnostics gorunumu

## API

IDE server'i mevcut compiler protokolunu kullanir ve tarayici tarafina JSON dondurur:

```text
POST /api/check
POST /api/run
POST /api/build
GET  /api/examples
GET  /api/example?name=<ornek.ana>
```

`/api/build`, aktif editor icerigini `target/ide/ide_current.ana` dosyasina yazar ve mevcut Anadil executable'i uzerinden `derle --json` calistirir.

## Sinirlar

- Su an tek dosyali editor modeli vardir.
- Browser destekliyorsa File System Access API ile dogrudan kaydeder; destek yoksa indirme fallback'i kullanir.
- Debugger, autocomplete ve syntax highlighting henuz yoktur.
