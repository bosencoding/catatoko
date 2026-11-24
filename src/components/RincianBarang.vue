<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { toast } from "vue-sonner";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import DialogBatal from "@/components/DialogBatal.vue";
import Foto from "@/components/Foto.vue";
import FotoPicker from "@/components/FotoPicker.vue";
import InputJumlah from "@/components/InputJumlah.vue";
import InputRupiah from "@/components/InputRupiah.vue";
import InputTanggal from "@/components/InputTanggal.vue";
import Kolom from "@/components/Kolom.vue";
import TabelKiriman from "@/components/TabelKiriman.vue";
import { panggil, pesanGalat } from "@/lib/api";
import { hariIni, jumlah, rupiah, tanggal } from "@/lib/format";
import { kelasKontrol } from "@/lib/gaya";
import type { Barang, Kiriman, PerbaikanKiriman } from "@/lib/types";
import { useSesi } from "@/stores/sesi";

const props = defineProps<{ buka: boolean; barangId: number | null }>();
const emit = defineEmits<{ "update:buka": [v: boolean]; ubah: [barang: Barang]; berubah: [] }>();

const sesi = useSesi();
const bolehKelola = computed(() => sesi.boleh("barang_masuk", 3));
const barang = ref<Barang | null>(null);
const kiriman = ref<Kiriman[]>([]);
const semua = ref(false);
const sibuk = ref(false);

const perbaiki = ref<Kiriman | null>(null);
const fp = ref<PerbaikanKiriman>({ harga_beli: 0, spek: null, kedaluwarsa: null, pemasok: null, foto_sumber: null });
const hargaBaru = ref<number | null>(0);
const sesuaikan = ref<Kiriman | null>(null);
const fs = ref({ arah: "kurang" as "kurang" | "tambah", jumlah: null as number | null, alasan: "rusak", tanggal: hariIni(), catatan: "" });
const batal = ref<Kiriman | null>(null);
const galat = ref("");

const ALASAN = [
  ["rusak", "Rusak"],
  ["kedaluwarsa", "Kedaluwarsa"],
  ["hilang", "Hilang"],
  ["pakai_sendiri", "Pakai sendiri"],
  ["koreksi", "Koreksi hitung"],
];

async function muat() {
  if (props.barangId === null) return;
  try {
    [barang.value, kiriman.value] = await Promise.all([
      panggil<Barang>("barang_ambil", { id: props.barangId }),
      panggil<Kiriman[]>("kiriman_barang", { barang_id: props.barangId, hanya_bersisa: !semua.value }),
    ]);
  } catch (e) {
    toast.error(pesanGalat(e));
  }
}
watch(() => [props.buka, props.barangId, semua.value], ([b]) => b && muat(), { immediate: true });
defineExpose({ muat });

function bukaPerbaiki(k: Kiriman) {
  galat.value = "";
  fp.value = { harga_beli: k.harga_beli ?? 0, spek: k.spek, kedaluwarsa: k.kedaluwarsa, pemasok: k.pemasok, foto_sumber: null };
  hargaBaru.value = k.harga_beli ?? 0;
  perbaiki.value = k;
}
function bukaSesuaikan(k: Kiriman) {
  galat.value = "";
  fs.value = { arah: "kurang", jumlah: null, alasan: "rusak", tanggal: hariIni(), catatan: "" };
  sesuaikan.value = k;
}

async function jalankan(kerja: () => Promise<unknown>, pesan: string, tutup: () => void) {
  galat.value = "";
  sibuk.value = true;
  try {
    await kerja();
    toast.success(pesan);
    tutup();
    await muat();
    emit("berubah");
  } catch (e) {
    galat.value = pesanGalat(e);
    if (batal.value) toast.error(galat.value);
  } finally {
    sibuk.value = false;
  }
}

const simpanPerbaikan = () =>
  jalankan(
    () => panggil("kiriman_perbaiki", { id: perbaiki.value!.id, perbaikan: { ...fp.value, harga_beli: hargaBaru.value, spek: fp.value.spek || null, kedaluwarsa: fp.value.kedaluwarsa || null, pemasok: fp.value.pemasok || null } }),
    "Kiriman diperbaiki.",
    () => (perbaiki.value = null),
  );
const simpanPenyesuaian = () =>
  jalankan(
    () =>
      panggil("stok_sesuaikan", {
        kiriman_id: sesuaikan.value!.id,
        tanggal: fs.value.tanggal,
        selisih: (fs.value.arah === "kurang" ? -1 : 1) * (fs.value.jumlah as number),
        alasan: fs.value.alasan,
        catatan: fs.value.catatan.trim() || null,
      }),
    "Penyesuaian stok tersimpan.",
    () => (sesuaikan.value = null),
  );
const batalkan = (alasan: string) => jalankan(() => panggil("masuk_batal", { kiriman_id: batal.value!.id, alasan }), "Barang masuk dibatalkan.", () => (batal.value = null));
</script>

<template>
  <Dialog :open="buka" @update:open="emit('update:buka', $event)">
    <DialogContent class="sm:max-w-4xl">
      <DialogHeader>
        <DialogTitle>{{ barang?.nama ?? "Barang" }}</DialogTitle>
        <DialogDescription>{{ barang?.kode }}<template v-if="barang?.kategori"> · {{ barang.kategori }}</template><template v-if="barang && !barang.aktif"> · tidak dijual lagi</template></DialogDescription>
      </DialogHeader>
      <div v-if="barang" class="flex min-w-0 flex-col gap-4">
        <div class="flex items-center gap-4">
          <Foto :relatif="barang.foto" class="size-24 shrink-0" />
          <dl class="grid min-w-0 flex-1 grid-cols-2 gap-x-6 gap-y-2 text-sm md:grid-cols-4">
            <div><dt class="text-xs text-muted-foreground">Total stok</dt><dd class="text-lg font-semibold tabular-nums" data-uji="rincian-stok">{{ jumlah(barang.stok, barang.satuan) }}</dd></div>
            <div><dt class="text-xs text-muted-foreground">Harga jual</dt><dd class="font-medium tabular-nums">{{ rupiah(barang.harga_jual) }}</dd></div>
            <div v-if="sesi.bolehHargaBeli"><dt class="text-xs text-muted-foreground">Nilai stok</dt><dd class="font-medium tabular-nums">{{ rupiah(barang.nilai_stok) }}</dd></div>
            <div><dt class="text-xs text-muted-foreground">Stok minimum</dt><dd class="font-medium tabular-nums">{{ barang.stok_minimum ? jumlah(barang.stok_minimum, barang.satuan) : "—" }}</dd></div>
          </dl>
          <Button v-if="sesi.boleh('barang', 3)" variant="outline" data-uji="rincian-ubah" @click="emit('ubah', barang)">Ubah barang</Button>
        </div>
        <div class="flex items-center justify-between">
          <h3 class="text-sm font-semibold">Kiriman (stok lama di atas)</h3>
          <label class="flex items-center gap-2 text-xs text-muted-foreground"><input v-model="semua" type="checkbox" class="size-3.5 accent-primary" data-uji="rincian-semua" /> Tampilkan yang habis dan dibatalkan</label>
        </div>
        <div class="max-h-[45vh] overflow-auto">
          <TabelKiriman :kiriman="kiriman">
            <template #aksi="{ kiriman: k }">
              <template v-if="bolehKelola && !k.batal">
                <Button variant="ghost" size="sm" :data-uji="`perbaiki-${k.id}`" @click="bukaPerbaiki(k)">Perbaiki</Button>
                <Button variant="ghost" size="sm" :data-uji="`sesuaikan-${k.id}`" @click="bukaSesuaikan(k)">Sesuaikan</Button>
                <Button v-if="!k.pernah_keluar" variant="ghost" size="sm" class="text-destructive" :data-uji="`batal-kiriman-${k.id}`" @click="batal = k">Batalkan</Button>
              </template>
            </template>
          </TabelKiriman>
        </div>
      </div>
    </DialogContent>
  </Dialog>

  <Dialog :open="perbaiki !== null" @update:open="!$event && (perbaiki = null)">
    <DialogContent class="sm:max-w-md">
      <DialogHeader>
        <DialogTitle>Perbaiki kiriman {{ perbaiki ? tanggal(perbaiki.tanggal_masuk) : "" }}</DialogTitle>
        <DialogDescription>Jumlah tidak diubah di sini (pakai Sesuaikan). Bila harga beli diubah, catatan kas pembeliannya ikut disesuaikan.</DialogDescription>
      </DialogHeader>
      <form class="flex flex-col gap-3" @submit.prevent="simpanPerbaikan">
        <Kolom label="Harga beli per satuan"><InputRupiah v-model="hargaBaru" data-uji="perbaiki-harga" /></Kolom>
        <Kolom label="Spek"><Input :model-value="fp.spek ?? ''" data-uji="perbaiki-spek" @update:model-value="fp.spek = String($event)" /></Kolom>
        <Kolom label="Kedaluwarsa"><InputTanggal v-model="fp.kedaluwarsa" data-uji="perbaiki-exp" /></Kolom>
        <Kolom label="Pemasok"><Input :model-value="fp.pemasok ?? ''" @update:model-value="fp.pemasok = String($event)" /></Kolom>
        <Kolom label="Foto kiriman"><FotoPicker v-model="fp.foto_sumber" :lama="perbaiki?.foto" /></Kolom>
        <p v-if="galat" class="text-sm text-destructive" data-uji="perbaiki-galat">{{ galat }}</p>
        <DialogFooter>
          <Button type="button" variant="outline" @click="perbaiki = null">Tidak jadi</Button>
          <Button type="submit" :disabled="hargaBaru === null || sibuk" data-uji="perbaiki-simpan">Simpan perbaikan</Button>
        </DialogFooter>
      </form>
    </DialogContent>
  </Dialog>

  <Dialog :open="sesuaikan !== null" @update:open="!$event && (sesuaikan = null)">
    <DialogContent class="sm:max-w-md">
      <DialogHeader>
        <DialogTitle>Penyesuaian stok</DialogTitle>
        <DialogDescription v-if="sesuaikan">Kiriman {{ tanggal(sesuaikan.tanggal_masuk) }}, sisa {{ jumlah(sesuaikan.sisa, sesuaikan.satuan) }}. Tidak memengaruhi kas.</DialogDescription>
      </DialogHeader>
      <form class="grid grid-cols-2 gap-3" @submit.prevent="simpanPenyesuaian">
        <Kolom label="Stok">
          <select v-model="fs.arah" :class="kelasKontrol" data-uji="sesuai-arah"><option value="kurang">Berkurang</option><option value="tambah">Bertambah</option></select>
        </Kolom>
        <Kolom label="Sebanyak"><InputJumlah v-model="fs.jumlah" :satuan="sesuaikan?.satuan" wajib data-uji="sesuai-jumlah" /></Kolom>
        <Kolom label="Alasan">
          <select v-model="fs.alasan" :class="kelasKontrol" data-uji="sesuai-alasan"><option v-for="[v, l] in ALASAN" :key="v" :value="v">{{ l }}</option></select>
        </Kolom>
        <Kolom label="Tanggal"><InputTanggal v-model="fs.tanggal" /></Kolom>
        <Kolom label="Catatan (opsional)" class="col-span-2"><Input v-model="fs.catatan" /></Kolom>
        <p v-if="galat" class="col-span-2 text-sm text-destructive" data-uji="sesuai-galat">{{ galat }}</p>
        <DialogFooter class="col-span-2">
          <Button type="button" variant="outline" @click="sesuaikan = null">Tidak jadi</Button>
          <Button type="submit" :disabled="fs.jumlah === null || sibuk" data-uji="sesuai-simpan">Simpan penyesuaian</Button>
        </DialogFooter>
      </form>
    </DialogContent>
  </Dialog>

  <DialogBatal :buka="batal !== null" judul="Batalkan barang masuk ini?" keterangan="Stok kiriman ini dihapus dan catatan kas pembeliannya ikut dibatalkan." :sibuk="sibuk" @update:buka="!$event && (batal = null)" @konfirmasi="batalkan" />
</template>
