<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { toast } from "vue-sonner";
import { PlusIcon } from "@lucide/vue";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import Halaman from "@/components/Halaman.vue";
import Kolom from "@/components/Kolom.vue";
import { panggil, pesanGalat } from "@/lib/api";
import { kelasKontrol } from "@/lib/gaya";
import type { Modul, Pengguna, PenggunaMasukan, Tingkat } from "@/lib/types";
import { useSesi } from "@/stores/sesi";

const sesi = useSesi();
const MODUL: { kunci: Modul; nama: string; ket: string }[] = [
  { kunci: "barang", nama: "Barang", ket: "Daftar barang dan kategorinya" },
  { kunci: "barang_keluar", nama: "Barang Keluar", ket: "Mencatat penjualan / barang keluar" },
  { kunci: "barang_masuk", nama: "Barang Masuk", ket: "Mencatat kiriman, perbaikan, penyesuaian stok" },
  { kunci: "kas", nama: "Kas", ket: "Semua catatan uang, termasuk pembelian" },
  { kunci: "laporan", nama: "Laporan", ket: "Melihat dan mengunduh laporan" },
];
const TINGKAT = ["Tidak ada", "Lihat", "Tambah", "Ubah & batalkan"];
/** Modul yang memuat harga beli: hak "harga beli dan laba" ikut wajib. */
const BUTUH_HARGA: Modul[] = ["barang_masuk", "laporan"];

const daftar = ref<Pengguna[]>([]);
const form = ref<{ id: number | null; m: PenggunaMasukan; sandi: string } | null>(null);
const sandiUntuk = ref<Pengguna | null>(null);
const sandiBaru = ref("");
const galat = ref("");
const sibuk = ref(false);
let bawaan: Partial<Record<Modul, Tingkat>> = {};

const wajibHarga = computed(() => !!form.value && BUTUH_HARGA.some((k) => (form.value!.m.hak[k] ?? 0) > 0));
const lengkap = computed(() => !!form.value && form.value.m.nama.trim() !== "" && form.value.m.nama_pengguna.trim() !== "" && (form.value.id !== null || form.value.sandi.length >= 8));

async function muat() {
  try {
    daftar.value = await panggil<Pengguna[]>("pengguna_daftar");
    bawaan = await panggil("hak_bawaan_staf");
  } catch (e) {
    toast.error(pesanGalat(e));
  }
}
onMounted(muat);

function tambah() {
  galat.value = "";
  form.value = { id: null, sandi: "", m: { nama_pengguna: "", nama: "", peran: "staf", lihat_harga_beli: false, aktif: true, hak: { ...bawaan } } };
}
function ubah(p: Pengguna) {
  galat.value = "";
  form.value = { id: p.id, sandi: "", m: { nama_pengguna: p.nama_pengguna, nama: p.nama, peran: p.peran, lihat_harga_beli: p.lihat_harga_beli, aktif: p.aktif, hak: { ...p.hak } } };
}
function aturHak(k: Modul, t: number) {
  if (!form.value) return;
  form.value.m.hak[k] = t as Tingkat;
  if (wajibHarga.value) form.value.m.lihat_harga_beli = true;
}
function pakaiBawaan() {
  if (!form.value) return;
  form.value.m.hak = { ...bawaan };
  form.value.m.lihat_harga_beli = false;
}

async function simpan() {
  if (!form.value) return;
  galat.value = "";
  sibuk.value = true;
  try {
    const masukan = { ...form.value.m, hak: Object.fromEntries(MODUL.map((m) => [m.kunci, form.value!.m.hak[m.kunci] ?? 0])) };
    if (form.value.id === null) await panggil("pengguna_tambah", { masukan, sandi: form.value.sandi });
    else await panggil("pengguna_ubah", { id: form.value.id, masukan });
    toast.success(form.value.id === null ? "Akun baru dibuat." : "Perubahan akun tersimpan. Berlaku saat pengguna itu masuk lagi.");
    form.value = null;
    await muat();
  } catch (e) {
    galat.value = pesanGalat(e);
  } finally {
    sibuk.value = false;
  }
}

async function aturSandi() {
  if (!sandiUntuk.value) return;
  galat.value = "";
  sibuk.value = true;
  try {
    await panggil("pengguna_atur_ulang_sandi", { id: sandiUntuk.value.id, sandi_baru: sandiBaru.value });
    toast.success(`Kata sandi ${sandiUntuk.value.nama} sudah diganti.`);
    sandiUntuk.value = null;
  } catch (e) {
    galat.value = pesanGalat(e);
  } finally {
    sibuk.value = false;
  }
}

function ringkasHak(p: Pengguna): string {
  if (p.peran === "pemilik") return "Akses penuh";
  const isi = MODUL.filter((m) => (p.hak[m.kunci] ?? 0) > 0).map((m) => `${m.nama}: ${TINGKAT[p.hak[m.kunci] ?? 0].toLowerCase()}`);
  if (p.lihat_harga_beli) isi.push("lihat harga beli");
  return isi.join(" · ") || "Belum punya hak apa pun";
}
</script>

<template>
  <Halaman judul="Pengguna" keterangan="Akun untuk pemilik dan staf, beserta apa saja yang boleh mereka lakukan.">
    <template #aksi><Button data-uji="pengguna-baru" @click="tambah"><PlusIcon /> Akun baru</Button></template>

    <div class="overflow-auto rounded-lg border">
      <table class="w-full text-sm">
        <thead class="bg-muted/60 text-left text-xs text-muted-foreground">
          <tr><th class="px-3 py-2 font-medium">Nama</th><th class="px-3 py-2 font-medium">Nama pengguna</th><th class="px-3 py-2 font-medium">Hak akses</th><th class="px-3 py-2" /></tr>
        </thead>
        <tbody>
          <tr v-for="p in daftar" :key="p.id" class="border-t" :class="p.aktif ? '' : 'text-muted-foreground'" :data-uji="`pengguna-${p.nama_pengguna}`">
            <td class="px-3 py-2 whitespace-nowrap">
              {{ p.nama }}
              <Badge v-if="p.peran === 'pemilik'" class="ml-1">Pemilik</Badge>
              <Badge v-if="!p.aktif" variant="outline" class="ml-1">nonaktif</Badge>
              <Badge v-if="p.id === sesi.sesi?.pengguna_id" variant="outline" class="ml-1">Anda</Badge>
            </td>
            <td class="px-3 py-2">{{ p.nama_pengguna }}</td>
            <td class="px-3 py-2 text-xs text-muted-foreground">{{ ringkasHak(p) }}</td>
            <td class="px-3 py-2 text-right whitespace-nowrap">
              <Button variant="ghost" size="sm" :data-uji="`pengguna-ubah-${p.nama_pengguna}`" @click="ubah(p)">Ubah</Button>
              <Button variant="ghost" size="sm" :data-uji="`pengguna-sandi-${p.nama_pengguna}`" @click="((sandiUntuk = p), (sandiBaru = ''), (galat = ''))">Atur ulang sandi</Button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <Dialog :open="form !== null" @update:open="!$event && (form = null)">
      <DialogContent v-if="form" class="sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>{{ form.id === null ? "Akun baru" : `Ubah akun ${form.m.nama}` }}</DialogTitle>
          <DialogDescription>Pemilik selalu punya akses penuh. Untuk staf, pilih apa saja yang boleh dilakukan.</DialogDescription>
        </DialogHeader>
        <form class="grid grid-cols-2 gap-3" @submit.prevent="simpan">
          <Kolom label="Nama"><Input v-model="form.m.nama" data-uji="p-nama" /></Kolom>
          <Kolom label="Nama pengguna" bantuan="Dipakai saat masuk"><Input v-model="form.m.nama_pengguna" autocapitalize="none" data-uji="p-nama-pengguna" /></Kolom>
          <Kolom v-if="form.id === null" label="Kata sandi" bantuan="Minimal 8 karakter"><Input v-model="form.sandi" type="password" data-uji="p-sandi" /></Kolom>
          <Kolom label="Peran">
            <select v-model="form.m.peran" :class="kelasKontrol" data-uji="p-peran"><option value="staf">Staf</option><option value="pemilik">Pemilik (akses penuh)</option></select>
          </Kolom>

          <div v-if="form.m.peran === 'staf'" class="col-span-2 rounded-lg border">
            <div class="flex items-center justify-between border-b px-3 py-2">
              <span class="text-sm font-medium">Hak akses</span>
              <Button type="button" variant="outline" size="sm" data-uji="p-bawaan" @click="pakaiBawaan">Pakai bawaan kasir</Button>
            </div>
            <div v-for="m in MODUL" :key="m.kunci" class="flex items-center gap-3 border-b px-3 py-2 last:border-b-0">
              <div class="min-w-0 flex-1">
                <div class="text-sm font-medium">{{ m.nama }}</div>
                <div class="text-xs text-muted-foreground">{{ m.ket }}</div>
              </div>
              <div class="w-44">
                <select :value="form.m.hak[m.kunci] ?? 0" :class="kelasKontrol" :data-uji="`p-hak-${m.kunci}`" @change="aturHak(m.kunci, Number(($event.target as HTMLSelectElement).value))">
                  <option v-for="(t, i) in TINGKAT" :key="i" :value="i">{{ t }}</option>
                </select>
              </div>
            </div>
            <label class="flex items-start gap-2 border-t px-3 py-2.5 text-sm">
              <input v-model="form.m.lihat_harga_beli" type="checkbox" class="mt-0.5 size-4 accent-primary" :disabled="wajibHarga" data-uji="p-harga-beli" />
              <span>
                Lihat harga beli dan laba
                <span v-if="wajibHarga" class="block text-xs text-muted-foreground">Wajib dicentang karena Barang Masuk dan Laporan memuat harga beli.</span>
              </span>
            </label>
          </div>

          <label v-if="form.id !== null" class="col-span-2 flex items-center gap-2 text-sm">
            <input v-model="form.m.aktif" type="checkbox" class="size-4 accent-primary" data-uji="p-aktif" /> Akun aktif (hilangkan centang agar tidak bisa masuk lagi)
          </label>
          <p v-if="galat" class="col-span-2 text-sm text-destructive" data-uji="p-galat">{{ galat }}</p>
          <DialogFooter class="col-span-2">
            <Button type="button" variant="outline" @click="form = null">Tidak jadi</Button>
            <Button type="submit" :disabled="!lengkap || sibuk" data-uji="p-simpan">Simpan</Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>

    <Dialog :open="sandiUntuk !== null" @update:open="!$event && (sandiUntuk = null)">
      <DialogContent v-if="sandiUntuk" class="sm:max-w-sm">
        <DialogHeader>
          <DialogTitle>Atur ulang kata sandi</DialogTitle>
          <DialogDescription>Untuk {{ sandiUntuk.nama }} ({{ sandiUntuk.nama_pengguna }}). Beri tahu kata sandi barunya secara langsung.</DialogDescription>
        </DialogHeader>
        <form class="flex flex-col gap-3" @submit.prevent="aturSandi">
          <Kolom label="Kata sandi baru" bantuan="Minimal 8 karakter"><Input v-model="sandiBaru" type="password" data-uji="p-sandi-baru" /></Kolom>
          <p v-if="galat" class="text-sm text-destructive">{{ galat }}</p>
          <DialogFooter>
            <Button type="button" variant="outline" @click="sandiUntuk = null">Tidak jadi</Button>
            <Button type="submit" :disabled="sandiBaru.length < 8 || sibuk" data-uji="p-sandi-simpan">Ganti kata sandi</Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  </Halaman>
</template>
