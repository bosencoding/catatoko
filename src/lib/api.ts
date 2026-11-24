import { invoke } from "@tauri-apps/api/core";

/** Galat dari Rust: `kode` untuk logika, `message` untuk ditampilkan. */
export class GalatApp extends Error {
  kode: string;
  constructor(kode: string, pesan: string) {
    super(pesan);
    this.kode = kode;
    this.name = "GalatApp";
  }
}

let saatTerkunci: (pesan: string) => void = () => {};
/** Didaftarkan oleh store sesi: dipanggil ketika Rust menolak karena belum login/terkunci. */
export function aturSaatTerkunci(fn: (pesan: string) => void) {
  saatTerkunci = fn;
}

export async function panggil<T>(perintah: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(perintah, args);
  } catch (e) {
    const g =
      e && typeof e === "object" && "kode" in e && "pesan" in e
        ? new GalatApp(String((e as { kode: unknown }).kode), String((e as { pesan: unknown }).pesan))
        : new GalatApp("INTERNAL", `Terjadi kesalahan pada aplikasi: ${String(e)}`);
    if (g.kode === "BELUM_LOGIN" || g.kode === "TERKUNCI") saatTerkunci(g.message);
    throw g;
  }
}

/** Pesan yang layak ditampilkan dari galat apa pun. */
export function pesanGalat(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}
