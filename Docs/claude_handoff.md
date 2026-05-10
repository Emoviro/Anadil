# Claude Handoff Talimatlari

Bu dosya, Codex yerine gecici olarak Claude Code kullanilirken projenin
yonunu korumak icin yazildi. Amac hiz kazanmak degil, repo disiplinini
bozmadan guvenli yardim almaktir.

## Once Oku

Claude herhangi bir kod degistirmeden once su dosyalari okumali:

1. `Docs/project_status.md`
2. `Docs/native_compiler.md`
3. `Docs/memory_model.md`
4. `Docs/todo.md`
5. Ilgili task'a gore `README.md`

## Ana Yon

- Anadil C'ye transpile etmeyecek.
- Native hedef Windows x64 MASM assembly + Anadil runtime library.
- Runtime su an `runtime/anadil_runtime.asm` icinde.
- Build hatti `ml64`, `lib`, `link`, `kernel32.lib` ile calisiyor.
- V0.1 hedefi: compiler/runtime omurgasini sabitlemek.
- V0.1'de heap, GC, RC, dizi, yapi, dynamic string yok.
- V0.2+ icin hedef memory modeli RC.

## Guvenli Gorevler

Claude'a verilebilecek guvenli isler:

- Dokuman okuma ve ozetleme.
- `Docs/*.md` icinde tutarlilik duzeltmeleri.
- Test senaryosu onermek.
- README/dokuman dili sadeleştirmek.
- Kod review yapmak, ama degisiklik yapmadan bulgu listelemek.
- V0.1 tamam kriterlerini taslaklamak.
- IDE polish icin fikir vermek, kod degistirmeden.

## Riskli Gorevler

Claude su alanlarda dikkatli olmali veya sadece analiz yapmali:

- `src/native.rs`
- `src/main.rs` native build hatti
- `runtime/anadil_runtime.asm`
- `tests/native_edge_cases.rs`
- `src/bin/anadil-ide.rs`

Bu dosyalarda degisiklik gerekiyorsa:

1. Once kisa plan yaz.
2. Sadece ilgili dosyalara dokun.
3. Buyuk refactor yapma.
4. Mevcut commit stilini bozma.
5. Test komutlarini belirt.

## Yasak / Kacinilacaklar

- `git reset --hard`, `git checkout --`, destructive cleanup yapma.
- C transpiler onermeye geri donme.
- Runtime'i tekrar C runtime'a baglama (`printf`, `getchar`, `strcmp`,
  C `exit` geri gelmemeli).
- IDE gorsel revizyonuna buyuk girme.
- Aynı anda compiler, runtime ve IDE'yi birlikte refactor etme.
- V0.1 kapsamına heap/GC/RC implementasyonu sokma.

## Tercih Edilen Task Formati

Claude'a is verirken su format kullanilsin:

```text
Bu repo Anadil native compiler projesi.
Once Docs/project_status.md ve Docs/todo.md oku.
Sadece [dosya/dosyalar] uzerinde calis.
Buyuk refactor yapma.
Degisiklikten sonra hangi testlerin calismasi gerektigini yaz.
```

## Su Anki En Mantikli Sonraki Isler

1. `Docs/project_status.md` ve `Docs/todo.md` uzerinden V0.1 compiler
   tamam kriterlerini yazmak.
2. Native/interpreter test kapsaminda eksik dil orneklerini listelemek.
3. Runtime library paketleme modelini dokuman olarak netlestirmek.
4. Windows API bagimli runtime katmaninin ileride nasil soyutlanacagini
   kisa tasarim notu olarak yazmak.

## Commit Disiplini

- Her commit kucuk ve acik olmali.
- Commit mesajlari proje tarziyla uyumlu olmali:

```text
Add ...
Update ...
Document ...
Fix ...
```

- Claude kullanildigi commit mesajinda belirtilmemeli.
- Commit diff'i kendi basina aciklanabilir olmali.

## Test Notu

Docs-only degisikliklerde test sart degil. Kod degisirse uygun testler:

```powershell
cargo fmt --check
cargo check
cargo test --test native_edge_cases
cargo test --test native_examples
```

IDE degisirse ayrica manuel smoke test gerekir; `Docs/ide_smoke_test.md`
okunmali.

