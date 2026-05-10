# README "İndir ve Kullan" Bölümü (Taslak)

Bu dosya `README.md`'ye eklenmek üzere hazırlanmış taslaktır. Aşağıdaki
bölümün `## Durum` ile `## Calistirma` arasına eklenmesi önerilir;
böylece son kullanıcı "ben nasıl indirip kullanırım" sorusunu en başta
görür, geliştirici dokümantasyonu (Calistirma → cargo run vb.) altta
kalır.

`v0.1.0` etiketi ve GitHub Releases yayını yapıldıktan sonra link
çalışır hâle gelir; o ana kadar bağlantılar yer tutucu olarak durur.

---

## Eklenecek Bölüm

```markdown
## Indir ve Kullan

Anadil'i kullanmak icin Rust toolchain'ine veya kaynak kodu
derlemeye ihtiyaciniz yok. Hazir Windows x64 paketi:

[Anadil v0.1.0 — GitHub Releases](https://github.com/ArsenAlighieri/Anadil/releases/tag/v0.1.0)

Hizli kurulum:

1. ZIP arsivini indirip bir klasore (orn. `C:\Anadil`) cikartin.
2. `anadil-ide.exe` ile IDE'yi acin, ya da komut isteminden
   `anadil examples\topla.ana` ile native derleyip calistirin.
3. (Istege bagli) Klasoru `PATH` ortam degiskenine ekleyin.
4. (Native derleme icin) Visual Studio Build Tools kurulu degilse
   [indirin](https://visualstudio.microsoft.com/visual-cpp-build-tools/).
   Build Tools yalnizca `derle` komutu ve IDE'deki `EXE Derle` butonu
   icin gereklidir; interpreter modu (`yorumla`) ve IDE'nin diger
   ozellikleri Build Tools olmadan calisir.

Detayli kurulum talimati ZIP icindeki `KURULUM.txt` dosyasinda,
surum notlari `CHANGELOG.txt` icindedir.
```

---

## Entegrasyon talimatı

1. `README.md` dosyasını aç.
2. `## Durum` bölümünün sonu (mevcut "Bu CLI yuzeyi V0.1 icin sabit
   kabul edilir." satırı civarı) ile `## Calistirma` başlığı arasına
   yukarıdaki kod bloğunun **içeriğini** (üst ve alt ` ``` ` çizgileri
   olmadan) yapıştır.
3. Etiket / release yayınlandığında URL'in çalıştığını doğrula.

## Notlar

- v0.1.0 etiketi atılana kadar `https://github.com/ArsenAlighieri/Anadil/releases/tag/v0.1.0`
  bağlantısı 404 verir. Geçici olarak `https://github.com/ArsenAlighieri/Anadil/releases`
  (ana Releases sayfası) bağlantısı kullanılabilir.
- `Docs/release_layout.md` ve `KURULUM.txt` ile bu metnin tutarlı
  kalması önemli; özellikle Build Tools hakkında söylenen şeyler
  KURULUM.txt'tekiyle uyumlu.
- Başka bir GitHub kullanıcı/organizasyonuna taşınırsa
  (ör. `anadil/anadil`), URL'i güncellemek lazım.
