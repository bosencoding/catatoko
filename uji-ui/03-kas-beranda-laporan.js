// Kas manual & pindah akun → beranda (kartu, grafik, daftar) → enam laporan → ekspor XLSX/PDF.
// Catatan: tanggal contoh ada di tahun 2026; angka dicek pada periode "Tahun ini".
async function skenario() {
  await siapkanToko();
  await potret("beranda-kosong");
  pastikan(ada("beranda-kosong"), "beranda kosong menampilkan ajakan mulai");
  const barang = await benihStok();
  const usul = await __invoke("usul_alokasi", { barang_id: barang, jumlah: 3000 });
  await __invoke("keluar_tambah", { masukan: { tanggal: hariIniIso(), akun_kas_id: 1, catatan: null, baris: [{ barang_id: barang, harga_jual: 450000, alokasi: usul }], izinkan_kedaluwarsa: false } });
  await __invoke("akun_kas_tambah", { nama: "Bank" });

  // --- Kas: masuk, keluar, pindah, ubah, batal
  await ke("/kas", "kas-masuk");
  await klik("kas-masuk");
  pastikan(el("kas-simpan").disabled, "simpan nonaktif sebelum diisi");
  await isi("kas-jumlah", "15.000.000");
  const pilihKategori = async (nama) => {
    const s = await tunggu("kas-kategori");
    const o = [...s.options].find((x) => x.textContent.trim() === nama);
    pastikan(o, `kategori ${nama} ada`);
    await isi("kas-kategori", o.value);
  };
  pastikan(![...el("kas-kategori").options].some((o) => o.textContent.trim() === "Penjualan"), "kategori otomatis tidak bisa dipilih manual");
  await pilihKategori("Modal");
  await isi("kas-keterangan", "Modal awal");
  await klik("kas-simpan");
  await tungguTeks("Modal awal");
  await klik("kas-keluar");
  await isi("kas-jumlah", "200.000");
  await pilihKategori("Listrik/Air");
  await isi("kas-keterangan", "Listrik November");
  await klik("kas-simpan");
  await tungguTeks("Listrik November");
  const kas = async () => __invoke("kas_daftar", { saring: { dari: "2026-01-01", sampai: "2026-12-31", akun_kas_id: null, kategori_id: null, arah: null } });
  const listrik = (await kas()).find((k) => k.keterangan === "Listrik November");
  await klik(`kas-ubah-${listrik.id}`);
  sama(el("kas-jumlah").value, "200.000", "form ubah terisi nilai lama");
  await isi("kas-jumlah", "250.000");
  await klik("kas-simpan");
  await tungguTeks("Rp 250.000");
  await klik("kas-pindah");
  await isi("pindah-jumlah", "99.000.000");
  await klik("pindah-simpan");
  pastikan((await tunggu("pindah-galat")).innerText.includes("tidak cukup"), "pindah melebihi saldo ditolak dengan pesan");
  await isi("pindah-jumlah", "1.000.000");
  await klik("pindah-simpan");
  await tungguHilang("pindah-simpan");
  await tidur(300);
  sama(teks("saldo-total"), "Rp 4.220.000", "total saldo");
  sama(teks("saldo-akun-1"), "Rp 3.220.000", "saldo tunai");
  sama(teks("saldo-akun-2"), "Rp 1.000.000", "saldo bank");
  const otomatis = (await kas()).find((k) => k.sumber === "otomatis");
  pastikan(!ada(`kas-ubah-${otomatis.id}`) && !ada(`kas-batal-${otomatis.id}`), "catatan otomatis tanpa tombol ubah/batal");
  // catatan sementara lalu dibatalkan
  await klik("kas-keluar");
  await isi("kas-jumlah", "50.000");
  await pilihKategori("Lain-lain");
  await isi("kas-keterangan", "Salah catat");
  await klik("kas-simpan");
  await tungguTeks("Salah catat");
  const salah = (await kas()).find((k) => k.keterangan === "Salah catat");
  await klik(`kas-batal-${salah.id}`);
  await isi("alasan-batal", "salah catat");
  await klik("konfirmasi-batal");
  await tungguTeks("Dibatalkan: salah catat");
  sama(teks("saldo-total"), "Rp 4.220.000", "saldo kembali setelah batal");
  await isi("periode", "tahun_ini");
  await tidur(400);
  await potret("kas");

  // --- Beranda
  await ke("/", "kartu-saldo");
  await isi("periode", "tahun_ini");
  await tidur(600);
  sama(teks("kartu-saldo"), "Rp 4.220.000", "kartu saldo");
  sama(teks("kartu-masuk"), "Rp 16.350.000", "kartu kas masuk");
  sama(teks("kartu-keluar"), "Rp 12.130.000", "kartu kas keluar");
  sama(teks("kartu-laba"), "Rp 188.000", "kartu laba");
  sama(teks("kartu-nilai-stok"), "Rp 11.488.000", "kartu nilai stok");
  for (const g of ["grafik-kas", "grafik-barang"]) {
    const c = (await tunggu(`[data-uji="${g}"] canvas`));
    pastikan(c.width > 200 && c.height > 100, `${g} tergambar (${c.width}x${c.height})`);
    const piksel = c.getContext("2d").getImageData(0, 0, c.width, c.height).data;
    let terisi = 0;
    for (let i = 3; i < piksel.length; i += 4) if (piksel[i] > 0) terisi++;
    pastikan(terisi > 500, `${g} tidak kosong (${terisi} piksel)`);
  }
  pastikan(el("daftar-stok-lama").innerText.includes("03 Sep 2026") === false, "kiriman Sep sudah habis, tidak di stok lama");
  pastikan(el("daftar-stok-lama").innerText.includes("15 Okt 2026"), "stok lama memuat kiriman Okt");
  pastikan(el("daftar-terlaris").innerText.includes("Pakan Apung 781-2"), "terlaris memuat pakan");
  await potret("beranda");

  // --- Laporan
  await ke("/laporan", "laporan-arus_kas");
  await isi("periode", "tahun_ini");
  await tidur(500);
  sama(teks("laporan-judul"), "Laporan Arus Kas", "judul arus kas");
  pastikan(el("laporan-total").innerText.includes("Rp 16.350.000") && el("laporan-total").innerText.includes("Rp 12.130.000"), `total arus kas: ${teks("laporan-total")}`);
  pastikan(el("laporan-ringkasan").innerText.includes("Rp 4.220.000"), "saldo akhir di ringkasan");
  pastikan(!el("laporan-tabel").innerText.includes("Salah catat") && !el("laporan-tabel").innerText.includes("Pindah akun"), "batal dan pindah akun tidak masuk laporan");
  await klik("laporan-laba_kotor");
  await tungguTeks("Laporan Laba Kotor");
  await tidur(300);
  pastikan(el("laporan-ringkasan").innerText.includes("Rp 188.000"), "laba kotor");
  await potret("laporan-laba");
  await klik("laporan-posisi_stok");
  await tungguTeks("Laporan Posisi Stok");
  await tidur(300);
  pastikan(el("laporan-total").innerText.includes("Rp 11.488.000"), "nilai posisi stok");
  await klik("laporan-kartu_stok");
  await tungguTeks("Pilih barang untuk melihat kartu stoknya.");
  await pilihBarang("laporan", barang);
  await tungguTeks("Kartu Stok: Pakan Apung 781-2");
  await tidur(300);
  pastikan(el("laporan-ringkasan").innerText.replace(/\s+/g, " ").includes("Saldo akhir 29"), `saldo akhir kartu stok: ${teks("laporan-ringkasan")}`);
  await klik("laporan-barang_masuk");
  await tungguTeks("Laporan Barang Masuk");
  await klik("laporan-barang_keluar");
  await tungguTeks("Laporan Barang Keluar");
  await tidur(300);
  pastikan(el("laporan-total").innerText.includes("Rp 1.350.000"), "total barang keluar");

  // --- Ekspor (dialog simpan tidak bisa diklik skrip; perintahnya dipanggil langsung)
  const jenis = [{ jenis: "arus_kas" }, { jenis: "laba_kotor" }, { jenis: "posisi_stok" }, { jenis: "kartu_stok", barang_id: barang }, { jenis: "barang_masuk" }, { jenis: "barang_keluar" }];
  for (const j of jenis) {
    for (const format of ["xlsx", "pdf"]) {
      const jalur = await __invoke("laporan_ekspor", { jenis: j, dari: "2026-01-01", sampai: "2026-12-31", format, tujuan: `${FOLDER_HASIL}/${j.jenis}.${format}` });
      pastikan(jalur.endsWith(`${j.jenis}.${format}`), `ekspor ${j.jenis}.${format}`);
    }
  }
  // ekstensi dipaksa sesuai format
  const paksa = await __invoke("laporan_ekspor", { jenis: { jenis: "arus_kas" }, dari: "2026-01-01", sampai: "2026-12-31", format: "pdf", tujuan: `${FOLDER_HASIL}/tanpa-ekstensi` });
  pastikan(paksa.endsWith("tanpa-ekstensi.pdf"), "ekstensi ditambahkan");
}
