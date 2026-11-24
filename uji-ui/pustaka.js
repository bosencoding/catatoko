// Pustaka kecil untuk menguji antarmuka di aplikasi sungguhan (build debug).
// Digabung di depan berkas skenario oleh uji-ui/jalankan.sh.
const __invoke = window.__TAURI_INTERNALS__.invoke;
const tidur = (ms) => new Promise((r) => setTimeout(r, ms));
const el = (sel) => document.querySelector(`[data-uji="${sel}"]`) || (/^[\w-]+$/.test(sel) ? null : document.querySelector(sel));
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
