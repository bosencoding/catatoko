import { beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));

import { useSesi } from "./sesi";
import { GalatApp, panggil } from "@/lib/api";

const pemilik = { pengguna_id: 1, nama_pengguna: "budi", nama: "Pak Budi", pemilik: true, lihat_harga_beli: true, hak: {} };
const staf = { pengguna_id: 2, nama_pengguna: "ani", nama: "Ani", pemilik: false, lihat_harga_beli: false, hak: { kas: 1, barang_keluar: 2 } };

describe("sesi", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    invoke.mockReset();
  });

  it("pemilik boleh semuanya tanpa daftar hak", async () => {
    invoke.mockResolvedValueOnce(pemilik);
    const s = useSesi();
    await s.masuk("budi", "rahasia123");
    expect(invoke).toHaveBeenCalledWith("login", { nama_pengguna: "budi", sandi: "rahasia123" });
    expect(s.boleh("kas", 3)).toBe(true);
    expect(s.bolehHargaBeli).toBe(true);
    expect(s.pemilik).toBe(true);
  });

  it("staf mengikuti tingkat haknya", async () => {
    invoke.mockResolvedValueOnce(staf);
    const s = useSesi();
    await s.masuk("ani", "rahasia123");
    expect(s.boleh("kas", 1)).toBe(true);
    expect(s.boleh("kas", 2)).toBe(false);
    expect(s.boleh("barang_keluar", 2)).toBe(true);
    expect(s.boleh("laporan", 1)).toBe(false);
    expect(s.bolehHargaBeli).toBe(false);
    expect(s.pemilik).toBe(false);
  });

  it("tanpa sesi tidak boleh apa-apa", () => {
    const s = useSesi();
    expect(s.boleh("barang", 1)).toBe(false);
    expect(s.sesi).toBeNull();
  });

  it("galat dari Rust menjadi GalatApp dengan kode", async () => {
    invoke.mockRejectedValueOnce({ kode: "SANDI_SALAH", pesan: "Nama pengguna atau kata sandi salah." });
    const s = useSesi();
    await expect(s.masuk("budi", "x")).rejects.toMatchObject({ kode: "SANDI_SALAH", message: "Nama pengguna atau kata sandi salah." });
    expect(s.sesi).toBeNull();
  });

  it("sesi dikosongkan saat Rust bilang terkunci", async () => {
    invoke.mockResolvedValueOnce(pemilik);
    const s = useSesi();
    await s.masuk("budi", "rahasia123");
    invoke.mockRejectedValueOnce({ kode: "TERKUNCI", pesan: "Aplikasi terkunci." });
    await expect(panggil("barang_daftar")).rejects.toBeInstanceOf(GalatApp);
    expect(s.sesi).toBeNull();
    expect(s.pesanKunci).toBe("Aplikasi terkunci.");
  });

  it("galat yang bukan dari aplikasi tetap terbaca", async () => {
    invoke.mockRejectedValueOnce("command x not found");
    await expect(panggil("x")).rejects.toMatchObject({ kode: "INTERNAL" });
  });
});
