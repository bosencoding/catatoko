import { ref } from "vue";
import { createRouter, createWebHashHistory, type RouteRecordRaw } from "vue-router";
import { panggil } from "@/lib/api";
import type { Modul } from "@/lib/types";
import { useSesi } from "@/stores/sesi";

declare module "vue-router" {
  interface RouteMeta {
    publik?: boolean;
    /** Butuh hak "lihat" pada salah satu modul ini. */
    modul?: Modul[];
    pemilik?: boolean;
  }
}

/** Pesan bila aplikasi tidak bisa menyiapkan folder datanya saat dibuka. */
export const galatAwal = ref<string | null>(null);

const routes: RouteRecordRaw[] = [
  { path: "/galat", component: () => import("@/pages/GalatAwal.vue"), meta: { publik: true } },
  { path: "/setup", component: () => import("@/pages/Setup.vue"), meta: { publik: true } },
  { path: "/login", component: () => import("@/pages/Login.vue"), meta: { publik: true } },
  {
    path: "/",
    component: () => import("@/components/AppLayout.vue"),
    children: [
      { path: "", component: () => import("@/pages/Beranda.vue") },
      { path: "barang", component: () => import("@/pages/Barang.vue"), meta: { modul: ["barang"] } },
      { path: "masuk", component: () => import("@/pages/BarangMasuk.vue"), meta: { modul: ["barang_masuk"] } },
      { path: "keluar", component: () => import("@/pages/BarangKeluar.vue"), meta: { modul: ["barang_keluar"] } },
      { path: "kas", component: () => import("@/pages/Kas.vue"), meta: { modul: ["kas"] } },
      { path: "laporan", component: () => import("@/pages/Laporan.vue"), meta: { modul: ["laporan"] } },
      { path: "pengguna", component: () => import("@/pages/Pengguna.vue"), meta: { pemilik: true } },
      { path: "pengaturan", component: () => import("@/pages/Pengaturan.vue") },
    ],
  },
  { path: "/:lain(.*)*", redirect: "/" },
];

export const router = createRouter({ history: createWebHashHistory(), routes });

let sudahDiperiksa = false;
let statusDitanya = false;

router.beforeEach(async (ke) => {
  const sesi = useSesi();
  if (!sudahDiperiksa) {
    // Folder data tidak bisa dipakai: hanya layar pesan yang boleh tampil.
    if (!statusDitanya) {
      galatAwal.value = await panggil<string | null>("status_awal");
      statusDitanya = true;
    }
    if (galatAwal.value) return ke.path === "/galat" ? true : "/galat";
    // Sekali saat aplikasi dibuka: belum pernah disiapkan → wizard; kalau sudah, samakan sesi.
    if (await panggil<boolean>("perlu_setup")) {
      return ke.path === "/setup" ? true : "/setup";
    }
    sudahDiperiksa = true;
    await sesi.muat();
  }
  if (ke.path === "/setup" || ke.path === "/galat") return "/login";
  if (ke.meta.publik) return sesi.sesi ? "/" : true;
  if (!sesi.sesi) return "/login";
  if (ke.meta.pemilik && !sesi.pemilik) return "/";
  if (ke.meta.modul && !ke.meta.modul.some((m) => sesi.boleh(m, 1))) return "/";
  return true;
});
