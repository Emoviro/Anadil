# Anadil Local IDE

Anadil iki local IDE yolu tasir:

- `anadil ide`: hafif yerel web arayuzu
- `anadil-ide.exe`: native executable GUI

## Web IDE

```powershell
cargo run -- ide
```

Komut bos bir port bulur ve terminale su formatta adres yazar:

```text
Anadil IDE hazir: http://127.0.0.1:5817
```

## Native IDE

```powershell
cargo run --bin anadil-ide
```

Release executable:

```powershell
cargo build --release --bin anadil-ide
target\release\anadil-ide.exe
```

Native IDE `eframe/egui` ile yazilir. Browser, localhost veya HTML/CSS gerektirmez.

Degisikliklerden sonra hizli elle kontrol icin [Native IDE Smoke Test](ide_smoke_test.md) listesini kullan.

Kisayollar:

```text
Ctrl+O  Dosya ac
Ctrl+S  Kaydet
F5      Secili modu calistir
Ctrl+B  EXE Derle
Ctrl+Shift+F5  EXE Calistir
```

`Ac` ve `Farkli Kaydet`, native dosya secme/kaydetme penceresi acar. Kaydedilmemis degisiklik varsa pencere basliginda ve aktif dosya adinda `*` gorunur.

Proje akisi:

- `Klasor Ac`, bir proje klasoru secer ve sol explorer'da `.ana` dosyalarini gosterir.
- Proje listesi alt klasorleri recursive tarar; `.git` ve `target` klasorlerini atlar.
- `Yenile`, proje dosya listesini yeniden okur.
- `Yeni`, editoru yeni bir `adsiz.ana` taslagina cevirir.
- `Olustur`, proje klasoru icinde yazilan adda `.ana` dosyasi olusturur.
- `Yeniden Adlandir` ve `Sil`, aktif dosya uzerinde calisir; silme islemi onay ister.
- Son acilan proje klasoru ve dosya bir sonraki native IDE acilisinda geri yuklenir.
- Kaydedilmemis degisiklik varken dosya degistirme veya yeni dosya acma onay ister.
- Ust bardaki mod secici `Interpret et`, `Compile et` ve `Karsilastir` modlarini tasir; `Yap` veya `F5` secili modu calistirir.
- `EXE Derle`, build oncesi aktif dosyayi kaydeder ve executable'i kaynak dosyanin yanina uretir.
- Build sekmesi derlenen kaynak dosyayi, uretilen `.exe` yolunu, exit/stdout/stderr detaylarini ve toolchain hatalarinda kisa cozum notunu gosterir.
- `EXE Calistir`, son uretilen executable'i calistirir ve stdout/stderr/exit code bilgisini `Build` sekmesine yazar.
- `Karsilastir`, interpreter ve native executable stdout sonuclarini ayni panelde karsilastirir.
- Native executable, Explorer'dan cift tiklaninca terminal penceresi kapanmadan once Enter bekler.
- Alt panelde `Cikti`, `Diagnostics` ve `Build` sekmeleri bulunur.
- `Diagnostics` sekmesindeki satir/sutun bilgili hata kartlarina tiklaninca editor ilgili konuma odaklanir.
- Native editor ve explorer VS Code benzeri koyu tema ve ince resize ayiricilari kullanir; diagnostic kartina tiklaninca ilgili kod konumuna odaklanir.

## Ortak Ozellikler

- Ornek `.ana` dosyalarini sol panelden yukleme
- Native IDE'de proje klasoru acma ve `.ana` dosyalarini sol explorer'dan secme
- Yerel `.ana` dosyasi acma
- Dosya kaydetme
- Kaydedilmemis degisiklik gostergesi
- Kod editoru ve syntax highlighting
- Web IDE'de satir numaralari, cursor konumu, canli diagnostics ve uzun dosyalar icin sona gitme dugmesi
- Native IDE'de dogrudan compiler API ile `Kontrol`, `Calistir` ve `EXE Derle`
- Web IDE'de `Kontrol`, `Calistir` ve `EXE Derle` mevcut JSON API endpoint'lerini kullanir
- Alt panelde output, diagnostics ve build gorunumu

## Web IDE API

Web IDE server'i mevcut compiler protokolunu kullanir ve tarayici tarafina JSON dondurur:

```text
POST /api/check
POST /api/run
POST /api/build
GET  /api/examples
GET  /api/example?name=<ornek.ana>
```

`/api/build`, aktif editor icerigini `target/ide/ide_current.ana` dosyasina yazar ve mevcut Anadil executable'i uzerinden `derle --json` calistirir.

## Sinirlar

- Web IDE su an tek dosyali editor modeliyle calisir.
- Native IDE proje explorer, dosya olusturma, yeniden adlandirma ve silme islemlerini sunar.
- Browser destekliyorsa File System Access API ile dogrudan kaydeder; destek yoksa indirme fallback'i kullanir.
- Debugger ve autocomplete henuz yoktur.
