# Native IDE Smoke Test

Bu liste, native IDE degisikligi yapmadan once ve sonra hizli elle kontrol icindir.

## Son Sonuc

2026-05-10 otomatik kontrol:

- [x] `cargo build --release --bin anadil-ide`
- [x] `cargo test --bin anadil-ide`
- [ ] Manuel GUI smoke akisi henuz bu oturumda uygulanmadi.

Not: Codex oturumunda native GUI penceresini guvenilir bicimde gozlemleyemedigimiz
icin editor/dosya acma/tiklama akisi elle kontrol bekliyor. Release binary
basariyla uretildi: `target\release\anadil-ide.exe`.

## Hazirlik

```powershell
cargo build --release --bin anadil-ide
target\release\anadil-ide.exe
```

Test icin proje icinde gecici bir `.ana` dosyasi kullan.

## Editor ve Dosya

- IDE acilirken editor yazilabilir olmali.
- Yeni dosyada su kod yazilabilmeli:

```ana
Ana() {
    yazdır(10);
}
```

- `Kaydet` dosyayi `.ana` olarak kaydetmeli.
- IDE kapatip acinca son proje/dosya geri yuklenmeli.
- Sol explorer uzun dosya listesinde kaydirilabilmeli.
- `yazdır` ana builtin olarak calismali.
- `yazdir` ASCII alias olarak calismali.

## Interpret

- Run mode `Interpret et` sec.
- `Yap` veya `F5` calistir.
- `Cikti` sekmesinde `10` gorunmeli.
- Hata yoksa `Diagnostics` sekmesi bos olmali.

## Compile

- Run mode `Compile et` sec.
- `Yap` veya `Ctrl+B` calistir.
- `Build` sekmesi sunlari gostermeli:
  - kaynak `.ana` yolu
  - uretilen `.exe` yolu
  - stdout/stderr bolumleri
- `.exe` dosyasi kaynak dosyanin yaninda olusmali.

## EXE Calistir

- `EXE Calistir` dugmesine bas.
- `Build` sekmesinde exit code ve stdout gorunmeli.
- Explorer'dan `.exe` cift tiklaninca terminal hemen kapanmadan Enter beklemeli.

## Karsilastir

- Run mode `Karsilastir` sec.
- `Yap` veya `F5` calistir.
- Interpreter stdout ve native stdout ayniysa sonuc `AYNI` olmali.
- Fark varsa `Diagnostics` native farki gostermeli.

## Hata Senaryolari

Tip hatasi:

```ana
Ana() {
    x: sayı = doğru;
    yazdır(x);
}
```

- `Kontrol` Diagnostics sekmesinde satir/sutun bilgili hata gostermeli.
- Diagnostic kartina tiklayinca editor ilgili konuma odaklanmali.

Runtime hatasi:

```ana
Ana() {
    yazdır(10 / 0);
}
```

- Interpret runtime hatasini Diagnostics sekmesinde gostermeli.
- Native build veya run hatasi varsa Build sekmesi ham stdout/stderr ve kisa cozum notu gostermeli.

## Gecici Notlar

- Satir numarasi gutter'i bilincli olarak kapali. Geri getirilecekse once editor yazma erisimi bu listeyle tekrar kontrol edilmeli.
- Gorsel polish, bu smoke test temiz gecmeden yapilmamali.
