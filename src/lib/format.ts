// Format dan pembacaan angka gaya Indonesia: titik pemisah ribuan, koma desimal.

const BULAN = ["Jan", "Feb", "Mar", "Apr", "Mei", "Jun", "Jul", "Agu", "Sep", "Okt", "Nov", "Des"];

function ribuan(digit: string): string {
  return digit.replace(/\B(?=(\d{3})+(?!\d))/g, ".");
}

/** 385000 → "Rp 385.000"; nilai tersembunyi (null) → "—". */
export function rupiah(n: number | null | undefined): string {
  if (n === null || n === undefined) return "—";
  return `${n < 0 ? "-" : ""}Rp ${ribuan(Math.abs(Math.round(n)).toString())}`;
}

/** Jumlah per seperseribu satuan: 2000 → "2", 500 → "0,5", (10000, "karung") → "10 karung". */
export function jumlah(milli: number, satuan?: string): string {
  const mutlak = Math.abs(Math.round(milli));
  const utuh = Math.floor(mutlak / 1000);
  const pecahan = (mutlak % 1000).toString().padStart(3, "0").replace(/0+$/, "");
  const teks = `${milli < 0 ? "-" : ""}${ribuan(utuh.toString())}${pecahan ? `,${pecahan}` : ""}`;
  return satuan ? `${teks} ${satuan}` : teks;
}

/** "2026-09-03" → "03 Sep 2026". Teks lain dikembalikan apa adanya. */
export function tanggal(iso: string | null | undefined): string {
  if (!iso) return "";
  const m = /^(\d{4})-(\d{2})-(\d{2})/.exec(iso);
  if (!m) return iso;
  return `${m[3]} ${BULAN[Number(m[2]) - 1] ?? m[2]} ${m[1]}`;
}

/** "2026-11-24T10:05:00" → "24 Nov 2026 10:05". */
export function waktu(iso: string): string {
  const jam = /T(\d{2}:\d{2})/.exec(iso);
  return `${tanggal(iso)}${jam ? ` ${jam[1]}` : ""}`;
}

/** Tanggal lokal hari ini sebagai YYYY-MM-DD. */
export function hariIni(d = new Date()): string {
  const p = (x: number) => x.toString().padStart(2, "0");
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}`;
}

/** Membaca Rupiah bulat. Titik hanya diterima sebagai pemisah ribuan. Tidak sah → null. */
export function bacaRupiah(teks: string): number | null {
  const t = teks.replace(/^\s*Rp\.?\s*/i, "").trim();
  if (!/^(\d{1,3}(\.\d{3})+|\d+)$/.test(t)) return null;
  const n = Number(t.replace(/\./g, ""));
  return Number.isSafeInteger(n) ? n : null;
}

/**
 * Membaca jumlah barang menjadi seperseribu satuan. Harus lebih dari nol, maksimal 3 desimal.
 * Ada koma: titik = ribuan, koma = desimal. Tanpa koma dan tepat satu titik diikuti
 * 1–2 digit: titik = desimal. Selain itu titik = ribuan.
 */
export function bacaJumlah(teks: string): number | null {
  const t = teks.trim();
  let utuh: string;
  let pecahan = "";
  if (t.includes(",")) {
    const bagian = t.split(",");
    if (bagian.length !== 2) return null;
    [utuh, pecahan] = bagian;
    if (!/^(\d{1,3}(\.\d{3})+|\d+)$/.test(utuh) || !/^\d{1,3}$/.test(pecahan)) return null;
    utuh = utuh.replace(/\./g, "");
  } else if (/^\d+\.\d{1,2}$/.test(t)) {
    [utuh, pecahan] = t.split(".");
  } else if (/^(\d{1,3}(\.\d{3})+|\d+)$/.test(t)) {
    utuh = t.replace(/\./g, "");
  } else {
    return null;
  }
  const n = Number(utuh) * 1000 + Number(pecahan.padEnd(3, "0"));
  return Number.isSafeInteger(n) && n > 0 ? n : null;
}

/** Teks isian untuk kolom jumlah (tanpa pemisah ribuan agar mudah diedit). */
export function isiJumlah(milli: number | null): string {
  return milli === null ? "" : jumlah(milli).replace(/\./g, "");
}
