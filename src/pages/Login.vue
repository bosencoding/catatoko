<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import Kolom from "@/components/Kolom.vue";
import KodePemulihan from "@/components/KodePemulihan.vue";
import { panggil, pesanGalat } from "@/lib/api";
import type { Pengaturan } from "@/lib/types";
import { useSesi } from "@/stores/sesi";

const sesi = useSesi();
const router = useRouter();
const namaToko = ref("");
const mode = ref<"masuk" | "lupa" | "kode-baru">("masuk");
const f = ref({ nama_pengguna: "", sandi: "" });
const lupa = ref({ kode: "", nama_pengguna: "", sandi_baru: "" });
const kodeBaru = ref("");
const galat = ref("");
const sibuk = ref(false);

onMounted(async () => {
  try {
    namaToko.value = (await panggil<Pengaturan>("pengaturan_ambil")).nama_toko;
  } catch {
    // hiasan saja
  }
});

async function masuk() {
  galat.value = "";
  sibuk.value = true;
  try {
    await sesi.masuk(f.value.nama_pengguna, f.value.sandi);
    router.replace("/");
  } catch (e) {
    galat.value = pesanGalat(e);
    f.value.sandi = "";
  } finally {
    sibuk.value = false;
  }
}

async function pulihkan() {
  galat.value = "";
  sibuk.value = true;
  try {
    kodeBaru.value = await panggil<string>("pulihkan_sandi", lupa.value);
    mode.value = "kode-baru";
    lupa.value = { kode: "", nama_pengguna: "", sandi_baru: "" };
  } catch (e) {
    galat.value = pesanGalat(e);
  } finally {
    sibuk.value = false;
  }
}

function ganti(m: "masuk" | "lupa") {
  mode.value = m;
  galat.value = "";
}
</script>

<template>
  <div class="flex min-h-full items-center justify-center bg-muted/40 p-6">
    <Card class="w-full max-w-sm">
      <template v-if="mode === 'masuk'">
        <CardHeader>
          <CardTitle class="text-xl">{{ namaToko || "Catat" }}</CardTitle>
          <CardDescription>Masuk untuk mulai mencatat.</CardDescription>
        </CardHeader>
        <CardContent>
          <p v-if="sesi.pesanKunci" class="mb-3 rounded-lg border border-amber-500/40 bg-amber-500/10 px-3 py-2 text-sm" data-uji="pesan-kunci">{{ sesi.pesanKunci }}</p>
          <form class="flex flex-col gap-3" @submit.prevent="masuk">
            <Kolom label="Nama pengguna"><Input v-model="f.nama_pengguna" data-uji="nama-pengguna" autofocus autocapitalize="none" /></Kolom>
            <Kolom label="Kata sandi"><Input v-model="f.sandi" type="password" data-uji="sandi" /></Kolom>
            <p v-if="galat" class="text-sm text-destructive" data-uji="galat">{{ galat }}</p>
            <Button type="submit" :disabled="!f.nama_pengguna || !f.sandi || sibuk" data-uji="masuk">Masuk</Button>
            <button type="button" class="text-sm text-muted-foreground underline-offset-4 hover:underline" data-uji="lupa" @click="ganti('lupa')">Lupa kata sandi?</button>
          </form>
        </CardContent>
      </template>

      <template v-else-if="mode === 'lupa'">
        <CardHeader>
          <CardTitle class="text-xl">Atur ulang kata sandi pemilik</CardTitle>
          <CardDescription>Pakai kode pemulihan yang dicatat saat akun pemilik dibuat. Staf yang lupa kata sandi cukup minta pemilik mengaturnya ulang.</CardDescription>
        </CardHeader>
        <CardContent>
          <form class="flex flex-col gap-3" @submit.prevent="pulihkan">
            <Kolom label="Kode pemulihan"><Input v-model="lupa.kode" class="font-mono uppercase" placeholder="XXXXX-XXXXX-XXXXX-XXXXX" data-uji="kode" /></Kolom>
            <Kolom label="Nama pengguna pemilik"><Input v-model="lupa.nama_pengguna" autocapitalize="none" data-uji="nama-pengguna-pulih" /></Kolom>
            <Kolom label="Kata sandi baru" bantuan="Minimal 8 karakter"><Input v-model="lupa.sandi_baru" type="password" data-uji="sandi-baru" /></Kolom>
            <p v-if="galat" class="text-sm text-destructive" data-uji="galat">{{ galat }}</p>
            <Button type="submit" :disabled="!lupa.kode || !lupa.nama_pengguna || lupa.sandi_baru.length < 8 || sibuk" data-uji="pulihkan">Atur ulang kata sandi</Button>
            <button type="button" class="text-sm text-muted-foreground underline-offset-4 hover:underline" @click="ganti('masuk')">Kembali ke layar masuk</button>
          </form>
        </CardContent>
      </template>

      <template v-else>
        <CardHeader>
          <CardTitle class="text-xl">Kata sandi sudah diganti</CardTitle>
          <CardDescription>Kode pemulihan lama tidak berlaku lagi. Catat kode baru ini sebagai penggantinya.</CardDescription>
        </CardHeader>
        <CardContent>
          <KodePemulihan :kode="kodeBaru" label-lanjut="Kembali ke layar masuk" @lanjut="ganti('masuk')" />
        </CardContent>
      </template>
    </Card>
  </div>
</template>
