<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { toast } from "vue-sonner";
import { PlusIcon } from "@lucide/vue";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import Foto from "@/components/Foto.vue";
import FormBarang from "@/components/FormBarang.vue";
import Halaman from "@/components/Halaman.vue";
import RincianBarang from "@/components/RincianBarang.vue";
import { panggil, pesanGalat } from "@/lib/api";
import { jumlah, rupiah } from "@/lib/format";
import { kelasKontrol } from "@/lib/gaya";
import type { Barang } from "@/lib/types";
import { useSesi } from "@/stores/sesi";

const sesi = useSesi();
const daftar = ref<Barang[]>([]);
const cari = ref("");
const kategori = ref<string>("");
const nonaktif = ref(false);
const sudahMuat = ref(false);

const formBuka = ref(false);
const diubah = ref<Barang | null>(null);
const rincianId = ref<number | null>(null);

const semuaKategori = computed(() => [...new Set(daftar.value.map((b) => b.kategori).filter((k): k is string => !!k))].sort());
const tampil = computed(() => {
  const c = cari.value.trim().toLowerCase();
  return daftar.value.filter((b) => (!kategori.value || b.kategori === kategori.value) && (!c || b.nama.toLowerCase().includes(c) || b.kode.toLowerCase().includes(c)));
});
const menipis = (b: Barang) => b.stok_minimum > 0 && b.stok < b.stok_minimum;

async function muat() {
  try {
    daftar.value = await panggil<Barang[]>("barang_daftar", { cari: null, termasuk_nonaktif: nonaktif.value });
  } catch (e) {
    toast.error(pesanGalat(e));
  } finally {
    sudahMuat.value = true;
  }
}
watch(nonaktif, muat);
onMounted(muat);

function tambah() {
  diubah.value = null;
  formBuka.value = true;
}
function ubah(b: Barang) {
  diubah.value = b;
  formBuka.value = true;
}
</script>

<template>
  <Halaman judul="Barang" :keterangan="`${tampil.length} barang`">
    <template #aksi>
      <Input v-model="cari" class="w-56" placeholder="Cari nama atau kode…" data-uji="cari-barang" />
      <div v-if="semuaKategori.length" class="w-44">
        <select v-model="kategori" :class="kelasKontrol">
          <option value="">Semua kategori</option>
          <option v-for="k in semuaKategori" :key="k" :value="k">{{ k }}</option>
        </select>
      </div>
      <label class="flex items-center gap-2 text-sm text-muted-foreground"><input v-model="nonaktif" type="checkbox" class="size-4 accent-primary" /> Termasuk yang tidak dijual</label>
      <Button v-if="sesi.boleh('barang', 2)" data-uji="barang-baru" @click="tambah"><PlusIcon /> Barang baru</Button>
    </template>

    <div v-if="sudahMuat && !daftar.length" class="mx-auto mt-16 max-w-sm text-center text-muted-foreground" data-uji="barang-kosong">
      <p class="font-medium text-foreground">Belum ada barang.</p>
      <p class="mt-1 text-sm">Mulai dengan menambah barang, lalu catat stoknya lewat Barang Masuk.</p>
    </div>
    <div v-else-if="sudahMuat && !tampil.length" class="mt-16 text-center text-sm text-muted-foreground">Tidak ada barang yang cocok dengan pencarian.</div>

    <div class="grid grid-cols-[repeat(auto-fill,minmax(15rem,1fr))] gap-4">
      <button
        v-for="b in tampil"
        :key="b.id"
        type="button"
        class="flex gap-3 rounded-xl border bg-card p-3 text-left transition-colors hover:border-primary/50 hover:bg-accent/40"
        :class="b.aktif ? '' : 'opacity-60'"
        :data-uji="`barang-${b.id}`"
        @click="rincianId = b.id"
      >
        <Foto :relatif="b.foto" kecil class="size-16 shrink-0" />
        <div class="min-w-0 flex-1">
          <div class="truncate font-medium" :title="b.nama">{{ b.nama }}</div>
          <div class="truncate text-xs text-muted-foreground">{{ b.kode }}<template v-if="b.kategori"> · {{ b.kategori }}</template></div>
          <div class="mt-1.5 flex flex-wrap items-center gap-1.5">
            <span class="text-sm font-semibold tabular-nums" :class="b.stok > 0 ? '' : 'text-destructive'">{{ jumlah(b.stok, b.satuan) }}</span>
            <Badge v-if="menipis(b)" variant="destructive">Stok menipis</Badge>
            <Badge v-if="!b.aktif" variant="outline">tidak dijual</Badge>
          </div>
          <div class="mt-0.5 text-xs text-muted-foreground tabular-nums">Jual {{ rupiah(b.harga_jual) }}</div>
        </div>
      </button>
    </div>

    <FormBarang v-model:buka="formBuka" :barang="diubah" @tersimpan="muat" />
    <RincianBarang :buka="rincianId !== null && !formBuka" :barang-id="rincianId" @update:buka="!$event && !formBuka && (rincianId = null)" @ubah="ubah" @berubah="muat" />
  </Halaman>
</template>
