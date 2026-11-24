<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { open, save } from "@tauri-apps/plugin-dialog";
import { toast } from "vue-sonner";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import Halaman from "@/components/Halaman.vue";
import Kolom from "@/components/Kolom.vue";
import PilihPeriode, { bulanIni } from "@/components/PilihPeriode.vue";
import { panggil, pesanGalat } from "@/lib/api";
import { hariIni, waktu } from "@/lib/format";
import { muatNamaToko } from "@/lib/toko";
import type { AkunKas, BarisLog, KategoriKas, Pengaturan } from "@/lib/types";
import { useSesi } from "@/stores/sesi";

const sesi = useSesi();
const router = useRouter();
const sibuk = ref(false);

// --- Kata sandi sendiri (semua pengguna)
const sandi = ref({ lama: "", baru: "", ulangi: "" });
const galatSandi = ref("");
async function gantiSandi() {
  galatSandi.value = "";
  sibuk.value = true;
  try {
    await panggil("ganti_sandi_sendiri", { lama: sandi.value.lama, baru: sandi.value.baru });
    toast.success("Kata sandi Anda sudah diganti.");
    sandi.value = { lama: "", baru: "", ulangi: "" };
  } catch (e) {
    galatSandi.value = pesanGalat(e);
  } finally {
    sibuk.value = false;
  }
}

// --- Khusus pemilik
const toko = ref<Pengaturan>({ nama_toko: "", kunci_otomatis_menit: 10 });
const akun = ref<AkunKas[]>([]);
const kategori = ref<KategoriKas[]>([]);
const akunBaru = ref("");
const kategoriBaru = ref("");
const log = ref<BarisLog[]>([]);
const periodeLog = ref({ dari: hariIni(new Date(Date.now() - 6 * 86_400_000)), sampai: hariIni() });
const pulih = ref<string | null>(null);

async function muat() {
  if (!sesi.pemilik) return;
  try {
    [toko.value, akun.value, kategori.value] = await Promise.all([panggil<Pengaturan>("pengaturan_ambil"), panggil<AkunKas[]>("akun_kas_daftar"), panggil<KategoriKas[]>("kategori_kas_daftar")]);
  } catch (e) {
    toast.error(pesanGalat(e));
  }
}
async function muatLog() {
  try {
    log.value = await panggil<BarisLog[]>("log_daftar", periodeLog.value);
  } catch (e) {
    toast.error(pesanGalat(e));
  }
}
onMounted(async () => {
  await muat();
  if (sesi.pemilik) await muatLog();
});

async function jalankan(kerja: () => Promise<unknown>, pesan: string) {
  sibuk.value = true;
  try {
    await kerja();
    toast.success(pesan);
    await muat();
  } catch (e) {
    toast.error(pesanGalat(e));
  } finally {
    sibuk.value = false;
  }
}
const simpanToko = () =>
  jalankan(async () => {
    await panggil("pengaturan_simpan", { kunci: "nama_toko", nilai: toko.value.nama_toko });
    await panggil("pengaturan_simpan", { kunci: "kunci_otomatis_menit", nilai: String(toko.value.kunci_otomatis_menit) });
    await muatNamaToko();
  }, "Pengaturan toko tersimpan.");
const tambahAkun = () => jalankan(async () => (await panggil("akun_kas_tambah", { nama: akunBaru.value }), (akunBaru.value = "")), "Akun kas ditambahkan.");
const ubahAkun = (a: AkunKas, aktif: boolean) => jalankan(() => panggil("akun_kas_ubah", { id: a.id, nama: a.nama, aktif }), aktif ? "Akun kas diaktifkan lagi." : "Akun kas dinonaktifkan.");
const tambahKategori = () => jalankan(async () => (await panggil("kategori_kas_tambah", { nama: kategoriBaru.value }), (kategoriBaru.value = "")), "Kategori kas ditambahkan.");

async function cadangkan(pilihLokasi: boolean) {
  let tujuan: string | null = null;
  if (pilihLokasi) {
    tujuan = await save({ defaultPath: `catat-${hariIni().replace(/-/g, "")}.zip`, filters: [{ name: "Cadangan Catat", extensions: ["zip"] }] });
    if (!tujuan) return;
  }
  sibuk.value = true;
  try {
    const jalur = await panggil<string>("cadangan_buat", { tujuan });
    toast.success(`Cadangan tersimpan di ${jalur}`, { duration: 10_000 });
  } catch (e) {
    toast.error(pesanGalat(e));
  } finally {
    sibuk.value = false;
  }
}
async function pilihPulih() {
  const jalur = await open({ multiple: false, directory: false, filters: [{ name: "Cadangan Catat", extensions: ["zip"] }] });
  if (typeof jalur === "string") pulih.value = jalur;
}
async function pulihkan() {
  if (!pulih.value) return;
  sibuk.value = true;
  try {
    await panggil("cadangan_pulihkan", { sumber: pulih.value });
    pulih.value = null;
    sesi.sesi = null;
    sesi.pesanKunci = "Data sudah dipulihkan dari cadangan. Silakan masuk lagi.";
    router.replace("/login");
  } catch (e) {
    toast.error(pesanGalat(e), { duration: 10_000 });
  } finally {
    sibuk.value = false;
  }
}
</script>

<template>
  <Halaman judul="Pengaturan">
    <div class="grid max-w-5xl grid-cols-1 gap-5 lg:grid-cols-2">
      <section class="h-fit rounded-xl border bg-card p-4">
        <h2 class="text-sm font-semibold">Kata sandi saya</h2>
        <form class="mt-3 flex flex-col gap-3" @submit.prevent="gantiSandi">
          <Kolom label="Kata sandi sekarang"><Input v-model="sandi.lama" type="password" data-uji="sandi-lama" /></Kolom>
          <Kolom label="Kata sandi baru" bantuan="Minimal 8 karakter"><Input v-model="sandi.baru" type="password" data-uji="sandi-baru" /></Kolom>
          <Kolom label="Ulangi kata sandi baru" :galat="sandi.ulangi && sandi.ulangi !== sandi.baru ? 'Kata sandi tidak sama.' : undefined"><Input v-model="sandi.ulangi" type="password" data-uji="sandi-ulangi" /></Kolom>
          <p v-if="galatSandi" class="text-sm text-destructive" data-uji="sandi-galat">{{ galatSandi }}</p>
          <Button type="submit" class="self-start" :disabled="!sandi.lama || sandi.baru.length < 8 || sandi.baru !== sandi.ulangi || sibuk" data-uji="sandi-simpan">Ganti kata sandi</Button>
        </form>
      </section>

      <template v-if="sesi.pemilik">
        <section class="h-fit rounded-xl border bg-card p-4">
          <h2 class="text-sm font-semibold">Toko</h2>
          <form class="mt-3 flex flex-col gap-3" @submit.prevent="simpanToko">
            <Kolom label="Nama toko" bantuan="Tampil di layar dan di kepala laporan"><Input v-model="toko.nama_toko" data-uji="toko-nama" /></Kolom>
            <Kolom label="Kunci otomatis setelah (menit)" bantuan="Aplikasi minta masuk lagi bila tidak dipakai selama ini. 1 sampai 120.">
              <Input v-model.number="toko.kunci_otomatis_menit" type="number" min="1" max="120" class="w-28" data-uji="toko-menit" />
            </Kolom>
            <Button type="submit" class="self-start" :disabled="sibuk || !toko.nama_toko.trim()" data-uji="toko-simpan">Simpan</Button>
          </form>
        </section>

        <section class="h-fit rounded-xl border bg-card p-4">
          <h2 class="text-sm font-semibold">Akun kas</h2>
          <p class="text-xs text-muted-foreground">Tempat uang disimpan, misalnya Tunai dan Bank.</p>
          <ul class="mt-3 divide-y text-sm">
            <li v-for="a in akun" :key="a.id" class="flex items-center gap-2 py-1.5">
              <span class="flex-1" :class="a.aktif ? '' : 'text-muted-foreground line-through'">{{ a.nama }}</span>
              <Button variant="ghost" size="sm" :disabled="sibuk" @click="ubahAkun(a, !a.aktif)">{{ a.aktif ? "Nonaktifkan" : "Aktifkan" }}</Button>
            </li>
          </ul>
          <form class="mt-2 flex gap-2" @submit.prevent="tambahAkun">
            <Input v-model="akunBaru" placeholder="Nama akun baru" data-uji="akun-baru" />
            <Button type="submit" variant="outline" :disabled="!akunBaru.trim() || sibuk" data-uji="akun-tambah">Tambah</Button>
          </form>
        </section>

        <section class="h-fit rounded-xl border bg-card p-4">
          <h2 class="text-sm font-semibold">Kategori kas</h2>
          <div class="mt-3 flex flex-wrap gap-1.5">
            <Badge v-for="k in kategori" :key="k.id" :variant="k.bawaan ? 'secondary' : 'outline'" :title="k.bawaan ? 'Diisi otomatis oleh aplikasi' : ''">{{ k.nama }}</Badge>
          </div>
          <form class="mt-3 flex gap-2" @submit.prevent="tambahKategori">
            <Input v-model="kategoriBaru" placeholder="Nama kategori baru, misalnya Sewa" data-uji="kategori-baru" />
            <Button type="submit" variant="outline" :disabled="!kategoriBaru.trim() || sibuk" data-uji="kategori-tambah">Tambah</Button>
          </form>
        </section>

        <section class="h-fit rounded-xl border bg-card p-4 lg:col-span-2">
          <h2 class="text-sm font-semibold">Cadangan data</h2>
          <p class="text-xs text-muted-foreground">Satu file zip berisi semua catatan dan foto. Simpan salinannya di luar komputer ini (flashdisk lain atau penyimpanan awan) secara berkala.</p>
          <div class="mt-3 flex flex-wrap gap-2">
            <Button :disabled="sibuk" data-uji="cadangkan" @click="cadangkan(false)">Cadangkan sekarang</Button>
            <Button variant="outline" :disabled="sibuk" @click="cadangkan(true)">Cadangkan ke lokasi lain…</Button>
            <Button variant="outline" class="text-destructive" :disabled="sibuk" data-uji="pulihkan" @click="pilihPulih">Pulihkan dari cadangan…</Button>
          </div>
        </section>

        <section class="rounded-xl border bg-card p-4 lg:col-span-2">
          <div class="flex flex-wrap items-center gap-3">
            <h2 class="mr-auto text-sm font-semibold">Log aktivitas</h2>
            <PilihPeriode v-model="periodeLog" @update:model-value="muatLog" />
          </div>
          <div class="mt-3 max-h-96 overflow-auto rounded-lg border">
            <table class="w-full text-sm" data-uji="log">
              <thead class="sticky top-0 bg-muted text-left text-xs text-muted-foreground">
                <tr><th class="px-3 py-2 font-medium">Waktu</th><th class="px-3 py-2 font-medium">Oleh</th><th class="px-3 py-2 font-medium">Kegiatan</th></tr>
              </thead>
              <tbody>
                <tr v-if="!log.length"><td colspan="3" class="px-3 py-6 text-center text-muted-foreground">Tidak ada kegiatan pada periode ini.</td></tr>
                <tr v-for="l in log" :key="l.id" class="border-t align-top">
                  <td class="px-3 py-1.5 whitespace-nowrap tabular-nums">{{ waktu(l.waktu) }}</td>
                  <td class="px-3 py-1.5 whitespace-nowrap">{{ l.pengguna ?? "—" }}</td>
                  <td class="px-3 py-1.5">{{ l.ringkasan }} <span class="text-xs text-muted-foreground">{{ l.tindakan }}</span></td>
                </tr>
              </tbody>
            </table>
          </div>
        </section>
      </template>
    </div>

    <Dialog :open="pulih !== null" @update:open="!$event && (pulih = null)">
      <DialogContent class="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Pulihkan dari cadangan?</DialogTitle>
          <DialogDescription>
            Semua data yang ada sekarang akan DIGANTI dengan isi cadangan ini. Data sekarang disimpan dulu secara otomatis di folder cadangan, jadi masih bisa dikembalikan.
          </DialogDescription>
        </DialogHeader>
        <p class="rounded-lg bg-muted px-3 py-2 text-xs break-all">{{ pulih }}</p>
        <DialogFooter>
          <Button variant="outline" @click="pulih = null">Tidak jadi</Button>
          <Button variant="destructive" :disabled="sibuk" data-uji="pulihkan-ya" @click="pulihkan">Ya, ganti data sekarang</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </Halaman>
</template>
