import { beforeEach, describe, expect, it, vi } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { GalatApp } from "@/lib/api";
import { useSesi } from "@/stores/sesi";

const panggil = vi.fn();
vi.mock("@/lib/api", async (asli) => ({ ...(await asli<typeof import("@/lib/api")>()), panggil: (...a: unknown[]) => panggil(...a) }));
vi.mock("vue-sonner", () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

import BarangKeluar from "./BarangKeluar.vue";

const pakan = { id: 1, kode: "B0001", nama: "Pakan Apung 781-2", kategori_id: null, kategori: null, satuan: "karung", harga_jual: 450000, stok_minimum: 5000, foto: null, aktif: true, stok: 32000, nilai_stok: null };
const kiriman = [
  { id: 11, tanggal_masuk: "2026-09-03", sisa: 2000, kedaluwarsa: "2027-01-31", sudah_kedaluwarsa: false, spek: "protein 30%" },
  { id: 12, tanggal_masuk: "2026-10-15", sisa: 10000, kedaluwarsa: "2027-03-31", sudah_kedaluwarsa: false, spek: "protein 31%" },
].map((k) => ({ barang_id: 1, barang_nama: pakan.nama, satuan: "karung", jumlah_awal: k.sisa, harga_beli: null, pemasok: null, foto: null, stok_awal: false, batal: false, alasan_batal: null, hari_ke_kedaluwarsa: 90, pernah_keluar: false, ...k }));

let usul: () => Promise<unknown>;

function siapkan() {
  panggil.mockImplementation((perintah: string) => {
    switch (perintah) {
      case "barang_daftar":
        return Promise.resolve([pakan]);
      case "akun_kas_daftar":
        return Promise.resolve([{ id: 1, nama: "Tunai", aktif: true }]);
      case "keluar_terbaru":
        return Promise.resolve([]);
      case "kiriman_barang":
        return Promise.resolve(kiriman);
      case "usul_alokasi":
        return usul();
      case "keluar_tambah":
        return Promise.resolve(7);
      default:
        return Promise.reject(new Error(`perintah tak terduga: ${perintah}`));
    }
  });
}

async function buka() {
  const w = mount(BarangKeluar, { attachTo: document.body });
  await flushPromises();
  return w;
}

async function pilihPakanTigaKarung(w: Awaited<ReturnType<typeof buka>>) {
  await w.get('[data-uji="pilih-barang-0"]').trigger("focus");
  await w.get('[data-uji="opsi-barang-1"]').trigger("mousedown");
  await flushPromises();
  await w.get('[data-uji="jumlah-0"]').setValue("3");
  await flushPromises();
}

describe("BarangKeluar", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    useSesi().sesi = { pengguna_id: 1, nama_pengguna: "budi", nama: "Budi", pemilik: true, lihat_harga_beli: true, hak: {} };
    panggil.mockReset();
    usul = () => Promise.resolve([{ kiriman_id: 11, jumlah: 2000 }, { kiriman_id: 12, jumlah: 1000 }]);
    siapkan();
    document.body.innerHTML = "";
  });

  it("mengusulkan kiriman terlama dan menampilkan rinciannya", async () => {
    const w = await buka();
    await pilihPakanTigaKarung(w);
    expect(panggil).toHaveBeenCalledWith("usul_alokasi", { barang_id: 1, jumlah: 3000 });
    const rinci = w.get('[data-uji="alokasi-0"]').text();
    expect(rinci).toContain("2 dari 03 Sep");
    expect(rinci).toContain("1 dari 15 Okt");
  });

  it("mengisi harga bawaan dan menghitung total", async () => {
    const w = await buka();
    await pilihPakanTigaKarung(w);
    expect((w.get('[data-uji="harga-0"]').element as HTMLInputElement).value).toBe("450.000");
    expect(w.get('[data-uji="total"]').text()).toContain("Rp 1.350.000");
  });

  it("menampilkan stok kurang pada barisnya dan menahan tombol simpan", async () => {
    usul = () => Promise.reject(new GalatApp("STOK_KURANG", "Stok Pakan Apung 781-2 tidak cukup: diminta 3 karung, tersedia 2 karung."));
    const w = await buka();
    await pilihPakanTigaKarung(w);
    expect(w.get('[data-uji="galat-0"]').text()).toContain("tersedia 2 karung");
    expect(w.get('[data-uji="simpan"]').attributes("disabled")).toBeDefined();
  });

  it("menyimpan dengan alokasi hasil usul", async () => {
    const w = await buka();
    await pilihPakanTigaKarung(w);
    expect(w.get('[data-uji="simpan"]').attributes("disabled")).toBeUndefined();
    await w.get('[data-uji="simpan"]').trigger("click");
    await flushPromises();
    const kirim = panggil.mock.calls.find((c) => c[0] === "keluar_tambah");
    expect(kirim).toBeTruthy();
    const masukan = (kirim![1] as { masukan: { baris: unknown[]; izinkan_kedaluwarsa: boolean; akun_kas_id: number } }).masukan;
    expect(masukan.baris).toEqual([{ barang_id: 1, harga_jual: 450000, alokasi: [{ kiriman_id: 11, jumlah: 2000 }, { kiriman_id: 12, jumlah: 1000 }] }]);
    expect(masukan.izinkan_kedaluwarsa).toBe(false);
    expect(masukan.akun_kas_id).toBe(1);
    // form dikosongkan setelah tersimpan
    await flushPromises();
    expect((w.get('[data-uji="jumlah-0"]').element as HTMLInputElement).value).toBe("");
  });

  it("jawaban usul yang terlambat tidak menimpa isian terbaru", async () => {
    const tunda: ((v: unknown) => void)[] = [];
    usul = () => new Promise((r) => tunda.push(r));
    const w = await buka();
    await pilihPakanTigaKarung(w);
    await w.get('[data-uji="jumlah-0"]').setValue("1");
    await flushPromises();
    tunda[tunda.length - 1]([{ kiriman_id: 11, jumlah: 1000 }]);
    tunda[0]([{ kiriman_id: 11, jumlah: 2000 }, { kiriman_id: 12, jumlah: 1000 }]);
    await flushPromises();
    expect(w.get('[data-uji="alokasi-0"]').text()).toBe("1 dari 03 Sep");
  });
});
