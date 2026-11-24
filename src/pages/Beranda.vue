<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { RouterLink } from "vue-router";
import { toast } from "vue-sonner";
import GrafikBarang from "@/components/GrafikBarang.vue";
import GrafikKas from "@/components/GrafikKas.vue";
import Halaman from "@/components/Halaman.vue";
import PilihPeriode, { bulanIni } from "@/components/PilihPeriode.vue";
import { panggil, pesanGalat } from "@/lib/api";
import { jumlah, rupiah, tanggal } from "@/lib/format";
import type { Butir, Ringkasan } from "@/lib/types";
import { useSesi } from "@/stores/sesi";

const sesi = useSesi();
const periode = ref(bulanIni());
const r = ref<Ringkasan | null>(null);

function butir(dari: string, sampai: string): Butir {
  const hari = Math.round((Date.parse(sampai) - Date.parse(dari)) / 86_400_000) + 1;
  return hari <= 31 ? "hari" : hari <= 120 ? "minggu" : "bulan";
}

async function muat() {
  try {
    r.value = await panggil<Ringkasan>("ringkasan", { ...periode.value, butir: butir(periode.value.dari, periode.value.sampai) });
  } catch (e) {
    toast.error(pesanGalat(e));
  }
}
watch(periode, muat);
onMounted(muat);

const adaBarang = computed(() => sesi.boleh("barang", 1) || sesi.boleh("barang_keluar", 1) || sesi.boleh("barang_masuk", 1));
const adaGerak = computed(() => !!r.value && (r.value.deret_barang.some((t) => t.masuk || t.keluar) || !!r.value.deret_kas?.some((t) => t.masuk || t.keluar)));
const terlarisMaks = computed(() => Math.max(1, ...(r.value?.terlaris.map((t) => t[1]) ?? [])));
const kosongSemua = computed(() => !!r.value && !adaGerak.value && !r.value.stok_lama.length && (r.value.kas?.saldo_total ?? 0) === 0);
</script>

<template>
  <Halaman judul="Beranda" :keterangan="`Halo, ${sesi.sesi?.nama ?? ''}.`">
    <template #aksi><PilihPeriode v-model="periode" /></template>

    <div v-if="kosongSemua" class="mx-auto mt-16 max-w-md text-center" data-uji="beranda-kosong">
      <p class="text-lg font-medium">Belum ada catatan.</p>
      <p class="mt-1 text-sm text-muted-foreground">
        Mulai dengan menambah barang, lalu masukkan stok yang ada sekarang lewat
        <RouterLink v-if="sesi.boleh('barang_masuk', 2)" to="/masuk" class="text-primary underline-offset-4 hover:underline">Barang Masuk</RouterLink>
        <span v-else>Barang Masuk</span>
        dengan centang “Stok awal”.
      </p>
    </div>

    <div v-else-if="r" class="flex flex-col gap-5">
      <div class="grid grid-cols-[repeat(auto-fill,minmax(13rem,1fr))] gap-3">
        <template v-if="r.kas">
          <div class="rounded-xl border bg-primary p-4 text-primary-foreground">
            <div class="text-xs opacity-80">Saldo kas</div>
            <div class="text-xl font-semibold tabular-nums" data-uji="kartu-saldo">{{ rupiah(r.kas.saldo_total) }}</div>
            <div class="mt-1 truncate text-xs opacity-80">{{ r.kas.saldo_per_akun.filter((a) => a.aktif || a.saldo).map((a) => `${a.nama} ${rupiah(a.saldo)}`).join(" · ") }}</div>
          </div>
          <div class="rounded-xl border bg-card p-4">
            <div class="text-xs text-muted-foreground">Kas masuk (periode ini)</div>
            <div class="text-xl font-semibold text-emerald-600 tabular-nums dark:text-emerald-400" data-uji="kartu-masuk">{{ rupiah(r.kas.masuk) }}</div>
          </div>
          <div class="rounded-xl border bg-card p-4">
            <div class="text-xs text-muted-foreground">Kas keluar (periode ini)</div>
            <div class="text-xl font-semibold text-rose-600 tabular-nums dark:text-rose-400" data-uji="kartu-keluar">{{ rupiah(r.kas.keluar) }}</div>
          </div>
        </template>
        <div v-if="r.laba !== null" class="rounded-xl border bg-card p-4">
          <div class="text-xs text-muted-foreground">Laba kotor penjualan (periode ini)</div>
          <div class="text-xl font-semibold tabular-nums" data-uji="kartu-laba">{{ rupiah(r.laba) }}</div>
        </div>
        <div v-if="r.nilai_stok !== null" class="rounded-xl border bg-card p-4">
          <div class="text-xs text-muted-foreground">Nilai stok sekarang</div>
          <div class="text-xl font-semibold tabular-nums" data-uji="kartu-nilai-stok">{{ rupiah(r.nilai_stok) }}</div>
        </div>
      </div>

      <div class="grid grid-cols-1 gap-5 xl:grid-cols-2">
        <section v-if="r.deret_kas" class="rounded-xl border bg-card p-4">
          <h2 class="mb-2 text-sm font-semibold">Arus kas</h2>
          <GrafikKas :deret="r.deret_kas" />
        </section>
        <section v-if="adaBarang" class="rounded-xl border bg-card p-4">
          <h2 class="mb-2 text-sm font-semibold">Arus barang <span class="font-normal text-muted-foreground">(jumlah satuan, semua barang)</span></h2>
          <GrafikBarang :deret="r.deret_barang" />
        </section>
      </div>

      <div v-if="adaBarang" class="grid grid-cols-1 gap-5 lg:grid-cols-2 2xl:grid-cols-4">
        <section class="rounded-xl border bg-card p-4" data-uji="daftar-stok-lama">
          <h2 class="text-sm font-semibold">Stok lama, habiskan dulu</h2>
          <p class="mb-2 text-xs text-muted-foreground">Kiriman tertua yang masih bersisa.</p>
          <p v-if="!r.stok_lama.length" class="py-4 text-center text-sm text-muted-foreground">Tidak ada stok.</p>
          <ul class="divide-y text-sm">
            <li v-for="k in r.stok_lama" :key="k.id" class="flex items-baseline justify-between gap-3 py-1.5">
              <span class="min-w-0 truncate">{{ k.barang_nama }} <span class="text-xs text-muted-foreground">{{ k.spek }}</span></span>
              <span class="shrink-0 text-right tabular-nums">{{ jumlah(k.sisa, k.satuan) }}<span class="block text-xs text-muted-foreground">masuk {{ tanggal(k.tanggal_masuk) }}</span></span>
            </li>
          </ul>
        </section>
        <section class="rounded-xl border bg-card p-4" data-uji="daftar-kedaluwarsa">
          <h2 class="text-sm font-semibold">Mendekati kedaluwarsa</h2>
          <p class="mb-2 text-xs text-muted-foreground">Dalam 30 hari, atau sudah lewat.</p>
          <p v-if="!r.hampir_kedaluwarsa.length" class="py-4 text-center text-sm text-muted-foreground">Aman, tidak ada.</p>
          <ul class="divide-y text-sm">
            <li v-for="k in r.hampir_kedaluwarsa" :key="k.id" class="flex items-baseline justify-between gap-3 py-1.5">
              <span class="min-w-0 truncate">{{ k.barang_nama }}</span>
              <span class="shrink-0 text-right tabular-nums" :class="k.sudah_kedaluwarsa ? 'text-destructive' : 'text-amber-600 dark:text-amber-400'">
                {{ jumlah(k.sisa, k.satuan) }}
                <span class="block text-xs">{{ k.sudah_kedaluwarsa ? "lewat" : `${k.hari_ke_kedaluwarsa} hari lagi` }} · {{ tanggal(k.kedaluwarsa) }}</span>
              </span>
            </li>
          </ul>
        </section>
        <section class="rounded-xl border bg-card p-4" data-uji="daftar-menipis">
          <h2 class="text-sm font-semibold">Stok menipis</h2>
          <p class="mb-2 text-xs text-muted-foreground">Di bawah stok minimum.</p>
          <p v-if="!r.stok_menipis.length" class="py-4 text-center text-sm text-muted-foreground">Aman, tidak ada.</p>
          <ul class="divide-y text-sm">
            <li v-for="b in r.stok_menipis" :key="b.id" class="flex items-baseline justify-between gap-3 py-1.5">
              <span class="min-w-0 truncate">{{ b.nama }}</span>
              <span class="shrink-0 text-right text-destructive tabular-nums">{{ jumlah(b.stok, b.satuan) }}<span class="block text-xs text-muted-foreground">minimum {{ jumlah(b.stok_minimum) }}</span></span>
            </li>
          </ul>
        </section>
        <section class="rounded-xl border bg-card p-4" data-uji="daftar-terlaris">
          <h2 class="text-sm font-semibold">Paling banyak keluar</h2>
          <p class="mb-2 text-xs text-muted-foreground">Pada periode ini.</p>
          <p v-if="!r.terlaris.length" class="py-4 text-center text-sm text-muted-foreground">Belum ada barang keluar.</p>
          <ul class="flex flex-col gap-2 text-sm">
            <li v-for="[nama, n] in r.terlaris" :key="nama">
              <div class="flex justify-between gap-3"><span class="truncate">{{ nama }}</span><span class="shrink-0 tabular-nums">{{ jumlah(n) }}</span></div>
              <div class="mt-1 h-1.5 rounded-full bg-muted"><div class="h-full rounded-full bg-primary" :style="{ width: `${(n / terlarisMaks) * 100}%` }" /></div>
            </li>
          </ul>
        </section>
      </div>
    </div>
  </Halaman>
</template>
