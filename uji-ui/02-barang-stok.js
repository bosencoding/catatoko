// Barang berfoto → tiga kiriman contoh → jual stok lama dulu → batal → ganti kiriman
// → perbaiki & sesuaikan → akun kasir tidak melihat harga beli.
async function skenario() {
  await siapkanToko();

  // --- Barang baru lewat formulir
  await ke("/barang", "barang-kosong");
  await klik("barang-baru");
  await isi("barang-nama", "Pakan Apung 781-2");
  await isi("barang-satuan", "karung");
  await isi("barang-harga", "450.000");
  await isi("barang-minimum", "5");
  await klik("barang-kategori-ganti");
  await isi("barang-kategori-baru", "Pakan");
  await klik("barang-simpan");
  await tunggu("barang-1");
  await tungguTeks("B0001");
  await tungguTeks("Pakan");

  // --- Foto: lewat perintah (dialog berkas tidak bisa diklik oleh skrip), lalu harus tampil
  const b = await __invoke("barang_ambil", { id: 1 });
  await __invoke("barang_ubah", { id: 1, masukan: { kode: b.kode, nama: b.nama, kategori_id: b.kategori_id, satuan: b.satuan, harga_jual: b.harga_jual, stok_minimum: b.stok_minimum, foto_sumber: CONTOH_FOTO, aktif: true } });
  await ke("/", "menu-/barang");
  await ke("/barang", "barang-1");
  const gambar = await tunggu('[data-uji="barang-1"] img');
  for (let i = 0; i < 60 && !gambar.complete; i++) await tidur(50);
  pastikan(gambar.naturalWidth === 256, `foto kecil tampil lewat protokol aset (naturalWidth=${gambar.naturalWidth}, src=${gambar.src})`);
  await potret("barang");

  // --- Barang masuk: contoh di spec
  await ke("/masuk", "masuk-simpan");
  const kiriman = [
    ["03/09/2026", "2", "385.000", "protein 30%", "31/01/2027", true],
    ["15/10/2026", "10", "392.000", "protein 31%", "31/03/2027", false],
    ["20/11/2026", "20", "398.000", "protein 31%", "31/05/2027", false],
  ];
  for (const [tgl, jml, harga, spek, exp, stokAwal] of kiriman) {
    await pilihBarang("masuk", 1);
    await isi("masuk-jumlah", jml);
    await isi("masuk-harga", harga);
    await isi("masuk-tanggal", tgl);
    await isi("masuk-exp", exp);
    await isi("masuk-spek", spek);
    await centang("masuk-stok-awal", stokAwal);
    if (tgl === "15/10/2026") sama(teks("masuk-total"), "Rp 3.920.000", "total barang masuk");
    await klik("masuk-simpan");
    await tungguTeks(spek);
    await tidur(300);
  }
  await potret("masuk");
  const sisa = async () => (await __invoke("kiriman_barang", { barang_id: 1, hanya_bersisa: false })).map((k) => k.sisa).join("/");
  sama(await sisa(), "2000/10000/20000", "sisa setelah tiga kiriman");
  const saldo = async () => (await __invoke("kas_saldo")).total;
  sama(await saldo(), -(3920000 + 7960000), "kas keluar hanya untuk yang bukan stok awal");

  // --- Barang keluar: stok lama dulu
  await ke("/keluar", "pilih-barang-0");
  await pilihBarang(0, 1);
  sama(el("harga-0").value, "450.000", "harga jual bawaan");
  await isi("jumlah-0", "3");
  await tunggu("alokasi-0");
  sama(teks("alokasi-0"), "2 dari 03 Sep · 1 dari 15 Okt", "usul alokasi");
  sama(teks("total"), "Rp 1.350.000", "total penjualan");
  await potret("keluar");
  await klik("simpan");
  await tunggu("transaksi-1");
  sama(await sisa(), "0/9000/20000", "sisa setelah jual 3");
  sama(await saldo(), -(3920000 + 7960000) + 1350000, "kas bertambah penjualan");
  await tungguTeks("Rp 188.000"); // laba: 1.350.000 − (2×385.000 + 392.000)

  // --- Batal: stok dan kas kembali
  await klik("batal-1");
  await isi("alasan-batal", "salah harga");
  await klik("konfirmasi-batal");
  await tungguTeks("Dibatalkan: salah harga");
  sama(await sisa(), "2000/10000/20000", "sisa setelah batal");
  sama(await saldo(), -(3920000 + 7960000), "kas setelah batal");

  // --- Stok kurang
  await pilihBarang(0, 1);
  await isi("jumlah-0", "33");
  pastikan((await tunggu("galat-0")).innerText.includes("tersedia 32 karung"), `pesan stok kurang: ${teks("galat-0")}`);
  pastikan(el("simpan").disabled, "simpan nonaktif saat stok kurang");

  // --- Ganti kiriman secara manual, jumlah pecahan
  await isi("jumlah-0", "1");
  await tunggu("alokasi-0");
  await klik("ganti-kiriman-0");
  await isi("ambil-1", "");
  await isi("ambil-3", "1,5");
  await klik("pakai-kiriman");
  await tidur(200);
  sama(teks("alokasi-0"), "1,5 dari 20 Nov", "alokasi manual");
  sama(teks("total"), "Rp 675.000", "total 1,5 karung");
  await klik("simpan");
  await tunggu("transaksi-2");
  sama(await sisa(), "2000/10000/18500", "sisa setelah jual manual");

  // --- Rincian barang: tanda keluar dulu, perbaiki, sesuaikan
  await ke("/barang", "barang-1");
  await klik("barang-1");
  await tunggu("kiriman-1");
  pastikan(el("kiriman-1").contains(el("keluar-dulu")), "kiriman Sep bertanda keluar dulu");
  pastikan(document.body.innerText.includes("Harga beli"), "pemilik melihat kolom harga beli");
  sama(teks("rincian-stok"), "30,5 karung", "total stok di rincian");
  await potret("rincian");
  await klik("perbaiki-2");
  await isi("perbaiki-harga", "390.000");
  await klik("perbaiki-simpan");
  await tungguHilang("perbaiki-simpan");
  const kas = await __invoke("kas_daftar", { saring: { dari: "2026-01-01", sampai: "2026-12-31", akun_kas_id: null, kategori_id: null, arah: null } });
  pastikan(kas.some((k) => k.ref_jenis === "kiriman" && k.ref_id === 2 && k.jumlah === 3900000 && !k.batal), "kas pembelian mengikuti harga baru");
  await klik("sesuaikan-2");
  await isi("sesuai-jumlah", "1");
  await klik("sesuai-simpan");
  await tungguHilang("sesuai-simpan");
  sama(await sisa(), "2000/9000/18500", "sisa setelah 1 karung rusak");
  pastikan(!ada("batal-kiriman-2"), "kiriman yang sudah disesuaikan tidak bisa dibatalkan");
  pastikan(ada("batal-kiriman-1"), "kiriman yang belum keluar masih bisa dibatalkan");

  // --- Kasir: tidak melihat harga beli, tidak bisa barang masuk / kas
  const hak = await __invoke("hak_bawaan_staf");
  await __invoke("pengguna_tambah", { masukan: { nama_pengguna: "ani", nama: "Ani Kasir", peran: "staf", lihat_harga_beli: false, hak, aktif: true }, sandi: "kasir12345" });
  location.hash = "#/";
  await tidur(300);
  await klik("keluar");
  await tunggu("masuk");
  await masukSebagai("ani", "kasir12345");
  sama(teks("nama-pengguna"), "Ani Kasir", "masuk sebagai kasir");
  const menu = semua("menu-/").concat(["/barang", "/masuk", "/keluar", "/kas", "/laporan", "/pengguna", "/pengaturan"].flatMap((m) => semua(`menu-${m}`))).map((e) => e.dataset.uji).join(",");
  sama(menu, "menu-/,menu-/barang,menu-/keluar,menu-/pengaturan", "menu kasir");
  pastikan(ada("cepat-keluar") && !ada("cepat-masuk"), "kasir hanya punya tombol cepat barang keluar");
  await ke("/masuk");
  await tidur(300);
  sama(location.hash, "#/", "kasir dialihkan dari Barang Masuk");
  await gagal(__invoke("kas_saldo"), "TIDAK_BERHAK");
  await gagal(__invoke("masuk_tambah", { masukan: { barang_id: 1, tanggal_masuk: hariIniIso(), jumlah: 1000, harga_beli: 1, spek: null, kedaluwarsa: null, pemasok: null, foto_sumber: null, stok_awal: true, akun_kas_id: null } }), "TIDAK_BERHAK");
  const kirimanKasir = await __invoke("kiriman_barang", { barang_id: 1, hanya_bersisa: true });
  pastikan(kirimanKasir.every((k) => k.harga_beli === null), "Rust tidak mengirim harga beli ke kasir");
  await ke("/barang", "barang-1");
  await klik("barang-1");
  await tunggu("kiriman-1");
  pastikan(!document.body.innerText.includes("Harga beli") && !document.body.innerText.includes("Nilai stok"), "kasir tidak melihat harga beli/nilai stok");
  pastikan(!ada("perbaiki-2") && !ada("rincian-ubah"), "kasir tidak punya tombol kelola");
  await potret("rincian-kasir");
  // kasir tetap bisa menjual
  await ke("/keluar", "pilih-barang-0");
  await pilihBarang(0, 1);
  await isi("jumlah-0", "2");
  await tunggu("alokasi-0");
  sama(teks("alokasi-0"), "2 dari 03 Sep", "kasir: stok lama dulu");
  await klik("simpan");
  await tunggu("transaksi-3");
  pastikan(!document.body.innerText.includes("Laba"), "kasir tidak melihat kolom laba");
  pastikan(!ada("batal-3"), "kasir tidak bisa membatalkan");
}
