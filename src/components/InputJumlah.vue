<script setup lang="ts">
import { ref, watch } from "vue";
import { Input } from "@/components/ui/input";
import { bacaJumlah, isiJumlah } from "@/lib/format";

const props = defineProps<{ modelValue: number | null; satuan?: string; wajib?: boolean; placeholder?: string }>();
defineOptions({ inheritAttrs: false });
const emit = defineEmits<{ "update:modelValue": [nilai: number | null] }>();

const teks = ref(isiJumlah(props.modelValue));
const disentuh = ref(false);

watch(
  () => props.modelValue,
  (n) => {
    if (n === bacaJumlah(teks.value)) return;
    teks.value = isiJumlah(n);
    // Dikosongkan dari luar (formulir direset): jangan langsung ditandai merah.
    if (n === null) disentuh.value = false;
  },
);

function ketik(v: string | number) {
  teks.value = String(v);
  emit("update:modelValue", bacaJumlah(teks.value));
}
function selesai() {
  disentuh.value = true;
  const n = bacaJumlah(teks.value);
  if (n !== null) teks.value = isiJumlah(n);
}
</script>

<template>
  <div class="relative">
    <Input
      v-bind="$attrs"
      :model-value="teks"
      inputmode="decimal"
      class="text-right tabular-nums"
      :class="satuan ? 'pr-16' : ''"
      :placeholder="placeholder ?? '0'"
      :aria-invalid="(teks.trim() !== '' && modelValue === null) || (wajib && disentuh && modelValue === null) || undefined"
      @update:model-value="ketik"
      @blur="selesai"
    />
    <span v-if="satuan" class="pointer-events-none absolute inset-y-0 right-2.5 flex max-w-14 items-center truncate text-xs text-muted-foreground">{{ satuan }}</span>
  </div>
</template>
