<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted } from "vue";
import { RouterLink, RouterView, useRouter } from "vue-router";
import {
  BanknoteIcon,
  FileTextIcon,
  LayoutDashboardIcon,
  LogOutIcon,
  MoonIcon,
  PackageIcon,
  PackageMinusIcon,
  PackagePlusIcon,
  SettingsIcon,
  SunIcon,
  UsersIcon,
} from "@lucide/vue";
import { Button } from "@/components/ui/button";
import { panggil } from "@/lib/api";
import { gantiTema, tema } from "@/lib/tema";
import { muatNamaToko, namaToko } from "@/lib/toko";
import { useSesi } from "@/stores/sesi";

const sesi = useSesi();
const router = useRouter();

const menu = computed(() =>
  [
    { ke: "/", label: "Beranda", ikon: LayoutDashboardIcon, tampil: true },
    { ke: "/barang", label: "Barang", ikon: PackageIcon, tampil: sesi.boleh("barang", 1) },
    { ke: "/masuk", label: "Barang Masuk", ikon: PackagePlusIcon, tampil: sesi.boleh("barang_masuk", 1) },
    { ke: "/keluar", label: "Barang Keluar", ikon: PackageMinusIcon, tampil: sesi.boleh("barang_keluar", 1) },
    { ke: "/kas", label: "Kas", ikon: BanknoteIcon, tampil: sesi.boleh("kas", 1) },
    { ke: "/laporan", label: "Laporan", ikon: FileTextIcon, tampil: sesi.boleh("laporan", 1) },
    { ke: "/pengguna", label: "Pengguna", ikon: UsersIcon, tampil: sesi.pemilik },
    { ke: "/pengaturan", label: "Pengaturan", ikon: SettingsIcon, tampil: true },
  ].filter((m) => m.tampil),
);

// Aktivitas pengguna memperpanjang sesi di sisi Rust, paling sering sekali per menit.
let sentuhTerakhir = 0;
function aktivitas() {
  const kini = Date.now();
  if (kini - sentuhTerakhir < 60_000 || !sesi.sesi) return;
  sentuhTerakhir = kini;
  panggil("sentuh").catch(() => {});
}
let berkala: ReturnType<typeof setInterval> | undefined;

onMounted(async () => {
  window.addEventListener("pointerdown", aktivitas, true);
  window.addEventListener("keydown", aktivitas, true);
  // Kunci otomatis ditentukan Rust; di sini hanya memeriksa apakah sesi masih ada.
  berkala = setInterval(() => sesi.muat().catch(() => {}), 30_000);
  await muatNamaToko();
});
onBeforeUnmount(() => {
  window.removeEventListener("pointerdown", aktivitas, true);
  window.removeEventListener("keydown", aktivitas, true);
  clearInterval(berkala);
});

async function keluar() {
  await sesi.keluar();
  router.replace("/login");
}
</script>

<template>
  <div class="flex h-full bg-background text-foreground">
    <aside class="flex w-56 shrink-0 flex-col border-r bg-sidebar text-sidebar-foreground">
      <div class="px-4 pb-3 pt-5">
        <div class="truncate text-base font-semibold" :title="namaToko">{{ namaToko }}</div>
        <div class="text-xs text-muted-foreground">Catat · kas &amp; stok</div>
      </div>
      <div class="flex flex-col gap-1.5 px-3 pb-3">
        <Button v-if="sesi.boleh('barang_keluar', 2)" as-child class="justify-start">
          <RouterLink to="/keluar" data-uji="cepat-keluar"><PackageMinusIcon /> Barang Keluar</RouterLink>
        </Button>
        <Button v-if="sesi.boleh('barang_masuk', 2)" as-child variant="outline" class="justify-start">
          <RouterLink to="/masuk" data-uji="cepat-masuk"><PackagePlusIcon /> Barang Masuk</RouterLink>
        </Button>
      </div>
      <nav class="flex min-h-0 flex-1 flex-col gap-0.5 overflow-auto border-t px-3 py-3">
        <RouterLink
          v-for="m in menu"
          :key="m.ke"
          :to="m.ke"
          :data-uji="`menu-${m.ke}`"
          class="flex items-center gap-2.5 rounded-lg px-2.5 py-2 text-sm text-muted-foreground transition-colors hover:bg-sidebar-accent hover:text-sidebar-accent-foreground"
          :exact-active-class="m.ke === '/' ? 'bg-sidebar-accent font-medium text-sidebar-accent-foreground' : ''"
          :active-class="m.ke !== '/' ? 'bg-sidebar-accent font-medium text-sidebar-accent-foreground' : ''"
        >
          <component :is="m.ikon" class="size-4" />
          {{ m.label }}
        </RouterLink>
      </nav>
      <div class="flex items-center gap-2 border-t px-3 py-3">
        <div class="min-w-0 flex-1">
          <div class="truncate text-sm font-medium" data-uji="nama-pengguna">{{ sesi.sesi?.nama }}</div>
          <div class="text-xs text-muted-foreground">{{ sesi.pemilik ? "Pemilik" : "Staf" }}</div>
        </div>
        <Button variant="ghost" size="icon" :title="tema === 'gelap' ? 'Mode terang' : 'Mode gelap'" @click="gantiTema">
          <SunIcon v-if="tema === 'gelap'" /><MoonIcon v-else />
        </Button>
        <Button variant="ghost" size="icon" title="Keluar" data-uji="keluar" @click="keluar"><LogOutIcon /></Button>
      </div>
    </aside>
    <main class="min-w-0 flex-1">
      <RouterView />
    </main>
  </div>
</template>
