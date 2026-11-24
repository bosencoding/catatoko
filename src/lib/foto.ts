import { convertFileSrc } from "@tauri-apps/api/core";
import { panggil } from "@/lib/api";

let folder: Promise<string> | null = null;

/** Alamat yang bisa ditampilkan webview untuk foto di `data/foto/`. */
export async function urlFoto(relatif: string, kecil = false): Promise<string> {
  folder ??= panggil<string>("folder_data");
  const rel = kecil ? relatif.replace(/\.jpg$/, "_k.jpg") : relatif;
  return convertFileSrc(`${await folder}/${rel}`);
}
