<script setup lang="ts">
import { computed, ref } from "vue";
import { useRouter } from "vue-router";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import Kolom from "@/components/Kolom.vue";
import KodePemulihan from "@/components/KodePemulihan.vue";
import { panggil, pesanGalat } from "@/lib/api";

const router = useRouter();
const f = ref({ nama_toko: "", nama: "", nama_pengguna: "", sandi: "", ulangi: "" });
const galat = ref("");
const sibuk = ref(false);
const kode = ref("");

const beda = computed(() => f.value.ulangi !== "" && f.value.sandi !== f.value.ulangi);
const lengkap = computed(
  () => f.value.nama_toko.trim() && f.value.nama.trim() && f.value.nama_pengguna.trim() && f.value.sandi.length >= 8 && f.value.sandi === f.value.ulangi,
);

async function simpan() {
  galat.value = "";
  sibuk.value = true;
  try {
    const { ulangi: _ulangi, ...isi } = f.value;
    kode.value = await panggil<string>("setup_awal", isi);
  } catch (e) {
    galat.value = pesanGalat(e);
  } finally {
    sibuk.value = false;
  }
}
</script>

<template>
  <div class="flex min-h-full items-center justify-center bg-muted/40 p-6">
    <Card class="w-full max-w-md">
      <template v-if="!kode">
        <CardHeader>
          <CardTitle class="text-xl">Selamat datang di Catat</CardTitle>
          <CardDescription>Buat akun pemilik toko. Akun ini punya akses penuh; akun staf bisa ditambah nanti.</CardDescription>
        </CardHeader>
        <CardContent>
          <form class="flex flex-col gap-3" @submit.prevent="simpan">
            <Kolom label="Nama toko"><Input v-model="f.nama_toko" data-uji="nama-toko" autofocus /></Kolom>
            <Kolom label="Nama Anda"><Input v-model="f.nama" data-uji="nama" /></Kolom>
            <Kolom label="Nama pengguna" bantuan="Dipakai saat masuk, misalnya: budi"><Input v-model="f.nama_pengguna" data-uji="nama-pengguna" autocapitalize="none" /></Kolom>
            <Kolom label="Kata sandi" bantuan="Minimal 8 karakter"><Input v-model="f.sandi" type="password" data-uji="sandi" /></Kolom>
            <Kolom label="Ulangi kata sandi" :galat="beda ? 'Kata sandi tidak sama.' : undefined">
              <Input v-model="f.ulangi" type="password" data-uji="ulangi" :aria-invalid="beda || undefined" />
            </Kolom>
            <p v-if="galat" class="text-sm text-destructive" data-uji="galat">{{ galat }}</p>
            <Button type="submit" :disabled="!lengkap || sibuk" data-uji="simpan">Buat akun pemilik</Button>
          </form>
        </CardContent>
      </template>
      <template v-else>
        <CardHeader>
          <CardTitle class="text-xl">Catat kode pemulihan ini</CardTitle>
          <CardDescription>
            Kode ini satu-satunya cara mengatur ulang kata sandi pemilik kalau lupa. Kode hanya ditampilkan sekali. Tulis di kertas dan simpan di tempat aman.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <KodePemulihan :kode="kode" label-lanjut="Lanjut ke layar masuk" @lanjut="router.replace('/login')" />
        </CardContent>
      </template>
    </Card>
  </div>
</template>
