import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { aturSaatTerkunci, panggil } from "@/lib/api";
import type { Modul, Sesi, Tingkat } from "@/lib/types";

export const useSesi = defineStore("sesi", () => {
  const sesi = ref<Sesi | null>(null);
  /** Pesan untuk layar login setelah sesi dikunci dari sisi Rust. */
  const pesanKunci = ref("");

  aturSaatTerkunci((pesan) => {
    if (sesi.value) pesanKunci.value = pesan;
    sesi.value = null;
  });

  const pemilik = computed(() => sesi.value?.pemilik ?? false);
  const bolehHargaBeli = computed(() => !!sesi.value && (sesi.value.pemilik || sesi.value.lihat_harga_beli));

  function boleh(modul: Modul, tingkat: Tingkat): boolean {
    if (!sesi.value) return false;
    return sesi.value.pemilik || (sesi.value.hak[modul] ?? 0) >= tingkat;
  }

  async function masuk(nama_pengguna: string, sandi: string) {
    sesi.value = await panggil<Sesi>("login", { nama_pengguna, sandi });
    pesanKunci.value = "";
  }

  async function keluar() {
    try {
      await panggil("logout");
    } finally {
      sesi.value = null;
    }
  }

  /** Menyamakan dengan sesi di sisi Rust (mis. saat aplikasi dibuka atau pemeriksaan berkala). */
  async function muat() {
    const s = await panggil<Sesi | null>("sesi_sekarang");
    if (!s && sesi.value) pesanKunci.value = "Aplikasi terkunci karena lama tidak dipakai. Silakan masuk lagi.";
    sesi.value = s;
    return s;
  }

  return { sesi, pesanKunci, pemilik, bolehHargaBeli, boleh, masuk, keluar, muat };
});
