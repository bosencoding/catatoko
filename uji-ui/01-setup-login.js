// Penyiapan awal → kode pemulihan → masuk → tata letak → keluar → sandi salah.
async function skenario() {
  await tunggu("nama-toko");
  sama(location.hash, "#/setup", "aplikasi baru membuka wizard");
  await isi("nama-toko", "Toko Mina Jaya");
  await isi("nama", "Pak Budi");
  await isi("nama-pengguna", "budi");
  await isi("sandi", "rahasia123");
  await isi("ulangi", "rahasia124");
  pastikan(el("simpan").disabled, "tombol nonaktif bila sandi tidak sama");
  await isi("ulangi", "rahasia123");
  await klik("simpan");
  const kode = (await tunggu("kode-pemulihan")).innerText.trim();
  pastikan(/^[A-HJ-NP-Z2-9]{5}(-[A-HJ-NP-Z2-9]{5}){3}$/.test(kode), `bentuk kode pemulihan: ${kode}`);
  await lapor("kode.txt", kode);
  await potret("kode");
  pastikan(el("lanjut").disabled, "harus mencentang dulu");
  await centang("sudah-catat");
  await klik("lanjut");
  await tunggu("masuk");
  sama(location.hash, "#/login", "setelah setup ke layar masuk");
  await tungguTeks("Toko Mina Jaya");

  await isi("nama-pengguna", "budi");
  await isi("sandi", "salahsalah");
  await klik("masuk");
  sama((await tunggu("galat")).innerText.trim(), "Nama pengguna atau kata sandi salah.", "pesan sandi salah");
  await potret("sandi-salah");

  await masukSebagai("budi", "rahasia123");
  sama(location.hash, "#/", "setelah masuk ke beranda");
  sama(teks("nama-pengguna"), "Pak Budi", "nama di pojok");
  for (const m of ["/", "/barang", "/masuk", "/keluar", "/kas", "/laporan", "/pengguna", "/pengaturan"]) {
    pastikan(ada(`menu-${m}`), `menu ${m} tampil untuk pemilik`);
  }
  pastikan(ada("cepat-keluar") && ada("cepat-masuk"), "tombol cepat tampil");
  await potret("beranda");
  await klik("keluar");
  await tunggu("masuk");

  // Lupa kata sandi dengan kode pemulihan
  await klik("lupa");
  await isi("kode", "AAAAA-AAAAA-AAAAA-AAAAA");
  await isi("nama-pengguna-pulih", "budi");
  await isi("sandi-baru", "sandibaru1");
  await klik("pulihkan");
  sama((await tunggu("galat")).innerText.trim(), "Kode pemulihan salah.", "kode salah ditolak");
  await isi("kode", kode.toLowerCase());
  await klik("pulihkan");
  const kodeBaru = (await tunggu("kode-pemulihan")).innerText.trim();
  pastikan(kodeBaru !== kode, "kode pemulihan diganti");
  await centang("sudah-catat");
  await klik("lanjut");
  await masukSebagai("budi", "sandibaru1");
}
