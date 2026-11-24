// Akun staf & hak akses → ganti sandi → nonaktifkan → pengaturan toko → cadangkan/pulihkan → kunci otomatis.
async function keluarLaluMasuk(nama, sandi) {
  location.hash = "#/";
  await tidur(300);
  await klik("keluar");
  await tunggu("masuk");
  await masukSebagai(nama, sandi);
}
const daftarMenu = () => [...document.querySelectorAll('[data-uji^="menu-"]')].map((e) => e.dataset.uji.replace("menu-", "")).join(",");

async function skenario() {
  await siapkanToko();
  await benihStok();

  // --- Akun kasir lewat formulir
  await ke("/pengguna", "pengguna-baru");
  await klik("pengguna-baru");
  await isi("p-nama", "Ani Kasir");
  await isi("p-nama-pengguna", "ani");
  await isi("p-sandi", "kasir12345");
  sama(el("p-hak-barang_keluar").value, "2", "bawaan: barang keluar = tambah");
  sama(el("p-hak-kas").value, "0", "bawaan: kas = tidak ada");
  await isi("p-hak-barang_masuk", "2");
  pastikan(el("p-harga-beli").checked && el("p-harga-beli").disabled, "barang masuk memaksa hak harga beli");
  await potret("form-pengguna");
  await klik("p-bawaan");
  pastikan(!el("p-harga-beli").checked && el("p-hak-barang_masuk").value === "0", "tombol bawaan kasir mengembalikan hak");
  await klik("p-simpan");
  await tunggu("pengguna-ani");
  pastikan(el("pengguna-ani").innerText.includes("Barang Keluar: tambah"), `ringkasan hak: ${teks("pengguna-ani")}`);
  // nama pengguna ganda ditolak dengan pesan
  await klik("pengguna-baru");
  await isi("p-nama", "Ani Lain");
  await isi("p-nama-pengguna", "ANI");
  await isi("p-sandi", "kasir12345");
  await klik("p-simpan");
  pastikan((await tunggu("p-galat")).innerText.includes("sudah dipakai"), "nama pengguna ganda ditolak");
  document.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
  await tidur(300);

  // --- Kasir: menu terbatas, pengaturan hanya kata sandi sendiri
  await keluarLaluMasuk("ani", "kasir12345");
  sama(daftarMenu(), "/,/barang,/keluar,/pengaturan", "menu kasir");
  await ke("/pengaturan", "sandi-lama");
  pastikan(!ada("toko-nama") && !ada("cadangkan") && !ada("log"), "kasir tidak melihat pengaturan pemilik");
  await ke("/pengguna");
  await tidur(300);
  sama(location.hash, "#/", "kasir dialihkan dari halaman Pengguna");
  await gagal(__invoke("pengguna_daftar"), "TIDAK_BERHAK");
  await gagal(__invoke("cadangan_buat", { tujuan: null }), "TIDAK_BERHAK");
  await gagal(__invoke("log_daftar", { dari: "2026-01-01", sampai: "2026-12-31" }), "TIDAK_BERHAK");
  await ke("/pengaturan", "sandi-lama");
  await isi("sandi-lama", "salahsalah");
  await isi("sandi-baru", "kasirbaru1");
  await isi("sandi-ulangi", "kasirbaru1");
  await klik("sandi-simpan");
  sama((await tunggu("sandi-galat")).innerText.trim(), "Kata sandi lama salah.", "sandi lama salah ditolak");
  await isi("sandi-lama", "kasir12345");
  await klik("sandi-simpan");
  await tungguTeks("Kata sandi Anda sudah diganti.");

  // --- Pemilik menambah hak Kas = lihat
  await keluarLaluMasuk("budi", "rahasia123");
  await ke("/pengguna", "pengguna-ani");
  await klik("pengguna-ubah-ani");
  await isi("p-hak-kas", "1");
  await klik("p-simpan");
  await tungguTeks("Kas: lihat");
  await keluarLaluMasuk("ani", "kasirbaru1");
  sama(daftarMenu(), "/,/barang,/keluar,/kas,/pengaturan", "menu kasir setelah diberi hak kas");
  await ke("/kas", "saldo-total");
  pastikan(!ada("kas-masuk") && !ada("kas-keluar"), "hak lihat: tanpa tombol tambah");
  await gagal(__invoke("kas_tambah", { masukan: { tanggal: hariIniIso(), arah: "masuk", akun_kas_id: 1, kategori_id: 4, jumlah: 1000, keterangan: null } }), "TIDAK_BERHAK");

  // --- Nonaktifkan akun
  await keluarLaluMasuk("budi", "rahasia123");
  await ke("/pengguna", "pengguna-ani");
  await klik("pengguna-ubah-ani");
  await centang("p-aktif", false);
  await klik("p-simpan");
  await tungguTeks("nonaktif");
  location.hash = "#/";
  await tidur(300);
  await klik("keluar");
  await tunggu("masuk");
  await isi("nama-pengguna", "ani");
  await isi("sandi", "kasirbaru1");
  await klik("masuk");
  sama((await tunggu("galat")).innerText.trim(), "Nama pengguna atau kata sandi salah.", "akun nonaktif tidak bisa masuk");
  await masukSebagai("budi", "rahasia123");

  // --- Pengaturan toko, akun kas, kategori, log
  await ke("/pengaturan", "toko-nama");
  await tidur(300);
  await isi("toko-nama", "Toko Mina Baru");
  await isi("toko-menit", "1");
  await klik("toko-simpan");
  await tungguTeks("Pengaturan toko tersimpan.");
  await isi("akun-baru", "Bank");
  await klik("akun-tambah");
  await tungguTeks("Akun kas ditambahkan.");
  await isi("kategori-baru", "Sewa");
  await klik("kategori-tambah");
  await tungguTeks("Kategori kas ditambahkan.");
  await tungguTeks("Sewa");
  pastikan(el("log").innerText.includes("pengguna.tambah") && el("log").innerText.includes("auth.gagal"), "log aktivitas memuat kegiatan");
  await potret("pengaturan");

  // --- Cadangkan → ubah data → pulihkan
  await klik("cadangkan");
  await tungguTeks("Cadangan tersimpan di");
  const zip = /(\/\S+\.zip)/.exec(document.body.innerText)[1];
  await __invoke("barang_tambah", { masukan: { kode: "", nama: "Barang Setelah Cadangan", kategori_id: null, satuan: "buah", harga_jual: 1000, stok_minimum: 0, foto_sumber: null, aktif: true } });
  sama((await __invoke("barang_daftar", { cari: null, termasuk_nonaktif: true })).length, 2, "dua barang sebelum pulih");
  await gagal(__invoke("cadangan_pulihkan", { sumber: CONTOH_FOTO }), "VALIDASI");
  sama((await __invoke("barang_daftar", { cari: null, termasuk_nonaktif: true })).length, 2, "file bukan cadangan tidak mengubah data");
  await __invoke("cadangan_pulihkan", { sumber: zip });
  sama(await __invoke("sesi_sekarang"), null, "pulih menghapus sesi");
  location.hash = "#/barang";
  await tunggu("masuk"); // permintaan pertama ditolak Rust → kembali ke layar masuk
  await tungguTeks("Toko Mina Baru");
  await masukSebagai("budi", "rahasia123");
  const setelah = await __invoke("barang_daftar", { cari: null, termasuk_nonaktif: true });
  sama(setelah.map((b) => b.nama).join(","), "Pakan Apung 781-2", "data kembali seperti saat dicadangkan");
  sama((await __invoke("kiriman_barang", { barang_id: 1, hanya_bersisa: true })).length, 3, "kiriman ikut pulih");

  // --- Kunci otomatis (1 menit): tanpa aktivitas, aplikasi kembali ke layar masuk sendiri
  await ke("/barang", "barang-1");
  await tunggu("pesan-kunci", 110000);
  pastikan(teks("pesan-kunci").includes("terkunci"), `pesan kunci: ${teks("pesan-kunci")}`);
  await potret("terkunci");
  await gagal(__invoke("barang_daftar", { cari: null, termasuk_nonaktif: false }), "BELUM_LOGIN");
}
