import { ref } from "vue";
import { panggil } from "@/lib/api";
import type { Pengaturan } from "@/lib/types";

/** Nama toko untuk tata letak; dimuat ulang setelah pengaturan diubah. */
export const namaToko = ref("Catat");

export async function muatNamaToko() {
  try {
    namaToko.value = (await panggil<Pengaturan>("pengaturan_ambil")).nama_toko || "Catat";
  } catch {
    // nama toko hanya hiasan
  }
}
