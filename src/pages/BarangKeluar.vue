<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { toast } from "vue-sonner";
import { PlusIcon, Trash2Icon } from "@lucide/vue";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import DialogBatal from "@/components/DialogBatal.vue";
import Halaman from "@/components/Halaman.vue";
import InputJumlah from "@/components/InputJumlah.vue";
import InputRupiah from "@/components/InputRupiah.vue";
import InputTanggal from "@/components/InputTanggal.vue";
import PilihBarang from "@/components/PilihBarang.vue";
import PilihKiriman from "@/components/PilihKiriman.vue";
import { GalatApp, panggil, pesanGalat } from "@/lib/api";
import { hariIni, jumlah, nilai, rupiah, tanggal, tanggalPendek } from "@/lib/format";
import { kelasKontrol } from "@/lib/gaya";
import type { AkunKas, Alokasi, Barang, KeluarMasukan, Kiriman, TransaksiKeluar } from "@/lib/types";
import { useSesi } from "@/stores/sesi";

interface Baris {
  kunci: number;
  barang_id: number | null;
  jumlah: number | null;
  harga: number | null;
  alokasi: Alokasi[];
  /** Kiriman dipilih sendiri lewat "Ganti kiriman". */
  manual: boolean;
  kedaluwarsa: boolean;
  galat: string;
  kiriman: Kiriman[];
  /** Nomor permintaan usul terakhir; jawaban yang lebih lama diabaikan. */
  urut: number;
  /** Usul untuk isian terbaru belum kembali. */
  menunggu: boolean;
}

const sesi = useSesi();
const bolehTambah = computed(() => sesi.boleh("barang_keluar", 2));
const bolehBatal = computed(() => sesi.boleh("barang_keluar", 3));

const barang = ref<Barang[]>([]);
const akun = ref<AkunKas[]>([]);
const riwayat = ref<TransaksiKeluar[]>([]);
let kunciBerikut = 1;
const barisBaru = (): Baris => ({ kunci: kunciBerikut++, barang_id: null, jumlah: null, harga: null, alokasi: [], manual: false, kedaluwarsa: false, galat: "", kiriman: [], urut: 0, menunggu: false });
const baris = ref<Baris[]>([barisBaru()]);
const kepala = reactive({ tanggal: hariIni(), akun_kas_id: null as number | null, catatan: "" });
const sibuk = ref(false);
const galatSimpan = ref("");
const gantiUntuk = ref<Baris | null>(null);
const batalUntuk = ref<TransaksiKeluar | null>(null);

const namaBarang = (id: number | null) => barang.value.find((b) => b.id === id);
const terisi = computed(() => baris.value.filter((b) => b.barang_id !== null));
const subtotal = (b: Baris) => (b.harga === null ? 0 : b.alokasi.reduce((n, a) => n + nilai(a.jumlah, b.harga as number), 0));
const total = computed(() => terisi.value.reduce((n, b) => n + subtotal(b), 0));
const bisaSimpan = computed(
  () =>
    bolehTambah.value &&
    !sibuk.value &&
    terisi.value.length > 0 &&
    terisi.value.every((b) => !b.menunggu && b.alokasi.length > 0 && b.alokasi.reduce((n, a) => n + a.jumlah, 0) === b.jumlah && b.harga !== null && !b.galat) &&
    kepala.akun_kas_id !== null &&
    kepala.tanggal !== "",
);

function teksAlokasi(b: Baris): string {
  return b.alokasi
    .map((a) => {
      const k = b.kiriman.find((x) => x.id === a.kiriman_id);
      return `${jumlah(a.jumlah)} dari ${k ? tanggalPendek(k.tanggal_masuk) : `kiriman #${a.kiriman_id}`}`;
    })
    .join(" · ");
}

async function muat() {
  [barang.value, akun.value, riwayat.value] = await Promise.all([
    panggil<Barang[]>("barang_daftar", { cari: null, termasuk_nonaktif: false }),
    panggil<AkunKas[]>("akun_kas_daftar"),
    panggil<TransaksiKeluar[]>("keluar_terbaru", { batas: 30 }),
  ]);
  akun.value = akun.value.filter((a) => a.aktif);
  if (!akun.value.some((a) => a.id === kepala.akun_kas_id)) kepala.akun_kas_id = akun.value[0]?.id ?? null;
}

/** Yang sudah diambil baris lain untuk barang yang sama: baris di atasnya, dan yang dipilih sendiri. */
function terpakaiOleh(b: Baris): Alokasi[] {
  const i = baris.value.indexOf(b);
  return baris.value.filter((x, j) => x !== b && x.barang_id === b.barang_id && (j < i || x.manual)).flatMap((x) => x.alokasi);
}

/** Baris otomatis di bawah `b` untuk barang yang sama perlu dihitung ulang bila `b` berubah. */
function hitungUlangDiBawah(b: Baris, barangId: number | null) {
  if (barangId === null) return;
  const i = baris.value.indexOf(b);
  const berikut = baris.value.find((x, j) => (i < 0 || j > i) && x !== b && !x.manual && x.barang_id === barangId && x.jumlah !== null);
  if (berikut) void usul(berikut);
}

async function usul(b: Baris) {
  b.manual = false;
  b.kedaluwarsa = false;
  const urut = ++b.urut;
  if (b.barang_id === null || b.jumlah === null) {
    b.alokasi = [];
    b.galat = "";
    b.menunggu = false;
    hitungUlangDiBawah(b, b.barang_id);
    return;
  }
  b.menunggu = true;
  try {
    const hasil = await panggil<Alokasi[]>("usul_alokasi", { barang_id: b.barang_id, jumlah: b.jumlah, terpakai: terpakaiOleh(b) });
    if (urut !== b.urut) return;
    b.alokasi = hasil;
    b.galat = "";
  } catch (e) {
    if (urut !== b.urut) return;
    b.alokasi = [];
    b.galat = pesanGalat(e);
  }
  b.menunggu = false;
  hitungUlangDiBawah(b, b.barang_id);
}

async function pilihBarang(b: Baris, id: number | null) {
  const sebelumnya = b.barang_id;
  b.barang_id = id;
  b.alokasi = [];
  if (sebelumnya !== id) hitungUlangDiBawah(b, sebelumnya);
  b.kiriman = [];
  b.harga = namaBarang(id)?.harga_jual ?? null;
  if (id !== null) {
    try {
      const k = await panggil<Kiriman[]>("kiriman_barang", { barang_id: id, hanya_bersisa: true });
      if (b.barang_id === id) b.kiriman = k;
    } catch (e) {
      b.galat = pesanGalat(e);
    }
  }
  await usul(b);
}

function ubahJumlah(b: Baris, n: number | null) {
  b.jumlah = n;
  void usul(b);
}

function pakaiKiriman(alokasi: Alokasi[], adaKedaluwarsa: boolean) {
  const b = gantiUntuk.value;
  if (!b) return;
  b.urut++;
  b.alokasi = alokasi;
  b.jumlah = alokasi.reduce((n, a) => n + a.jumlah, 0);
  b.manual = true;
  b.kedaluwarsa = adaKedaluwarsa;
  b.galat = "";
  b.menunggu = false;
  hitungUlangDiBawah(b, b.barang_id);
}

function hapus(b: Baris) {
  baris.value = baris.value.filter((x) => x !== b);
  if (!baris.value.length) baris.value = [barisBaru()];
  // Yang tadinya di bawah baris ini bisa mengambil lagi kiriman yang dilepas.
  const pertama = baris.value.find((x) => !x.manual && x.barang_id === b.barang_id && x.jumlah !== null);
  if (pertama && b.barang_id !== null) void usul(pertama);
}

async function simpan() {
  galatSimpan.value = "";
  sibuk.value = true;
  const masukan: KeluarMasukan = {
    tanggal: kepala.tanggal,
    akun_kas_id: kepala.akun_kas_id as number,
    catatan: kepala.catatan.trim() || null,
    baris: terisi.value.map((b) => ({ barang_id: b.barang_id as number, harga_jual: b.harga as number, alokasi: b.alokasi })),
    izinkan_kedaluwarsa: terisi.value.some((b) => b.kedaluwarsa),
  };
  try {
    await panggil<number>("keluar_tambah", { masukan });
    toast.success(`Barang keluar tersimpan. Total ${rupiah(total.value)}.`);
    baris.value = [barisBaru()];
    kepala.catatan = "";
    await muat();
  } catch (e) {
    galatSimpan.value = pesanGalat(e);
    // Stok bisa berubah sejak diusulkan: hitung ulang baris yang otomatis.
    if (e instanceof GalatApp && e.kode === "STOK_KURANG") await Promise.all(terisi.value.filter((b) => !b.manual).map(usul));
  } finally {
    sibuk.value = false;
  }
}

async function batalkan(alasan: string) {
  if (!batalUntuk.value) return;
  sibuk.value = true;
  try {
    await panggil("keluar_batal", { id: batalUntuk.value.id, alasan });
    toast.success("Transaksi dibatalkan. Stok dan kas sudah dikembalikan.");
    batalUntuk.value = null;
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
  <Halaman judul="Barang Keluar" keterangan="Stok terlama diambil lebih dulu secara otomatis.">
    <div class="flex flex-col gap-6">
      <section v-if="bolehTambah" class="rounded-xl border bg-card p-4">
        <div class="hidden grid-cols-[minmax(0,2fr)_9rem_11rem_9rem_2rem] gap-3 px-1 pb-2 text-xs font-medium text-muted-foreground md:grid">
          <span>Barang</span><span class="text-right">Jumlah</span><span class="text-right">Harga jual / satuan</span><span class="text-right">Subtotal</span><span />
        </div>
        <div v-for="(b, i) in baris" :key="b.kunci" class="border-t py-3 first:border-t-0">
          <div class="grid grid-cols-1 items-start gap-3 md:grid-cols-[minmax(0,2fr)_9rem_11rem_9rem_2rem]">
            <PilihBarang :model-value="b.barang_id" :daftar="barang" :uji="i" @update:model-value="pilihBarang(b, $event)" />
            <InputJumlah :model-value="b.jumlah" :satuan="namaBarang(b.barang_id)?.satuan" :data-uji="`jumlah-${i}`" @update:model-value="ubahJumlah(b, $event)" />
            <InputRupiah v-model="b.harga" :data-uji="`harga-${i}`" />
            <div class="flex h-8 items-center justify-end text-sm font-medium tabular-nums" :data-uji="`subtotal-${i}`">{{ rupiah(subtotal(b)) }}</div>
            <Button variant="ghost" size="icon" title="Hapus baris" @click="hapus(b)"><Trash2Icon /></Button>
          </div>
          <div v-if="b.barang_id !== null" class="mt-1.5 flex flex-wrap items-center gap-x-3 gap-y-1 pl-1 text-xs">
            <span v-if="b.galat" class="text-destructive" :data-uji="`galat-${i}`">{{ b.galat }}</span>
            <template v-else-if="b.alokasi.length">
              <span class="text-muted-foreground">Diambil:</span>
              <span :data-uji="`alokasi-${i}`">{{ teksAlokasi(b) }}</span>
              <Badge v-if="b.manual" variant="outline">dipilih sendiri</Badge>
              <Badge v-if="b.kedaluwarsa" variant="destructive">ada yang kedaluwarsa</Badge>
            </template>
            <span v-else class="text-muted-foreground">Stok tersedia: {{ jumlah(namaBarang(b.barang_id)?.stok ?? 0, namaBarang(b.barang_id)?.satuan) }}</span>
            <button v-if="b.kiriman.length" type="button" class="text-primary underline-offset-4 hover:underline" :data-uji="`ganti-kiriman-${i}`" @click="gantiUntuk = b">Ganti kiriman</button>
          </div>
        </div>
        <Button variant="outline" size="sm" class="mt-1" data-uji="tambah-baris" @click="baris.push(barisBaru())"><PlusIcon /> Tambah barang lain</Button>

        <div class="mt-4 grid grid-cols-1 items-end gap-3 border-t pt-4 md:grid-cols-[10rem_12rem_minmax(0,1fr)_auto_auto]">
          <label class="flex flex-col gap-1 text-sm"><span class="font-medium">Tanggal</span><InputTanggal v-model="kepala.tanggal" data-uji="tanggal" /></label>
          <label class="flex flex-col gap-1 text-sm">
            <span class="font-medium">Uang masuk ke</span>
            <select v-model="kepala.akun_kas_id" :class="kelasKontrol" data-uji="akun">
              <option v-for="a in akun" :key="a.id" :value="a.id">{{ a.nama }}</option>
            </select>
          </label>
          <label class="flex flex-col gap-1 text-sm"><span class="font-medium">Catatan (opsional)</span><Input v-model="kepala.catatan" placeholder="Nama pembeli, dll." /></label>
          <div class="text-right">
            <div class="text-xs text-muted-foreground">Total</div>
            <div class="text-xl font-semibold tabular-nums" data-uji="total">{{ rupiah(total) }}</div>
          </div>
          <Button size="lg" :disabled="!bisaSimpan" data-uji="simpan" @click="simpan">Simpan</Button>
        </div>
        <p v-if="galatSimpan" class="mt-2 text-sm text-destructive" data-uji="galat-simpan">{{ galatSimpan }}</p>
      </section>

      <section>
        <h2 class="mb-2 text-sm font-semibold">Barang keluar yang terakhir dicatat</h2>
        <div class="overflow-auto rounded-lg border">
          <table class="w-full text-sm">
            <thead class="bg-muted/60 text-left text-xs text-muted-foreground">
              <tr>
                <th class="px-3 py-2 font-medium">Tanggal</th>
                <th class="px-3 py-2 font-medium">Barang</th>
                <th class="px-3 py-2 text-right font-medium">Total</th>
                <th v-if="sesi.bolehHargaBeli" class="px-3 py-2 text-right font-medium">Laba</th>
                <th class="px-3 py-2 font-medium">Dicatat oleh</th>
                <th class="px-3 py-2" />
              </tr>
            </thead>
            <tbody>
              <tr v-if="!riwayat.length"><td colspan="6" class="px-3 py-6 text-center text-muted-foreground">Belum ada barang keluar.</td></tr>
              <tr v-for="t in riwayat" :key="t.id" class="border-t align-top" :class="t.batal ? 'text-muted-foreground' : ''" :data-uji="`transaksi-${t.id}`">
                <td class="px-3 py-2 whitespace-nowrap" :class="t.batal ? 'line-through' : ''">{{ tanggal(t.tanggal) }}</td>
                <td class="px-3 py-2">
                  <div v-for="(r, j) in t.baris" :key="j" :class="t.batal ? 'line-through' : ''">
                    {{ jumlah(r.jumlah, r.satuan) }} {{ r.barang_nama }}
                    <span class="text-xs text-muted-foreground">@ {{ rupiah(r.harga_jual) }} · kiriman {{ tanggalPendek(r.tanggal_masuk) }}</span>
                  </div>
                  <div v-if="t.catatan" class="text-xs text-muted-foreground">{{ t.catatan }}</div>
                  <div v-if="t.batal" class="text-xs">Dibatalkan: {{ t.alasan_batal }}</div>
                </td>
                <td class="px-3 py-2 text-right tabular-nums whitespace-nowrap" :class="t.batal ? 'line-through' : ''">{{ rupiah(t.total) }}</td>
                <td v-if="sesi.bolehHargaBeli" class="px-3 py-2 text-right tabular-nums whitespace-nowrap" :class="t.batal ? 'line-through' : ''">{{ rupiah(t.laba) }}</td>
                <td class="px-3 py-2 whitespace-nowrap">{{ t.dibuat_oleh_nama ?? "—" }}</td>
                <td class="px-3 py-2 text-right">
                  <Button v-if="bolehBatal && !t.batal" variant="ghost" size="sm" class="text-destructive" :data-uji="`batal-${t.id}`" @click="batalUntuk = t">Batalkan</Button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>
    </div>

    <PilihKiriman
      :buka="gantiUntuk !== null"
      :nama="namaBarang(gantiUntuk?.barang_id ?? null)?.nama ?? ''"
      :satuan="namaBarang(gantiUntuk?.barang_id ?? null)?.satuan ?? ''"
      :kiriman="gantiUntuk?.kiriman ?? []"
      :awal="gantiUntuk?.alokasi ?? []"
      @update:buka="!$event && (gantiUntuk = null)"
      @pakai="pakaiKiriman"
    />
    <DialogBatal :buka="batalUntuk !== null" judul="Batalkan barang keluar ini?" keterangan="Stok dikembalikan ke kirimannya dan catatan kas penjualannya ikut dibatalkan." :sibuk="sibuk" @update:buka="!$event && (batalUntuk = null)" @konfirmasi="batalkan" />
  </Halaman>
</template>
