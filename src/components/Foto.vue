<script setup lang="ts">
import { ref, watch } from "vue";
import { ImageIcon } from "@lucide/vue";
import { urlFoto } from "@/lib/foto";

const props = defineProps<{ relatif: string | null; kecil?: boolean }>();
const url = ref("");

watch(
  () => props.relatif,
  async (r) => {
    url.value = "";
    if (!r) return;
    try {
      url.value = await urlFoto(r, props.kecil);
    } catch {
      url.value = "";
    }
  },
  { immediate: true },
);
</script>

<template>
  <div class="flex items-center justify-center overflow-hidden rounded-lg bg-muted text-muted-foreground">
    <img v-if="url" :src="url" alt="" class="size-full object-cover" @error="url = ''" />
    <ImageIcon v-else class="size-1/3 opacity-50" />
  </div>
</template>
