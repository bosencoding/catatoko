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

/** "2026-09-03" → "03 Sep". */
export function tanggalPendek(iso: string): string {
  return tanggal(iso).slice(0, 6);
}

/** Nilai Rupiah dari jumlah (seperseribu satuan) × harga per satuan; pembulatan sama dengan sisi Rust. */
export function nilai(milli: number, harga: number): number {
  return Math.floor((milli * harga + 500) / 1000);
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
 * - Ada koma: koma = desimal, titik = pemisah ribuan ("1.250,75").
 * - Tanpa koma dengan SATU titik: titik = desimal ("0.5", "0.500", "1.250" = 1,25), seperti
 *   angka pada timbangan. Tidak pernah dibaca sebagai ribuan, supaya 0.500 kg tidak menjadi 500 kg.
 * - Tanpa koma dengan dua titik atau lebih: titik = pemisah ribuan ("1.250.000").
 */
export function bacaJumlah(teks: string): number | null {
  const t = teks.trim();
  const RIBUAN = /^(\d{1,3}(\.\d{3})+|\d+)$/;
  let utuh: string;
  let pecahan = "";
  if (t.includes(",")) {
    const bagian = t.split(",");
    if (bagian.length !== 2) return null;
    [utuh, pecahan] = bagian;
    if (!RIBUAN.test(utuh) || !/^\d{1,3}$/.test(pecahan)) return null;
    utuh = utuh.replace(/\./g, "");
  } else if (/^\d+\.\d{1,3}$/.test(t)) {
    [utuh, pecahan] = t.split(".");
  } else if (RIBUAN.test(t)) {
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

/** Membaca tanggal gaya Indonesia "hari/bulan/tahun" menjadi YYYY-MM-DD. Tidak sah → null. */
export function bacaTanggal(teks: string): string | null {
  const m = /^(\d{1,2})[/.-](\d{1,2})[/.-](\d{2}|\d{4})$/.exec(teks.trim());
  if (!m) return null;
  const [h, b] = [Number(m[1]), Number(m[2])];
  const t = m[3].length === 2 ? 2000 + Number(m[3]) : Number(m[3]);
  const d = new Date(Date.UTC(t, b - 1, h));
  if (d.getUTCFullYear() !== t || d.getUTCMonth() !== b - 1 || d.getUTCDate() !== h) return null;
  const p = (x: number, n = 2) => x.toString().padStart(n, "0");
  return `${p(t, 4)}-${p(b)}-${p(h)}`;
}

/** "2026-09-03" → "03/09/2026" untuk kolom isian tanggal. */
export function isiTanggal(iso: string | null | undefined): string {
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(iso ?? "");
  return m ? `${m[3]}/${m[2]}/${m[1]}` : "";
}
