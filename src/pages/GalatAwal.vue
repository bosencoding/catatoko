<script setup lang="ts">
import { TriangleAlertIcon } from "@lucide/vue";
import { galatAwal } from "@/router";
</script>

<template>
  <div class="flex min-h-full items-center justify-center bg-muted/40 p-6">
    <div class="w-full max-w-lg rounded-xl border bg-card p-6" data-uji="galat-awal">
      <div class="flex items-center gap-3">
        <TriangleAlertIcon class="size-6 shrink-0 text-destructive" />
        <h1 class="text-lg font-semibold">Catat tidak bisa dibuka</h1>
      </div>
      <p class="mt-3 text-sm whitespace-pre-line break-words">{{ galatAwal }}</p>
      <p class="mt-4 text-sm text-muted-foreground">Tutup aplikasi ini, perbaiki penyebabnya, lalu buka lagi. Data Anda tidak diubah.</p>
    </div>
  </div>
</template>
