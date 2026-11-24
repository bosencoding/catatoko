<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { toast } from "vue-sonner";
import { PlusIcon } from "@lucide/vue";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import DialogBatal from "@/components/DialogBatal.vue";
import FormBarang from "@/components/FormBarang.vue";
import FotoPicker from "@/components/FotoPicker.vue";
import Halaman from "@/components/Halaman.vue";
import InputJumlah from "@/components/InputJumlah.vue";
import InputRupiah from "@/components/InputRupiah.vue";
import InputTanggal from "@/components/InputTanggal.vue";
import Kolom from "@/components/Kolom.vue";
import PilihBarang from "@/components/PilihBarang.vue";
import { panggil, pesanGalat } from "@/lib/api";
import { hariIni, jumlah, nilai, rupiah, tanggal } from "@/lib/format";
import { kelasKontrol } from "@/lib/gaya";
import type { AkunKas, Barang, Kiriman, MasukMasukan } from "@/lib/types";
import { useSesi } from "@/stores/sesi";

const sesi = useSesi();
const bolehTambah = computed(() => sesi.boleh("barang_masuk", 2));
const bolehBatal = computed(() => sesi.boleh("barang_masuk", 3));

const barang = ref<Barang[]>([]);
const akun = ref<AkunKas[]>([]);
const riwayat = ref<Kiriman[]>([]);
const sibuk = ref(false);
const galat = ref("");
const formBarang = ref(false);
const batal = ref<Kiriman | null>(null);

const kosong = () => ({
  barang_id: null as number | null,
  tanggal_masuk: hariIni(),
  jumlah: null as number | null,
  harga_beli: null as number | null,
  spek: "",
  kedaluwarsa: "",
  pemasok: "",
  foto_sumber: null as string | null,
  stok_awal: false,
  akun_kas_id: null as number | null,
});
const f = reactive(kosong());

const terpilih = computed(() => barang.value.find((b) => b.id === f.barang_id));
const total = computed(() => (f.jumlah !== null && f.harga_beli !== null ? nilai(f.jumlah, f.harga_beli) : 0));
const lengkap = computed(() => f.barang_id !== null && f.jumlah !== null && f.harga_beli !== null && f.tanggal_masuk !== "" && (f.stok_awal || f.akun_kas_id !== null));

async function muat() {
  [barang.value, akun.value, riwayat.value] = await Promise.all([
    panggil<Barang[]>("barang_daftar", { cari: null, termasuk_nonaktif: false }),
    panggil<AkunKas[]>("akun_kas_daftar"),
    panggil<Kiriman[]>("masuk_terbaru", { batas: 50 }),
  ]);
  akun.value = akun.value.filter((a) => a.aktif);
  if (!akun.value.some((a) => a.id === f.akun_kas_id)) f.akun_kas_id = akun.value[0]?.id ?? null;
}

async function simpan() {
  galat.value = "";
  sibuk.value = true;
  const masukan: MasukMasukan = {
    barang_id: f.barang_id as number,
    tanggal_masuk: f.tanggal_masuk,
    jumlah: f.jumlah as number,
    harga_beli: f.harga_beli as number,
    spek: f.spek.trim() || null,
    kedaluwarsa: f.kedaluwarsa || null,
    pemasok: f.pemasok.trim() || null,
    foto_sumber: f.foto_sumber,
    stok_awal: f.stok_awal,
    akun_kas_id: f.stok_awal ? null : f.akun_kas_id,
  };
  try {
    await panggil<number>("masuk_tambah", { masukan });
    toast.success(`Barang masuk tersimpan: ${jumlah(masukan.jumlah, terpilih.value?.satuan)} ${terpilih.value?.nama ?? ""}.`);
    const { tanggal_masuk, akun_kas_id, stok_awal } = f;
    Object.assign(f, kosong(), { tanggal_masuk, akun_kas_id, stok_awal });
    await muat();
  } catch (e) {
    galat.value = pesanGalat(e);
  } finally {
    sibuk.value = false;
  }
}

async function barangBaruTersimpan(id: number) {
  await muat();
  f.barang_id = id;
}

async function batalkan(alasan: string) {
  if (!batal.value) return;
  sibuk.value = true;
  try {
    await panggil("masuk_batal", { kiriman_id: batal.value.id, alasan });
    toast.success("Barang masuk dibatalkan.");
    batal.value = null;
    await muat();
  } catch (e) {
    toast.error(pesanGalat(e));
  } finally {
    sibuk.value = false;
  }
}

onMounted(() => muat().catch((e) => toast.error(pesanGalat(e))));
</script>

<template>
  <Halaman judul="Barang Masuk" keterangan="Setiap barang masuk dicatat sebagai satu kiriman dengan harga dan speknya sendiri.">
    <div class="grid grid-cols-1 gap-6 xl:grid-cols-[26rem_minmax(0,1fr)]">
      <form v-if="bolehTambah" class="flex h-fit flex-col gap-3 rounded-xl border bg-card p-4" @submit.prevent="simpan">
        <Kolom label="Barang">
          <div class="flex gap-2">
            <PilihBarang v-model="f.barang_id" :daftar="barang" uji="masuk" class="flex-1" />
            <Button v-if="sesi.boleh('barang', 2)" type="button" variant="outline" title="Barang baru" data-uji="masuk-barang-baru" @click="formBarang = true"><PlusIcon /> Baru</Button>
          </div>
        </Kolom>
        <div class="grid grid-cols-2 gap-3">
          <Kolom label="Jumlah"><InputJumlah v-model="f.jumlah" :satuan="terpilih?.satuan" wajib data-uji="masuk-jumlah" /></Kolom>
          <Kolom label="Harga beli per satuan"><InputRupiah v-model="f.harga_beli" wajib data-uji="masuk-harga" /></Kolom>
          <Kolom label="Tanggal masuk"><InputTanggal v-model="f.tanggal_masuk" data-uji="masuk-tanggal" /></Kolom>
          <Kolom label="Kedaluwarsa (opsional)"><InputTanggal v-model="f.kedaluwarsa" data-uji="masuk-exp" /></Kolom>
        </div>
        <Kolom label="Spek (opsional)" bantuan="Yang membedakan kiriman ini, misalnya: protein 31%, butir 2 mm"><Input v-model="f.spek" data-uji="masuk-spek" /></Kolom>
        <Kolom label="Pemasok (opsional)"><Input v-model="f.pemasok" data-uji="masuk-pemasok" /></Kolom>
        <Kolom label="Foto kiriman (opsional)"><FotoPicker v-model="f.foto_sumber" /></Kolom>
        <label class="flex items-start gap-2 rounded-lg border p-3 text-sm">
          <input v-model="f.stok_awal" type="checkbox" class="mt-0.5 size-4 accent-primary" data-uji="masuk-stok-awal" />
          <span><b class="font-medium">Stok awal</b>: barang ini sudah ada sebelum memakai aplikasi, jadi tidak mengurangi kas.</span>
        </label>
        <Kolom v-if="!f.stok_awal" label="Dibayar dari">
          <select v-model="f.akun_kas_id" :class="kelasKontrol" data-uji="masuk-akun"><option v-for="a in akun" :key="a.id" :value="a.id">{{ a.nama }}</option></select>
        </Kolom>
        <div class="flex items-center justify-between border-t pt-3">
          <div>
            <div class="text-xs text-muted-foreground">{{ f.stok_awal ? "Nilai stok" : "Kas keluar" }}</div>
            <div class="text-lg font-semibold tabular-nums" data-uji="masuk-total">{{ rupiah(total) }}</div>
          </div>
          <Button type="submit" size="lg" :disabled="!lengkap || sibuk" data-uji="masuk-simpan">Simpan</Button>
        </div>
        <p v-if="galat" class="text-sm text-destructive" data-uji="masuk-galat">{{ galat }}</p>
      </form>

      <section class="min-w-0">
        <h2 class="mb-2 text-sm font-semibold">Barang masuk yang terakhir dicatat</h2>
        <div class="overflow-auto rounded-lg border">
          <table class="w-full text-sm">
            <thead class="bg-muted/60 text-left text-xs text-muted-foreground">
              <tr>
                <th class="px-3 py-2 font-medium">Tanggal</th>
                <th class="px-3 py-2 font-medium">Barang</th>
                <th class="px-3 py-2 text-right font-medium">Jumlah</th>
                <th class="px-3 py-2 text-right font-medium">Total</th>
                <th class="px-3 py-2" />
              </tr>
            </thead>
            <tbody>
              <tr v-if="!riwayat.length"><td colspan="5" class="px-3 py-6 text-center text-muted-foreground">Belum ada barang masuk.</td></tr>
              <tr v-for="k in riwayat" :key="k.id" class="border-t" :class="k.batal ? 'text-muted-foreground' : ''" :data-uji="`masuk-baris-${k.id}`">
                <td class="px-3 py-2 whitespace-nowrap" :class="k.batal ? 'line-through' : ''">{{ tanggal(k.tanggal_masuk) }}</td>
                <td class="px-3 py-2">
                  <span :class="k.batal ? 'line-through' : ''">{{ k.barang_nama }}</span>
                  <Badge v-if="k.stok_awal" variant="outline" class="ml-1">stok awal</Badge>
                  <div v-if="k.spek || k.pemasok" class="text-xs text-muted-foreground">{{ [k.spek, k.pemasok].filter(Boolean).join(" · ") }}</div>
                  <div v-if="k.batal" class="text-xs">Dibatalkan: {{ k.alasan_batal }}</div>
                </td>
                <td class="px-3 py-2 text-right tabular-nums whitespace-nowrap" :class="k.batal ? 'line-through' : ''">
                  {{ jumlah(k.jumlah_awal, k.satuan) }}
                  <div v-if="k.harga_beli !== null" class="text-xs text-muted-foreground">@ {{ rupiah(k.harga_beli) }}</div>
                </td>
                <td class="px-3 py-2 text-right tabular-nums whitespace-nowrap" :class="k.batal ? 'line-through' : ''">{{ k.harga_beli === null ? "—" : rupiah(nilai(k.jumlah_awal, k.harga_beli)) }}</td>
                <td class="px-3 py-2 text-right">
                  <Button v-if="bolehBatal && !k.batal && !k.pernah_keluar" variant="ghost" size="sm" class="text-destructive" :data-uji="`masuk-batal-${k.id}`" @click="batal = k">Batalkan</Button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>
    </div>

    <FormBarang v-model:buka="formBarang" :barang="null" @tersimpan="barangBaruTersimpan" />
    <DialogBatal :buka="batal !== null" judul="Batalkan barang masuk ini?" keterangan="Stok kiriman ini dihapus dan catatan kas pembeliannya ikut dibatalkan." :sibuk="sibuk" @update:buka="!$event && (batal = null)" @konfirmasi="batalkan" />
  </Halaman>
</template>
