<script setup lang="ts">
import { ref } from "vue";
import { Button } from "@/components/ui/button";

const props = defineProps<{ kode: string; labelLanjut: string }>();
defineEmits<{ lanjut: [] }>();

const sudah = ref(false);
const disalin = ref(false);

async function salin() {
  try {
    await navigator.clipboard.writeText(props.kode);
    disalin.value = true;
  } catch {
    disalin.value = false;
  }
}
</script>

<template>
  <div class="flex flex-col gap-4">
    <div class="select-all rounded-lg border bg-muted px-4 py-3 text-center font-mono text-lg font-semibold tracking-wider" data-uji="kode-pemulihan">
      {{ kode }}
    </div>
    <Button variant="outline" type="button" @click="salin">{{ disalin ? "Tersalin" : "Salin kode" }}</Button>
    <label class="flex items-center gap-2 text-sm">
      <input v-model="sudah" type="checkbox" class="size-4 accent-primary" data-uji="sudah-catat" />
      Sudah saya catat dan simpan
    </label>
    <Button type="button" :disabled="!sudah" data-uji="lanjut" @click="$emit('lanjut')">{{ labelLanjut }}</Button>
  </div>
</template>
