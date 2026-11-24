# Catat — Desain Aplikasi Pencatatan Kas dan Stok

Tanggal: 2026-11-24
Status: target Linux dikonfirmasi pemilik; dilanjutkan ke rencana kerja

Tanda **[keputusan pemilik]** berarti diputuskan oleh pemilik dalam diskusi.
Tanda **[usulan]** berarti diisi oleh perancang dan boleh dikoreksi.

## 1. Maksud dan alasan

Toko perikanan (pakan ikan dan perlengkapan terkait) butuh satu aplikasi
untuk mencatat uang masuk/keluar dan barang masuk/keluar. Masalah utamanya:
barang yang sama datang berkali-kali dengan spek dan harga yang bisa berbeda,
dan pemilik ingin stok lama habis lebih dulu sebelum stok baru dipakai.

Aplikasi harus bisa dibawa (portabel): satu folder berisi program dan datanya,
tanpa instalasi server dan tanpa internet.

## 2. Tujuan

1. Mencatat barang masuk dan keluar per kiriman, dengan stok lama diarahkan
   keluar lebih dulu. **[keputusan pemilik: stok lama dihabiskan dulu]**
2. Mencatat arus kas, sebagian otomatis dari transaksi barang.
3. Menampilkan grafik arus kas dan arus barang.
4. Mengekspor laporan ke XLSX dan PDF.
5. Login wajib, dengan hak akses per pengguna. Pemilik selalu akses penuh;
   akun staf bisa ditambah kelak. **[keputusan pemilik]**
6. Foto barang.
7. Data di SQLite, tersimpan di sebelah program. **[keputusan pemilik]**
8. Mudah dipakai orang yang bukan orang komputer; antarmuka bahasa Indonesia,
   mata uang Rupiah. **[usulan]**

## 3. Bukan tujuan (versi pertama)

Semua butir ini **[usulan]**:

- Hutang/piutang dan pembayaran bertahap. Semua transaksi dianggap lunas.
- Konversi satuan (mis. karung ke kg). Satu barang punya satu satuan; kalau
  dijual eceran per kg, barang dicatat dalam kg.
- Banyak toko/gudang, sinkronisasi antar komputer, akses dari HP.
- Cetak struk, barcode scanner, pajak/PPN, pembukuan berpasangan (akuntansi).
- Enkripsi file database. **[keputusan pemilik: tidak dienkripsi, karena
  risiko data hilang kalau kata sandi lupa]**

## 4. Yang membuat aplikasi ini gagal walau "jalan"

- Angka stok atau kas bisa salah karena transaksi diubah/dihapus tanpa jejak.
- Staf bisa melihat atau mengubah hal yang tidak diizinkan.
- Data tertinggal di komputer lama saat folder dipindah.
- Mencatat satu penjualan butuh lebih dari beberapa langkah.

## 5. Batasan

- Tauri 2, SQLite. **[keputusan pemilik]**
- Mesin pengembangan Linux; Rust belum terpasang (perlu `rustup`).
- Target versi pertama: **Linux** (mesin ini). **[keputusan pemilik: "sementara
  di mesin Linux dulu"]** Hasil akhirnya binari Linux portabel dengan folder
  `data/` di sebelahnya, dibangun dan diuji di sini.
- Windows menyusul dan di luar lingkup versi pertama. Supaya nanti tidak perlu
  merombak: jangan memakai jalur atau perintah khusus Linux di kode aplikasi,
  dan semua jalur file dibentuk lewat API jalur Rust.
- Satu pengguna aktif pada satu waktu, satu komputer.

## 6. Pendekatan

| | Pendekatan | Untung | Rugi |
|---|---|---|---|
| A | Semua aturan (login, hak akses, stok, kas) di Rust; tampilan hanya memanggil perintah | Hak akses tak bisa diakali dari tampilan; logika di satu tempat dan mudah diuji | Sedikit lebih lama dibangun |
| B | Aturan di tampilan, SQLite diakses langsung dari antarmuka | Paling cepat | Hak akses hanya disembunyikan, tidak dijaga |
| C | A + database terenkripsi (SQLCipher) | Aman kalau flashdisk hilang | Lupa kata sandi = data hilang; build Windows lebih rumit |

**Dipilih: A** **[keputusan pemilik, mengikuti saran]**. B bertentangan dengan
tujuan 5. C ditolak karena risiko kehilangan data.

Pilihan teknis **[usulan]**:

- Tampilan: Vue 3 + TypeScript + Vite + Tailwind + shadcn-vue, grafik ECharts,
  pengelola paket pnpm.
- Rust: `rusqlite` (fitur `bundled`), `argon2`, `rust_xlsxwriter`, `image`.
- PDF dibuat di Rust; pustakanya dipilih pembangun.

## 7. Penyimpanan data (portabel)

```
Catat/
├── catat.exe            (atau binari Linux)
└── data/
    ├── catat.db
    ├── foto/            foto barang & kiriman, sudah diperkecil
    └── cadangan/        hasil backup (zip)
```

- Folder `data/` selalu berada di sebelah program. Kalau folder program tidak
  bisa ditulis, aplikasi menampilkan pesan jelas dan tidak diam-diam menyimpan
  di tempat lain.
- SQLite memakai mode jurnal `DELETE` (bukan WAL) supaya data tetap satu file
  dan aman di flashdisk; `foreign_keys = ON`.
- Skema dikelola dengan migrasi bernomor yang dijalankan saat aplikasi dibuka.
- Uang disimpan sebagai bilangan bulat Rupiah. Jumlah barang disimpan sebagai
  bilangan bulat per seperseribu satuan (mendukung 3 desimal, mis. 0,5 kg).

## 8. Model data

- **pengguna**: nama pengguna, nama tampilan, hash kata sandi, peran
  (`pemilik`/`staf`), aktif.
- **hak_akses**: per pengguna staf, per modul, satu tingkat.
- **kategori_barang**, **barang**: kode, nama, kategori, satuan, harga jual
  bawaan, batas stok minimum, foto, aktif.
- **kiriman** (batch): barang, tanggal masuk, jumlah awal, sisa, harga beli per
  satuan, catatan spek, tanggal kedaluwarsa (opsional), pemasok (teks, opsional),
  foto (opsional).
- **transaksi_keluar** + **baris_keluar**: satu transaksi bisa berisi beberapa
  barang; tiap baris menunjuk satu kiriman, jumlah, harga jual per satuan.
- **penyesuaian_stok**: kiriman, selisih jumlah, alasan (rusak, kedaluwarsa,
  hilang, pakai sendiri, koreksi hitung).
- **kas**: tanggal, arah (masuk/keluar), kategori, jumlah, keterangan, sumber
  (`manual`/`otomatis`) dan rujukan ke transaksi asalnya.
- **akun_kas**: nama, aktif (bawaan: Tunai). Tiap catatan kas menunjuk satu akun.
- **kategori_kas**: bisa diatur; bawaan: Penjualan, Pembelian barang, Modal,
  Listrik/Air, Gaji, Lain-lain.
- **log_aktivitas**: siapa, kapan, tindakan, ringkasan data.

Setiap catatan menyimpan pembuat dan waktu dibuat.

## 9. Perilaku

### Barang masuk
Pilih barang (atau buat baru di tempat), isi jumlah, harga beli, spek,
kedaluwarsa, foto. Tersimpan sebagai kiriman baru dan otomatis membuat catatan
kas keluar "Pembelian barang". Ada pilihan "stok awal / tidak memengaruhi kas"
untuk memasukkan stok lama yang sudah ada sebelum aplikasi dipakai.

### Barang keluar
Pilih barang dan jumlah. Aplikasi mengisi otomatis dari kiriman dengan urutan
**tanggal masuk terlama dulu**; kalau tanggal masuk sama, yang kedaluwarsanya
lebih dekat dulu (kiriman tanpa tanggal kedaluwarsa di urutan belakang).
**[usulan, mengikuti keputusan pemilik "stok lama dulu"]** Tanggal kedaluwarsa
tidak mengubah urutan, hanya memunculkan peringatan. Kiriman yang **sudah
kedaluwarsa** tidak diambil otomatis; ditandai merah dan hanya bisa dipilih
manual dengan konfirmasi. Kalau jumlah melebihi sisa satu kiriman, otomatis
dipecah ke kiriman berikutnya. Pengguna boleh mengganti pilihan kiriman secara
manual. Saat menyimpan dipilih akun kas penerima (bawaan: Tunai). Harga jual terisi dari harga bawaan dan
bisa diubah. Menyimpan penjualan otomatis membuat kas masuk "Penjualan".
Stok tidak boleh menjadi negatif.

Tampilan daftar kiriman per barang:

```
Pakan Apung 781-2                    Total stok: 32 karung
┌────────────┬────────┬────────────┬─────────────┬──────────┐
│ Masuk      │ Sisa   │ Harga beli │ Spek        │ Exp      │
├────────────┼────────┼────────────┼─────────────┼──────────┤
│ 03 Sep  ⚠  │ 2 krg  │ Rp 385.000 │ protein 30% │ Jan 2027 │ ← keluar dulu
│ 15 Okt     │ 10 krg │ Rp 392.000 │ protein 31% │ Mar 2027 │
│ 20 Nov     │ 20 krg │ Rp 398.000 │ protein 31% │ Mei 2027 │
└────────────┴────────┴────────────┴─────────────┴──────────┘
```

### Koreksi **[usulan]**
Transaksi barang dan kas otomatis tidak dihapus atau diubah diam-diam. Yang ada
adalah **batalkan**: transaksi ditandai batal dengan alasan, stok kiriman
dikembalikan, dan catatan kas otomatisnya ikut dibatalkan. Transaksi batal
tetap terlihat (dicoret) dan tidak dihitung dalam laporan. Kas manual boleh
diubah/dihapus oleh yang berhak; perubahan masuk log aktivitas.
Barang masuk hanya bisa dibatalkan kalau kirimannya belum pernah keluar.

Kiriman yang sudah pernah keluar tetap bisa **diperbaiki** oleh yang berhak
(tingkat "ubah & batalkan"), semuanya tercatat di log dengan nilai lama dan baru:
- spek, kedaluwarsa, pemasok, foto: bebas diubah;
- harga beli: boleh diubah; catatan kas otomatis pembeliannya ikut disesuaikan,
  dan laba dihitung ulang dari harga yang baru;
- jumlah: tidak diubah langsung, melainkan lewat penyesuaian stok.

### Penyesuaian stok **[usulan]**
Untuk barang rusak, kedaluwarsa, hilang, pakai sendiri, atau selisih hitung.
Dilakukan per kiriman, bisa mengurangi atau menambah, wajib ada alasan; sisa
tidak boleh di bawah nol. Tidak memengaruhi kas. Butuh hak Barang Masuk tingkat
"ubah & batalkan". Bisa dibatalkan seperti transaksi lain. Muncul di kartu stok,
dan pengurangannya tampil sebagai baris terpisah "Kerugian stok" (dinilai
menurut harga beli) di laporan laba kotor.

### Kas
Daftar kas dengan saldo berjalan, saring per tanggal/kategori/arah/akun. Tambah
kas manual masuk atau keluar.

**Akun kas [usulan]:** uang bisa disimpan di lebih dari satu tempat. Bawaan
hanya ada akun "Tunai"; pemilik bisa menambah akun lain (mis. "Bank/Transfer").
Setiap catatan kas menunjuk satu akun, saldo ditampilkan per akun dan total,
dan ada "pindah antar akun" (mis. setor tunai ke bank) yang tidak dihitung
sebagai pemasukan/pengeluaran. Dengan begini saldo Tunai bisa dicocokkan dengan
uang di laci.

### Beranda (visualisasi)
- Kartu ringkas: saldo kas, kas masuk dan keluar bulan ini, nilai stok (menurut
  harga beli), laba kotor bulan ini.
- Grafik batang kas masuk vs keluar per hari/minggu/bulan; garis saldo kas.
- Grafik batang jumlah barang masuk vs keluar per periode; barang terlaris.
- Daftar perhatian: stok di bawah minimum, kiriman mendekati kedaluwarsa
  (≤ 30 hari), dan **stok lama** (kiriman tertua yang masih bersisa).
- Kartu dan grafik yang memuat harga beli/laba hanya tampil bagi yang berhak.

### Laporan dan ekspor
Semua laporan punya rentang tanggal dan tombol XLSX serta PDF:
1. Arus kas (rincian + ringkasan per kategori)
2. Posisi stok (per barang dan per kiriman, dengan nilai)
3. Kartu stok satu barang (riwayat masuk/keluar/penyesuaian)
4. Barang masuk dan barang keluar
5. Laba kotor penjualan (harga jual − harga beli kiriman yang diambil)

File disimpan lewat dialog "Simpan sebagai". PDF berisi nama toko, judul,
periode, tabel, dan total; ukuran A4.

### Foto
Dipilih dari file. Disalin ke `data/foto/`, diperkecil (sisi terpanjang maks.
1280 px, JPEG) plus gambar kecil untuk daftar. Jalur di database relatif
terhadap `data/`.

### Cadangan
Tombol "Cadangkan" membuat satu zip berisi database dan foto; "Pulihkan"
memuat kembali dari zip (hanya pemilik, dengan konfirmasi).

## 10. Keamanan dan hak akses

- **Pertama kali dibuka**: wizard membuat akun pemilik dan nama toko, lalu
  menampilkan **kode pemulihan** sekali untuk dicatat. Kode ini satu-satunya
  cara mengatur ulang kata sandi pemilik. **[usulan]**
- Kata sandi di-hash dengan Argon2id; minimal 8 karakter.
- 5 kali gagal login berturut-turut: akun ditahan sementara dengan jeda yang
  makin lama. Aplikasi terkunci otomatis setelah 10 menit tanpa aktivitas
  (bisa diatur pemilik).
- Sesi login dipegang di sisi Rust. **Setiap** perintah memeriksa sesi dan hak
  akses sebelum menyentuh data; tampilan hanya menyembunyikan menu demi
  kenyamanan.
- Peran `pemilik`: akses penuh, tidak bisa dikurangi; minimal harus selalu ada
  satu pemilik aktif.
- Peran `staf`: hak diatur pemilik per modul:

  | Modul | Tingkat |
  |---|---|
  | Barang, Barang Masuk, Barang Keluar, Kas, Laporan | tidak ada / lihat / tambah / ubah & batalkan |
  | Lihat harga beli dan laba | ya / tidak |
  | Pengguna, Pengaturan, Cadangan/Pulihkan | hanya pemilik |

  Tiap tingkat mencakup tingkat di bawahnya. Bawaan akun staf baru (kasir):
  Barang lihat, Barang Keluar tambah, Barang Masuk tidak ada, Kas tidak ada,
  Laporan tidak ada, harga beli tidak.
- Aturan harga beli **[usulan]**:
  - Tanpa hak "harga beli dan laba", Rust tidak mengirim harga beli, nilai stok,
    atau laba sama sekali (daftar kiriman tampil tanpa kolom harga beli).
  - Hak Barang Masuk (tingkat apa pun) dan laporan yang memuat harga beli/laba
    mensyaratkan hak "harga beli dan laba"; layar pengaturan memaksa keduanya
    dicentang bersama.
  - Hak Kas "lihat" berarti melihat semua catatan kas termasuk pembelian. Kalau
    pemilik tidak ingin staf tahu nilai pembelian, hak Kas jangan diberikan.
  - Penjualan oleh staf tetap membuat kas masuk otomatis walau ia tidak punya
    hak Kas.
- Pemilik boleh lebih dari satu. Pemilik bisa mengatur ulang kata sandi pengguna
  lain. Kode pemulihan berlaku per instalasi: dipakai dari layar login untuk
  mengatur ulang kata sandi salah satu akun pemilik, lalu langsung diganti
  dengan kode baru yang ditampilkan sekali. Disimpan sebagai hash. **[usulan]**
- Log aktivitas mencatat login, gagal login, dan semua tambah/ubah/batal;
  hanya pemilik yang bisa melihatnya, tidak ada yang bisa menghapusnya.
- Tauri: CSP ketat, izin (capabilities) seminimal mungkin, tanpa akses jaringan,
  akses file tampilan dibatasi ke `data/foto/`.

## 11. Susunan layar

Menu samping: Beranda · Barang · Barang Masuk · Barang Keluar · Kas · Laporan ·
Pengguna · Pengaturan. Tombol cepat "＋ Barang Keluar" dan "＋ Barang Masuk"
selalu terlihat. Formulir pendek, bawaan yang masuk akal (tanggal hari ini),
pesan galat dalam bahasa sehari-hari, mode terang dan gelap.

## 12. Pengujian

- Uji Rust (SQLite di memori) untuk: alokasi kiriman lama dulu termasuk pecah
  antar kiriman, larangan stok negatif, pembatalan mengembalikan stok dan kas,
  kas otomatis, pemeriksaan hak akses tiap perintah, login/penahanan, migrasi.
- Uji ekspor: file XLSX dan PDF terbentuk dan totalnya cocok dengan data.
- Uji tampilan seperlunya untuk formulir barang keluar.
- Uji jalan nyata di build Linux sebelum dinyatakan selesai.

## 13. Diserahkan ke pembangun

Pustaka PDF, struktur modul Rust dan komponen Vue, bentuk persis perintah
Tauri, nama kolom, dan desain visual rinci.
