import { beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));

async function routerBaru(jawab: Record<string, unknown>) {
  vi.resetModules();
  setActivePinia(createPinia());
  invoke.mockReset();
  invoke.mockImplementation((perintah: string) => (perintah in jawab ? Promise.resolve(jawab[perintah]) : Promise.reject({ kode: "INTERNAL", pesan: `tak terduga: ${perintah}` })));
  const { router } = await import("./router");
  return router;
}

describe("penjaga rute", () => {
  beforeEach(() => {
    location.hash = "";
  });

  it("folder data bermasalah → layar pesan, tanpa memanggil perintah lain", async () => {
    const router = await routerBaru({ status_awal: "Folder data tidak bisa ditulis: /x/data" });
    await router.push("/");
    expect(router.currentRoute.value.path).toBe("/galat");
    expect(invoke.mock.calls.map((c) => c[0])).toEqual(["status_awal"]);
    const { galatAwal } = await import("./router");
    expect(galatAwal.value).toContain("/x/data");
    // tidak bisa meninggalkan layar pesan
    await router.push("/login");
    expect(router.currentRoute.value.path).toBe("/galat");
  });

  it("aplikasi baru → wizard penyiapan", async () => {
    const router = await routerBaru({ status_awal: null, perlu_setup: true });
    await router.push("/");
    expect(router.currentRoute.value.path).toBe("/setup");
  });

  it("sudah disiapkan tetapi belum masuk → layar masuk", async () => {
    const router = await routerBaru({ status_awal: null, perlu_setup: false, sesi_sekarang: null });
    await router.push("/barang");
    expect(router.currentRoute.value.path).toBe("/login");
    await router.push("/galat");
    expect(router.currentRoute.value.path).toBe("/login");
  });

  it("staf dialihkan dari halaman yang bukan haknya", async () => {
    const staf = { pengguna_id: 2, nama_pengguna: "ani", nama: "Ani", pemilik: false, lihat_harga_beli: false, hak: { barang: 1, barang_keluar: 2 } };
    const router = await routerBaru({ status_awal: null, perlu_setup: false, sesi_sekarang: staf });
    await router.push("/keluar");
    expect(router.currentRoute.value.path).toBe("/keluar");
    for (const terlarang of ["/masuk", "/kas", "/laporan", "/pengguna"]) {
      await router.push(terlarang);
      expect(router.currentRoute.value.path, terlarang).toBe("/");
    }
  });
});
