# Catat

Aplikasi desktop portabel untuk mencatat kas dan stok toko. Dibuat untuk toko perikanan
(pakan ikan dan perlengkapannya), tempat barang yang sama datang berkali-kali dengan
spek dan harga berbeda dan stok lama harus habis lebih dulu.

- Stok dicatat **per kiriman**; saat barang keluar, kiriman terlama diambil otomatis.
- Kas penjualan dan pembelian tercatat otomatis; kas lain dicatat manual; bisa beberapa akun (Tunai, Bank).
- Beranda dengan grafik arus kas dan arus barang, stok lama, stok menipis, dan yang mendekati kedaluwarsa.
- Lima laporan (arus kas, laba kotor, posisi stok, kartu stok, barang masuk/keluar), ekspor Excel dan PDF.
- Login wajib, hak akses per pengguna, kunci otomatis, log aktivitas, cadangkan/pulihkan.
- Foto barang dan kiriman.
- Semua data di SQLite, di folder `data/` di sebelah program. Tanpa internet.

Panduan untuk pengguna: `docs/Panduan-Pengguna-Catat.pdf` (dibuat oleh `docs/panduan/buat_panduan.py`, butuh `reportlab` dan `pillow`).

Rancangan lengkap: `docs/superpowers/specs/2026-11-24-catat-design.md`.

## Memakai (Linux)

Ekstrak `Catat-linux-x86_64.tar.gz`, lalu jalankan `./catat` di dalam folder `Catat`.
Butuh WebKitGTK 4.1 (`webkit2gtk-4.1` di Arch, `libwebkit2gtk-4.1-0` di Debian/Ubuntu).

```
Catat/
├── catat            program
├── BACA-DULU.txt
└── data/            dibuat saat pertama dijalankan
    ├── catat.db
    ├── foto/
    └── cadangan/
```

Untuk pindah komputer atau flashdisk, salin seluruh folder `Catat`. Kalau folder itu
tidak bisa ditulis, aplikasi menampilkan pesan dan tidak menyimpan data di tempat lain.

Satu folder data hanya bisa dibuka satu jendela Catat pada satu waktu; jendela kedua menampilkan pesan.

Di luar folder itu, WebKit menyimpan cache dan pilihan mode terang/gelap di
`~/.local/share/id.catat.app` dan `~/.cache/id.catat.app`. Tidak ada catatan toko di sana.

## Memakai (Windows)

Ekstrak `Catat-windows-x86_64.zip`, lalu jalankan `catat.exe` di dalam folder `Catat`
(`WebView2Loader.dll` harus tetap di sebelahnya). Butuh Windows 10/11 64-bit dengan
WebView2 Runtime. Data berada di `Catat\data\`, sama seperti di Linux. WebView2 menyimpan
cache di `%LOCALAPPDATA%\id.catat.app`; tidak ada catatan toko di sana.

Versi Windows dibangun dengan kompilasi silang dari Linux dan **belum pernah dijalankan
di Windows**; lihat bagian "Belum ada".

## Membangun

Prasyarat: Rust (rustup), Node 22+, pnpm, dan pustaka pengembangan WebKitGTK 4.1.

```sh
pnpm install
pnpm tauri dev                 # mode pengembangan
scripts/paket-linux.sh         # paket portabel Linux di dist-portable/
scripts/paket-windows.sh       # paket portabel Windows (butuh mingw-w64, lihat isi skrip)
```

Saat pengembangan, `CATAT_DATA_DIR=/suatu/folder` memindahkan folder data.

## Pemeriksaan

```sh
(cd src-tauri && cargo test && cargo clippy --all-targets -- -D warnings)
pnpm test && pnpm vue-tsc --noEmit
```

Uji antarmuka pada aplikasi sungguhan (butuh layar; memakai build debug):

```sh
pnpm tauri build --debug --no-bundle
uji-ui/jalankan.sh uji-ui/01-setup-login.js
uji-ui/jalankan.sh uji-ui/02-barang-stok.js
uji-ui/jalankan.sh uji-ui/03-kas-beranda-laporan.js
CATAT_UJI_BATAS=1500 uji-ui/jalankan.sh uji-ui/04-pengguna-pengaturan.js   # menunggu kunci otomatis 1 menit
```

Skenario 02 dan 03 memakai tanggal contoh di tahun 2026. Alat bantu uji
(`src-tauri/src/uji_ui.rs`) hanya ikut di build debug.

## Susunan kode

- `src-tauri/src/`: semua aturan. `akses` (sesi dan hak), `auth`, `pengguna`, `barang`, `stok`
  (kiriman, alokasi, batal, penyesuaian), `kas`, `ringkasan`, `laporan`, `ekspor`, `cadangan`.
  `perintah/` hanya pembungkus perintah Tauri. Uji di `uji/`.
- `src/`: tampilan Vue. `pages/` satu berkas per layar, `lib/format.ts` untuk angka dan tanggal gaya Indonesia.
- Uang disimpan sebagai Rupiah bulat; jumlah barang per seperseribu satuan.

## Belum ada

Versi Windows sudah dibangun tetapi belum diuji di Windows sungguhan: uji otomatis dan uji
antarmuka hanya dijalankan di Linux. Di luar versi ini: hutang/piutang, konversi satuan,
cetak struk, barcode, banyak toko, enkripsi file database.
