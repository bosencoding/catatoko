// Pustaka kecil untuk menguji antarmuka di aplikasi sungguhan (build debug).
// Digabung di depan berkas skenario oleh uji-ui/jalankan.sh.
const __invoke = window.__TAURI_INTERNALS__.invoke;
const tidur = (ms) => new Promise((r) => setTimeout(r, ms));
// Nama sederhana = atribut data-uji; selain itu dianggap selektor CSS biasa.
const el = (sel) => (/^[\w\/.-]+$/.test(sel) ? document.querySelector(`[data-uji="${sel}"]`) : document.querySelector(sel));
const semua = (sel) => [...document.querySelectorAll(`[data-uji="${sel}"]`)];

async function tunggu(sel, ms = 8000) {
  const batas = Date.now() + ms;
  while (Date.now() < batas) {
    const e = el(sel);
    if (e) return e;
    await tidur(50);
  }
  throw new Error(`elemen '${sel}' tidak muncul; halaman: ${location.hash}; teks: ${document.body.innerText.slice(0, 300)}`);
}
async function tungguHilang(sel, ms = 8000) {
  const batas = Date.now() + ms;
  while (Date.now() < batas) {
    if (!el(sel)) return;
    await tidur(50);
  }
  throw new Error(`elemen '${sel}' tidak hilang`);
}
async function tungguTeks(teks, ms = 8000) {
  const batas = Date.now() + ms;
  while (Date.now() < batas) {
    if (document.body.innerText.includes(teks)) return;
    await tidur(50);
  }
  throw new Error(`teks '${teks}' tidak muncul; halaman: ${location.hash}; teks: ${document.body.innerText.slice(0, 400)}`);
}
async function isi(sel, nilai) {
  const e = await tunggu(sel);
  e.focus();
  const proto = e instanceof HTMLTextAreaElement ? HTMLTextAreaElement.prototype : e instanceof HTMLSelectElement ? HTMLSelectElement.prototype : HTMLInputElement.prototype;
  Object.getOwnPropertyDescriptor(proto, "value").set.call(e, String(nilai));
  e.dispatchEvent(new Event("input", { bubbles: true }));
  e.dispatchEvent(new Event("change", { bubbles: true }));
  e.dispatchEvent(new Event("blur", { bubbles: true }));
  e.dispatchEvent(new FocusEvent("focusout", { bubbles: true }));
  await tidur(30);
}
async function centang(sel, nilai = true) {
  const e = await tunggu(sel);
  if (e.checked !== nilai) e.click();
  await tidur(30);
}
async function klik(sel) {
  const e = await tunggu(sel);
  if (e.disabled) throw new Error(`tombol '${sel}' nonaktif; teks: ${document.body.innerText.slice(0, 300)}`);
  e.click();
  await tidur(80);
}
const teks = (sel) => el(sel)?.innerText?.trim() ?? null;
const ada = (sel) => !!el(sel);
function pastikan(syarat, pesan) {
  if (!syarat) throw new Error(`pastikan gagal: ${pesan}`);
}
function sama(nyata, harap, apa) {
  if (nyata !== harap) throw new Error(`${apa}: harap '${harap}', nyata '${nyata}'`);
}
async function lapor(nama, isinya) {
  await __invoke("uji_tulis", { nama, isi: typeof isinya === "string" ? isinya : JSON.stringify(isinya, null, 1) });
}
/** Meminta jalankan.sh memotret layar; menunggu sampai fotonya jadi. */
async function potret(nama) {
  await tidur(250);
  await lapor(`potret-${nama}.minta`, "");
  await tidur(1500);
}
async function masukSebagai(nama, sandi) {
  await isi("nama-pengguna", nama);
  await isi("sandi", sandi);
  await klik("masuk");
  await tunggu("keluar");
}

/** Menyiapkan toko langsung lewat perintah (tanpa wizard) lalu masuk lewat layar login. */
async function siapkanToko() {
  await tunggu("nama-toko");
  await __invoke("setup_awal", { nama_toko: "Toko Mina Jaya", nama_pengguna: "budi", nama: "Pak Budi", sandi: "rahasia123" });
  location.hash = "#/login";
  await tunggu("masuk");
  await masukSebagai("budi", "rahasia123");
}
async function ke(jalur, tanda) {
  location.hash = `#${jalur}`;
  await tidur(150);
  if (tanda) await tunggu(tanda);
}
/** Memilih barang pada komponen PilihBarang. */
async function pilihBarang(uji, id) {
  const e = await tunggu(`pilih-barang-${uji}`);
  e.focus();
  e.dispatchEvent(new FocusEvent("focus"));
  const o = await tunggu(`opsi-barang-${id}`);
  o.dispatchEvent(new MouseEvent("mousedown", { bubbles: true, cancelable: true }));
  await tidur(250);
}
const hariIniIso = () => {
  const d = new Date();
  const p = (x) => String(x).padStart(2, "0");
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}`;
};
async function gagal(janji, kode) {
  try {
    await janji;
  } catch (e) {
    if (e && e.kode === kode) return e;
    throw new Error(`harap galat ${kode}, nyata ${JSON.stringify(e)}`);
  }
  throw new Error(`harap galat ${kode}, tetapi berhasil`);
}

/** Data contoh di spec: satu barang, tiga kiriman (Sep stok awal; Okt & Nov dibayar tunai). */
async function benihStok() {
  const id = await __invoke("barang_tambah", { masukan: { kode: "", nama: "Pakan Apung 781-2", kategori_id: null, satuan: "karung", harga_jual: 450000, stok_minimum: 5000, foto_sumber: null, aktif: true } });
  for (const [tgl, jml, harga, spek, exp, awal] of [
    ["2026-09-03", 2000, 385000, "protein 30%", "2027-01-31", true],
    ["2026-10-15", 10000, 392000, "protein 31%", "2027-03-31", false],
    ["2026-11-20", 20000, 398000, "protein 31%", "2027-05-31", false],
  ]) {
    await __invoke("masuk_tambah", { masukan: { barang_id: id, tanggal_masuk: tgl, jumlah: jml, harga_beli: harga, spek, kedaluwarsa: exp, pemasok: "CV Mina", foto_sumber: null, stok_awal: awal, akun_kas_id: awal ? null : 1 } });
  }
  return id;
}
