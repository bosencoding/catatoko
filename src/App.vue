<script setup lang="ts">
import { watch } from "vue";
import { useRouter } from "vue-router";
import { Toaster } from "@/components/ui/sonner";
import { tema } from "@/lib/tema";
import { useSesi } from "@/stores/sesi";

const sesi = useSesi();
const router = useRouter();

// Sesi hilang (keluar, atau dikunci dari sisi Rust) → kembali ke layar masuk.
watch(
  () => sesi.sesi,
  (s) => {
    if (!s && !router.currentRoute.value.meta.publik) router.replace("/login");
  },
);
</script>

<template>
  <RouterView />
  <Toaster position="top-center" rich-colors :theme="tema === 'gelap' ? 'dark' : 'light'" />
</template>
