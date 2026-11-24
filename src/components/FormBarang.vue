<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { toast } from "vue-sonner";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import FotoPicker from "@/components/FotoPicker.vue";
import InputJumlah from "@/components/InputJumlah.vue";
import InputRupiah from "@/components/InputRupiah.vue";
import Kolom from "@/components/Kolom.vue";
import { panggil, pesanGalat } from "@/lib/api";
import { kelasKontrol } from "@/lib/gaya";
import type { Barang, BarangMasukan, Kategori } from "@/lib/types";

/** `barang` = null berarti menambah barang baru. */
const props = defineProps<{ buka: boolean; barang: Barang | null }>();
const emit = defineEmits<{ "update:buka": [v: boolean]; tersimpan: [id: number] }>();

const kategori = ref<Kategori[]>([]);
const f = ref<BarangMasukan>(kosong());
const stokMin = ref<number | null>(null);
const kategoriBaru = ref<string | null>(null);
const galat = ref("");
const sibuk = ref(false);

function kosong(): BarangMasukan {
  return { kode: "", nama: "", kategori_id: null, satuan: "", harga_jual: 0, stok_minimum: 0, foto_sumber: null, aktif: true };
}

watch(
  () => props.buka,
  async (b) => {
    if (!b) return;
    galat.value = "";
    kategoriBaru.value = null;
    const a = props.barang;
    f.value = a ? { kode: a.kode, nama: a.nama, kategori_id: a.kategori_id, satuan: a.satuan, harga_jual: a.harga_jual, stok_minimum: a.stok_minimum, foto_sumber: null, aktif: a.aktif } : kosong();
    stokMin.value = a && a.stok_minimum > 0 ? a.stok_minimum : null;
    try {
      kategori.value = await panggil<Kategori[]>("kategori_barang_daftar");
    } catch (e) {
      galat.value = pesanGalat(e);
    }
  },
);

const hargaSah = ref<number | null>(0);
watch(
  () => f.value.harga_jual,
  (n) => (hargaSah.value = n),
  { immediate: true },
);
const lengkap = computed(() => f.value.nama.trim() !== "" && f.value.satuan.trim() !== "" && hargaSah.value !== null);

async function simpan() {
  galat.value = "";
  sibuk.value = true;
  try {
    if (kategoriBaru.value?.trim()) {
      f.value.kategori_id = await panggil<number>("kategori_barang_tambah", { nama: kategoriBaru.value });
      kategoriBaru.value = null;
    }
    const masukan: BarangMasukan = { ...f.value, harga_jual: hargaSah.value as number, stok_minimum: stokMin.value ?? 0 };
    let id = props.barang?.id ?? 0;
    if (props.barang) await panggil("barang_ubah", { id, masukan });
    else id = await panggil<number>("barang_tambah", { masukan });
    toast.success(props.barang ? "Perubahan barang tersimpan." : "Barang baru tersimpan.");
    emit("tersimpan", id);
    emit("update:buka", false);
  } catch (e) {
    galat.value = pesanGalat(e);
  } finally {
    sibuk.value = false;
  }
}
</script>

<template>
  <Dialog :open="buka" @update:open="emit('update:buka', $event)">
    <DialogContent class="sm:max-w-lg">
      <DialogHeader>
        <DialogTitle>{{ barang ? "Ubah barang" : "Barang baru" }}</DialogTitle>
        <DialogDescription>Stok dan harga beli dicatat per kiriman lewat Barang Masuk, bukan di sini.</DialogDescription>
      </DialogHeader>
      <form class="grid grid-cols-2 gap-3" @submit.prevent="simpan">
        <Kolom label="Nama barang" class="col-span-2"><Input v-model="f.nama" data-uji="barang-nama" placeholder="Misalnya: Pakan Apung 781-2" /></Kolom>
        <Kolom label="Satuan" bantuan="karung, kg, buah, botol…"><Input v-model="f.satuan" data-uji="barang-satuan" /></Kolom>
        <Kolom label="Kode" bantuan="Kosongkan agar dibuat otomatis"><Input v-model="f.kode" data-uji="barang-kode" /></Kolom>
        <Kolom label="Harga jual per satuan"><InputRupiah v-model="hargaSah" data-uji="barang-harga" /></Kolom>
        <Kolom label="Stok minimum" bantuan="Diberi tanda bila stok di bawah ini"><InputJumlah v-model="stokMin" :satuan="f.satuan" data-uji="barang-minimum" /></Kolom>
        <Kolom label="Kategori" class="col-span-2">
          <div class="flex gap-2">
            <Input v-if="kategoriBaru !== null" v-model="kategoriBaru" placeholder="Nama kategori baru" data-uji="barang-kategori-baru" />
            <select v-else v-model="f.kategori_id" :class="kelasKontrol" data-uji="barang-kategori">
              <option :value="null">Tanpa kategori</option>
              <option v-for="k in kategori" :key="k.id" :value="k.id">{{ k.nama }}</option>
            </select>
            <Button type="button" variant="outline" data-uji="barang-kategori-ganti" @click="kategoriBaru = kategoriBaru === null ? '' : null">{{ kategoriBaru === null ? "Kategori baru" : "Pilih yang ada" }}</Button>
          </div>
        </Kolom>
        <Kolom label="Foto" class="col-span-2"><FotoPicker v-model="f.foto_sumber" :lama="barang?.foto" /></Kolom>
        <label v-if="barang" class="col-span-2 flex items-center gap-2 text-sm">
          <input v-model="f.aktif" type="checkbox" class="size-4 accent-primary" data-uji="barang-aktif" />
          Barang masih dijual (hilangkan centang untuk menyembunyikannya)
        </label>
        <p v-if="galat" class="col-span-2 text-sm text-destructive" data-uji="barang-galat">{{ galat }}</p>
        <DialogFooter class="col-span-2">
          <Button type="button" variant="outline" @click="emit('update:buka', false)">Tidak jadi</Button>
          <Button type="submit" :disabled="!lengkap || sibuk" data-uji="barang-simpan">Simpan</Button>
        </DialogFooter>
      </form>
    </DialogContent>
  </Dialog>
</template>
