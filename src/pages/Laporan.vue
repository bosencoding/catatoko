<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { save } from "@tauri-apps/plugin-dialog";
import { toast } from "vue-sonner";
import { FileSpreadsheetIcon, FileTextIcon } from "@lucide/vue";
import { Button } from "@/components/ui/button";
import Halaman from "@/components/Halaman.vue";
import PilihBarang from "@/components/PilihBarang.vue";
import PilihPeriode, { bulanIni } from "@/components/PilihPeriode.vue";
import { panggil, pesanGalat } from "@/lib/api";
import { jumlah, rupiah, tanggal } from "@/lib/format";
import type { Barang, JenisLaporan, Laporan, Sel } from "@/lib/types";
import { useSesi } from "@/stores/sesi";

type Kunci = JenisLaporan["jenis"];
const sesi = useSesi();

const SEMUA: { kunci: Kunci; nama: string; ket: string; hargaBeli: boolean }[] = [
  { kunci: "arus_kas", nama: "Arus kas", ket: "Semua uang masuk dan keluar", hargaBeli: false },
  { kunci: "laba_kotor", nama: "Laba kotor", ket: "Harga jual dikurangi harga beli", hargaBeli: true },
  { kunci: "posisi_stok", nama: "Posisi stok", ket: "Sisa dan nilai tiap kiriman saat ini", hargaBeli: true },
  { kunci: "kartu_stok", nama: "Kartu stok", ket: "Riwayat satu barang", hargaBeli: false },
  { kunci: "barang_masuk", nama: "Barang masuk", ket: "Semua kiriman yang diterima", hargaBeli: true },
  { kunci: "barang_keluar", nama: "Barang keluar", ket: "Semua barang yang keluar", hargaBeli: false },
];
const pilihan = computed(() => SEMUA.filter((j) => !j.hargaBeli || sesi.bolehHargaBeli));

const jenis = ref<Kunci>("arus_kas");
const periode = ref(bulanIni());
const barangId = ref<number | null>(null);
const barang = ref<Barang[]>([]);
const laporan = ref<Laporan | null>(null);
const galat = ref("");
const sibuk = ref(false);

const permintaan = computed<JenisLaporan | null>(() => {
  if (jenis.value === "kartu_stok") return barangId.value === null ? null : { jenis: "kartu_stok", barang_id: barangId.value };
  return { jenis: jenis.value } as JenisLaporan;
});

let urut = 0;
async function muat() {
  galat.value = "";
  const no = ++urut;
  if (!permintaan.value) {
    laporan.value = null;
    return;
  }
  try {
    const l = await panggil<Laporan>("laporan_buat", { jenis: permintaan.value, ...periode.value });
    if (no === urut) laporan.value = l;
  } catch (e) {
    if (no !== urut) return;
    laporan.value = null;
    galat.value = pesanGalat(e);
  }
}
watch([permintaan, periode], muat, { deep: true });
onMounted(async () => {
  await muat();
  try {
    barang.value = await panggil<Barang[]>("barang_daftar", { cari: null, termasuk_nonaktif: true });
  } catch {
    // tanpa hak barang, kartu stok tidak bisa dipilih
  }
});

function isi(s: Sel): string {
  switch (s.t) {
    case "teks":
      return s.v;
    case "rupiah":
      return rupiah(s.v);
    case "jumlah":
      return jumlah(s.v);
    case "tanggal":
      return tanggal(s.v);
    default:
      return "";
  }
}
const angka = (s: Sel | undefined) => s?.t === "rupiah" || s?.t === "jumlah";
const kolomAngka = computed(() => laporan.value?.kolom.map((_, i) => angka(laporan.value?.baris[0]?.[i]) || angka(laporan.value?.total?.[i])) ?? []);

async function ekspor(format: "xlsx" | "pdf") {
  if (!permintaan.value) return;
  const nama = `${jenis.value.replace("_", "-")}-${periode.value.dari}-sd-${periode.value.sampai}.${format}`;
  const tujuan = await save({ defaultPath: nama, filters: [{ name: format === "xlsx" ? "Excel" : "PDF", extensions: [format] }] });
  if (!tujuan) return;
  sibuk.value = true;
  try {
    const jalur = await panggil<string>("laporan_ekspor", { jenis: permintaan.value, ...periode.value, format, tujuan });
    toast.success(`Laporan tersimpan di ${jalur}`);
  } catch (e) {
    toast.error(pesanGalat(e));
  } finally {
    sibuk.value = false;
  }
}
</script>

<template>
  <Halaman judul="Laporan" keterangan="Lihat di layar, lalu unduh sebagai Excel atau PDF.">
    <template #aksi>
      <Button variant="outline" :disabled="!laporan || sibuk" data-uji="unduh-xlsx" @click="ekspor('xlsx')"><FileSpreadsheetIcon /> Unduh Excel</Button>
      <Button variant="outline" :disabled="!laporan || sibuk" data-uji="unduh-pdf" @click="ekspor('pdf')"><FileTextIcon /> Unduh PDF</Button>
    </template>

    <div class="flex flex-col gap-4">
      <div class="grid grid-cols-[repeat(auto-fill,minmax(11rem,1fr))] gap-2">
        <button
          v-for="j in pilihan"
          :key="j.kunci"
          type="button"
          class="rounded-xl border p-3 text-left transition-colors"
          :class="jenis === j.kunci ? 'border-primary bg-primary/10' : 'bg-card hover:bg-accent/50'"
          :data-uji="`laporan-${j.kunci}`"
          @click="jenis = j.kunci"
        >
          <div class="text-sm font-medium">{{ j.nama }}</div>
          <div class="text-xs text-muted-foreground">{{ j.ket }}</div>
        </button>
      </div>

      <div class="flex flex-wrap items-center gap-2">
        <PilihPeriode v-if="jenis !== 'posisi_stok'" v-model="periode" />
        <div v-if="jenis === 'kartu_stok'" class="w-80"><PilihBarang v-model="barangId" :daftar="barang" uji="laporan" /></div>
      </div>

      <p v-if="galat" class="text-sm text-destructive" data-uji="laporan-galat">{{ galat }}</p>
      <p v-else-if="jenis === 'kartu_stok' && barangId === null" class="py-10 text-center text-sm text-muted-foreground">Pilih barang untuk melihat kartu stoknya.</p>

      <section v-if="laporan" class="rounded-xl border bg-card">
        <header class="border-b px-4 py-3">
          <h2 class="font-semibold" data-uji="laporan-judul">{{ laporan.judul }}</h2>
          <p class="text-xs text-muted-foreground">{{ laporan.nama_toko }} · {{ laporan.periode }}</p>
        </header>
        <div class="overflow-auto">
          <table class="w-full text-sm" data-uji="laporan-tabel">
            <thead class="bg-muted/60 text-xs text-muted-foreground">
              <tr><th v-for="(k, i) in laporan.kolom" :key="k" class="px-3 py-2 font-medium whitespace-nowrap" :class="kolomAngka[i] ? 'text-right' : 'text-left'">{{ k }}</th></tr>
            </thead>
            <tbody>
              <tr v-if="!laporan.baris.length"><td :colspan="laporan.kolom.length" class="px-3 py-8 text-center text-muted-foreground">Tidak ada data pada periode ini.</td></tr>
              <tr v-for="(b, i) in laporan.baris" :key="i" class="border-t">
                <td v-for="(s, j) in b" :key="j" class="px-3 py-1.5" :class="angka(s) ? 'text-right tabular-nums whitespace-nowrap' : s.t === 'tanggal' ? 'whitespace-nowrap' : ''">{{ isi(s) }}</td>
              </tr>
            </tbody>
            <tfoot v-if="laporan.total && laporan.baris.length">
              <tr class="border-t-2 bg-muted/40 font-semibold" data-uji="laporan-total">
                <td v-for="(s, j) in laporan.total" :key="j" class="px-3 py-2" :class="angka(s) ? 'text-right tabular-nums whitespace-nowrap' : ''">{{ isi(s) }}</td>
              </tr>
            </tfoot>
          </table>
        </div>
        <dl v-if="laporan.ringkasan.length" class="grid max-w-md grid-cols-[1fr_auto] gap-x-6 gap-y-1 border-t px-4 py-3 text-sm" data-uji="laporan-ringkasan">
          <template v-for="[label, s] in laporan.ringkasan" :key="label">
            <dt class="text-muted-foreground">{{ label }}</dt>
            <dd class="text-right font-medium tabular-nums">{{ isi(s) }}</dd>
          </template>
        </dl>
      </section>
    </div>
  </Halaman>
</template>
