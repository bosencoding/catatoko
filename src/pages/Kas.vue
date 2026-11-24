<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from "vue";
import { toast } from "vue-sonner";
import { ArrowLeftRightIcon, MinusIcon, PlusIcon } from "@lucide/vue";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import DialogBatal from "@/components/DialogBatal.vue";
import Halaman from "@/components/Halaman.vue";
import InputRupiah from "@/components/InputRupiah.vue";
import InputTanggal from "@/components/InputTanggal.vue";
import Kolom from "@/components/Kolom.vue";
import PilihPeriode, { bulanIni } from "@/components/PilihPeriode.vue";
import { panggil, pesanGalat } from "@/lib/api";
import { hariIni, rupiah, tanggal } from "@/lib/format";
import { kelasKontrol } from "@/lib/gaya";
import type { AkunKas, BarisKas, KasMasukan, KategoriKas, Saldo, SaringKas } from "@/lib/types";
import { useSesi } from "@/stores/sesi";

const sesi = useSesi();
const bolehTambah = computed(() => sesi.boleh("kas", 2));
const bolehUbah = computed(() => sesi.boleh("kas", 3));

const saldo = ref<Saldo | null>(null);
const baris = ref<BarisKas[]>([]);
const akun = ref<AkunKas[]>([]);
const kategori = ref<KategoriKas[]>([]);
const periode = ref(bulanIni());
const saring = reactive({ akun_kas_id: null as number | null, kategori_id: null as number | null, arah: null as "masuk" | "keluar" | null });
const sibuk = ref(false);
const galat = ref("");

const akunAktif = computed(() => akun.value.filter((a) => a.aktif));
const kategoriManual = computed(() => kategori.value.filter((k) => !k.bawaan));
const totalMasuk = computed(() => baris.value.filter((b) => !b.batal && b.arah === "masuk" && b.ref_jenis !== "pindah").reduce((n, b) => n + b.jumlah, 0));
const totalKeluar = computed(() => baris.value.filter((b) => !b.batal && b.arah === "keluar" && b.ref_jenis !== "pindah").reduce((n, b) => n + b.jumlah, 0));

async function muat() {
  try {
    const s: SaringKas = { ...periode.value, ...saring };
    [saldo.value, baris.value, akun.value, kategori.value] = await Promise.all([
      panggil<Saldo>("kas_saldo"),
      panggil<BarisKas[]>("kas_daftar", { saring: s }),
      panggil<AkunKas[]>("akun_kas_daftar"),
      panggil<KategoriKas[]>("kategori_kas_daftar"),
    ]);
  } catch (e) {
    toast.error(pesanGalat(e));
  }
}
watch([periode, saring], muat, { deep: true });
onMounted(muat);

// --- Tambah / ubah catatan manual
const form = ref<{ id: number | null; f: Omit<KasMasukan, "jumlah" | "keterangan" | "akun_kas_id" | "kategori_id"> & { jumlah: number | null; keterangan: string; akun_kas_id: number | null; kategori_id: number | null } } | null>(null);

function bukaForm(arah: "masuk" | "keluar", lama?: BarisKas) {
  galat.value = "";
  form.value = {
    id: lama?.id ?? null,
    f: lama
      ? { tanggal: lama.tanggal, arah: lama.arah, akun_kas_id: lama.akun_kas_id, kategori_id: lama.kategori_id, jumlah: lama.jumlah, keterangan: lama.keterangan ?? "" }
      : { tanggal: hariIni(), arah, akun_kas_id: akunAktif.value[0]?.id ?? null, kategori_id: null, jumlah: null, keterangan: "" },
  };
}
const formLengkap = computed(() => !!form.value && form.value.f.tanggal !== "" && form.value.f.akun_kas_id !== null && form.value.f.kategori_id !== null && form.value.f.jumlah !== null && form.value.f.jumlah > 0);

async function simpanForm() {
  if (!form.value) return;
  galat.value = "";
  sibuk.value = true;
  const masukan = { ...form.value.f, keterangan: form.value.f.keterangan.trim() || null };
  try {
    if (form.value.id === null) await panggil("kas_tambah", { masukan });
    else await panggil("kas_ubah", { id: form.value.id, masukan });
    toast.success("Catatan kas tersimpan.");
    form.value = null;
    await muat();
  } catch (e) {
    galat.value = pesanGalat(e);
  } finally {
    sibuk.value = false;
  }
}

// --- Pindah antar akun
const pindah = ref<{ tanggal: string; dari_akun: number | null; ke_akun: number | null; jumlah: number | null; keterangan: string } | null>(null);
function bukaPindah() {
  galat.value = "";
  pindah.value = { tanggal: hariIni(), dari_akun: akunAktif.value[0]?.id ?? null, ke_akun: akunAktif.value[1]?.id ?? null, jumlah: null, keterangan: "" };
}
const pindahLengkap = computed(() => !!pindah.value && pindah.value.tanggal !== "" && pindah.value.dari_akun !== null && pindah.value.ke_akun !== null && pindah.value.dari_akun !== pindah.value.ke_akun && !!pindah.value.jumlah);
async function simpanPindah() {
  if (!pindah.value) return;
  galat.value = "";
  sibuk.value = true;
  try {
    await panggil("kas_pindah", { ...pindah.value, keterangan: pindah.value.keterangan.trim() || null });
    toast.success("Pindah akun tersimpan.");
    pindah.value = null;
    await muat();
  } catch (e) {
    galat.value = pesanGalat(e);
  } finally {
    sibuk.value = false;
  }
}

// --- Batal
const batal = ref<BarisKas | null>(null);
async function batalkan(alasan: string) {
  if (!batal.value) return;
  sibuk.value = true;
  try {
    await panggil("kas_batal", { id: batal.value.id, alasan });
    toast.success("Catatan kas dibatalkan.");
    batal.value = null;
    await muat();
  } catch (e) {
    toast.error(pesanGalat(e));
  } finally {
    sibuk.value = false;
  }
}
</script>

<template>
  <Halaman judul="Kas" keterangan="Penjualan dan pembelian barang tercatat otomatis. Yang lain dicatat di sini.">
    <template v-if="bolehTambah" #aksi>
      <Button data-uji="kas-masuk" @click="bukaForm('masuk')"><PlusIcon /> Kas masuk</Button>
      <Button variant="outline" data-uji="kas-keluar" @click="bukaForm('keluar')"><MinusIcon /> Kas keluar</Button>
      <Button v-if="akunAktif.length > 1" variant="outline" data-uji="kas-pindah" @click="bukaPindah"><ArrowLeftRightIcon /> Pindah akun</Button>
    </template>

    <div class="flex flex-col gap-5">
      <div v-if="saldo" class="grid grid-cols-[repeat(auto-fill,minmax(12rem,1fr))] gap-3">
        <div class="rounded-xl border bg-primary p-4 text-primary-foreground">
          <div class="text-xs opacity-80">Total saldo</div>
          <div class="text-xl font-semibold tabular-nums" data-uji="saldo-total">{{ rupiah(saldo.total) }}</div>
        </div>
        <div v-for="a in saldo.per_akun.filter((x) => x.aktif || x.saldo !== 0)" :key="a.akun_kas_id" class="rounded-xl border bg-card p-4">
          <div class="text-xs text-muted-foreground">{{ a.nama }}</div>
          <div class="text-xl font-semibold tabular-nums" :data-uji="`saldo-akun-${a.akun_kas_id}`">{{ rupiah(a.saldo) }}</div>
        </div>
      </div>

      <div class="flex flex-wrap items-center gap-2">
        <PilihPeriode v-model="periode" />
        <div class="w-40">
          <select v-model="saring.akun_kas_id" :class="kelasKontrol"><option :value="null">Semua akun</option><option v-for="a in akun" :key="a.id" :value="a.id">{{ a.nama }}</option></select>
        </div>
        <div class="w-44">
          <select v-model="saring.kategori_id" :class="kelasKontrol"><option :value="null">Semua kategori</option><option v-for="k in kategori" :key="k.id" :value="k.id">{{ k.nama }}</option></select>
        </div>
        <div class="w-44">
          <select v-model="saring.arah" :class="kelasKontrol"><option :value="null">Masuk &amp; keluar</option><option value="masuk">Masuk saja</option><option value="keluar">Keluar saja</option></select>
        </div>
        <span class="ml-auto text-sm text-muted-foreground">Masuk <b class="text-emerald-600 tabular-nums dark:text-emerald-400">{{ rupiah(totalMasuk) }}</b> · Keluar <b class="text-rose-600 tabular-nums dark:text-rose-400">{{ rupiah(totalKeluar) }}</b></span>
      </div>

      <div class="overflow-auto rounded-lg border">
        <table class="w-full text-sm">
          <thead class="bg-muted/60 text-left text-xs text-muted-foreground">
            <tr>
              <th class="px-3 py-2 font-medium">Tanggal</th>
              <th class="px-3 py-2 font-medium">Kategori</th>
              <th class="px-3 py-2 font-medium">Keterangan</th>
              <th class="px-3 py-2 text-right font-medium">Masuk</th>
              <th class="px-3 py-2 text-right font-medium">Keluar</th>
              <th class="px-3 py-2 text-right font-medium">{{ saring.akun_kas_id ? "Saldo akun" : "Saldo" }}</th>
              <th class="px-3 py-2" />
            </tr>
          </thead>
          <tbody>
            <tr v-if="!baris.length"><td colspan="7" class="px-3 py-8 text-center text-muted-foreground" data-uji="kas-kosong">Belum ada catatan kas pada periode ini.</td></tr>
            <tr v-for="b in baris" :key="b.id" class="border-t align-top" :class="b.batal ? 'text-muted-foreground' : ''" :data-uji="`kas-${b.id}`">
              <td class="px-3 py-2 whitespace-nowrap" :class="b.batal ? 'line-through' : ''">{{ tanggal(b.tanggal) }}</td>
              <td class="px-3 py-2">
                <span :class="b.batal ? 'line-through' : ''">{{ b.kategori }}</span>
                <div class="text-xs text-muted-foreground">{{ b.akun }}<Badge v-if="b.sumber === 'otomatis'" variant="outline" class="ml-1">otomatis</Badge></div>
              </td>
              <td class="px-3 py-2">
                <span :class="b.batal ? 'line-through' : ''">{{ b.keterangan ?? "—" }}</span>
                <div v-if="b.batal" class="text-xs">Dibatalkan: {{ b.alasan_batal }}</div>
              </td>
              <td class="px-3 py-2 text-right tabular-nums whitespace-nowrap text-emerald-600 dark:text-emerald-400" :class="b.batal ? 'line-through opacity-60' : ''">{{ b.arah === "masuk" ? rupiah(b.jumlah) : "" }}</td>
              <td class="px-3 py-2 text-right tabular-nums whitespace-nowrap text-rose-600 dark:text-rose-400" :class="b.batal ? 'line-through opacity-60' : ''">{{ b.arah === "keluar" ? rupiah(b.jumlah) : "" }}</td>
              <td class="px-3 py-2 text-right tabular-nums whitespace-nowrap">{{ rupiah(b.saldo) }}</td>
              <td class="px-3 py-2 text-right whitespace-nowrap">
                <template v-if="bolehUbah && !b.batal && b.sumber === 'manual'">
                  <Button v-if="b.ref_jenis !== 'pindah'" variant="ghost" size="sm" :data-uji="`kas-ubah-${b.id}`" @click="bukaForm(b.arah, b)">Ubah</Button>
                  <Button variant="ghost" size="sm" class="text-destructive" :data-uji="`kas-batal-${b.id}`" @click="batal = b">Batalkan</Button>
                </template>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <Dialog :open="form !== null" @update:open="!$event && (form = null)">
      <DialogContent v-if="form" class="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{{ form.id === null ? "" : "Ubah " }}{{ form.f.arah === "masuk" ? "Kas masuk" : "Kas keluar" }}</DialogTitle>
          <DialogDescription>Untuk uang di luar jual-beli barang: modal, listrik, gaji, dan lainnya.</DialogDescription>
        </DialogHeader>
        <form class="grid grid-cols-2 gap-3" @submit.prevent="simpanForm">
          <Kolom label="Jumlah" class="col-span-2"><InputRupiah v-model="form.f.jumlah" wajib data-uji="kas-jumlah" /></Kolom>
          <Kolom label="Kategori">
            <select v-model="form.f.kategori_id" :class="kelasKontrol" data-uji="kas-kategori">
              <option :value="null" disabled>Pilih kategori</option>
              <option v-for="k in kategoriManual" :key="k.id" :value="k.id">{{ k.nama }}</option>
            </select>
          </Kolom>
          <Kolom label="Akun">
            <select v-model="form.f.akun_kas_id" :class="kelasKontrol" data-uji="kas-akun"><option v-for="a in akunAktif" :key="a.id" :value="a.id">{{ a.nama }}</option></select>
          </Kolom>
          <Kolom label="Tanggal" class="col-span-2"><InputTanggal v-model="form.f.tanggal" data-uji="kas-tanggal" /></Kolom>
          <Kolom label="Keterangan (opsional)" class="col-span-2"><Input v-model="form.f.keterangan" data-uji="kas-keterangan" /></Kolom>
          <p v-if="galat" class="col-span-2 text-sm text-destructive" data-uji="kas-galat">{{ galat }}</p>
          <DialogFooter class="col-span-2">
            <Button type="button" variant="outline" @click="form = null">Tidak jadi</Button>
            <Button type="submit" :disabled="!formLengkap || sibuk" data-uji="kas-simpan">Simpan</Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>

    <Dialog :open="pindah !== null" @update:open="!$event && (pindah = null)">
      <DialogContent v-if="pindah" class="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Pindah antar akun</DialogTitle>
          <DialogDescription>Misalnya setor uang tunai ke bank. Tidak dihitung sebagai pemasukan atau pengeluaran.</DialogDescription>
        </DialogHeader>
        <form class="grid grid-cols-2 gap-3" @submit.prevent="simpanPindah">
          <Kolom label="Dari"><select v-model="pindah.dari_akun" :class="kelasKontrol" data-uji="pindah-dari"><option v-for="a in akunAktif" :key="a.id" :value="a.id">{{ a.nama }}</option></select></Kolom>
          <Kolom label="Ke"><select v-model="pindah.ke_akun" :class="kelasKontrol" data-uji="pindah-ke"><option v-for="a in akunAktif" :key="a.id" :value="a.id">{{ a.nama }}</option></select></Kolom>
          <Kolom label="Jumlah"><InputRupiah v-model="pindah.jumlah" wajib data-uji="pindah-jumlah" /></Kolom>
          <Kolom label="Tanggal"><InputTanggal v-model="pindah.tanggal" /></Kolom>
          <Kolom label="Keterangan (opsional)" class="col-span-2"><Input v-model="pindah.keterangan" /></Kolom>
          <p v-if="galat" class="col-span-2 text-sm text-destructive" data-uji="pindah-galat">{{ galat }}</p>
          <DialogFooter class="col-span-2">
            <Button type="button" variant="outline" @click="pindah = null">Tidak jadi</Button>
            <Button type="submit" :disabled="!pindahLengkap || sibuk" data-uji="pindah-simpan">Pindahkan</Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>

    <DialogBatal :buka="batal !== null" judul="Batalkan catatan kas ini?" :sibuk="sibuk" @update:buka="!$event && (batal = null)" @konfirmasi="batalkan" />
  </Halaman>
</template>
