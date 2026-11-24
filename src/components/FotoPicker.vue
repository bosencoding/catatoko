<script setup lang="ts">
import { open } from "@tauri-apps/plugin-dialog";
import { Button } from "@/components/ui/button";
import Foto from "@/components/Foto.vue";

/** `modelValue` = jalur file baru yang dipilih; `lama` = foto yang sudah tersimpan. */
defineProps<{ modelValue: string | null; lama?: string | null }>();
const emit = defineEmits<{ "update:modelValue": [jalur: string | null] }>();

async function pilih() {
  const jalur = await open({ multiple: false, directory: false, filters: [{ name: "Gambar", extensions: ["jpg", "jpeg", "png", "webp"] }] });
  if (typeof jalur === "string") emit("update:modelValue", jalur);
}
</script>

<template>
  <div class="flex items-center gap-3">
    <Foto :relatif="modelValue ? null : (lama ?? null)" kecil class="size-16 shrink-0" />
    <div class="min-w-0 flex-1">
      <div class="flex gap-2">
        <Button type="button" variant="outline" size="sm" @click="pilih">{{ lama || modelValue ? "Ganti foto" : "Pilih foto" }}</Button>
        <Button v-if="modelValue" type="button" variant="ghost" size="sm" @click="emit('update:modelValue', null)">Batal ganti</Button>
      </div>
      <p class="mt-1 truncate text-xs text-muted-foreground" :title="modelValue ?? ''">
        {{ modelValue ? `Foto baru: ${modelValue.split(/[\\/]/).pop()}` : "JPG, PNG, atau WebP. Diperkecil otomatis." }}
      </p>
    </div>
  </div>
</template>
