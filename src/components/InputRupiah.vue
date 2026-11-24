<script setup lang="ts">
import { ref, watch } from "vue";
import { Input } from "@/components/ui/input";
import { bacaRupiah, rupiah } from "@/lib/format";

const props = defineProps<{ modelValue: number | null; wajib?: boolean; placeholder?: string }>();
defineOptions({ inheritAttrs: false });
const emit = defineEmits<{ "update:modelValue": [nilai: number | null] }>();

const tampil = (n: number | null) => (n === null ? "" : rupiah(n).replace("Rp ", ""));
const teks = ref(tampil(props.modelValue));
const disentuh = ref(false);

watch(
  () => props.modelValue,
  (n) => {
    if (n === bacaRupiah(teks.value)) return;
    teks.value = tampil(n);
    // Dikosongkan dari luar (formulir direset): jangan langsung ditandai merah.
    if (n === null) disentuh.value = false;
  },
);

function ketik(v: string | number) {
  teks.value = String(v);
  emit("update:modelValue", bacaRupiah(teks.value));
}
function selesai() {
  disentuh.value = true;
  const n = bacaRupiah(teks.value);
  if (n !== null) teks.value = tampil(n);
}
</script>

<template>
  <div class="relative">
    <span class="pointer-events-none absolute inset-y-0 left-2.5 flex items-center text-sm text-muted-foreground">Rp</span>
    <Input
      v-bind="$attrs"
      :model-value="teks"
      inputmode="numeric"
      class="pl-9 text-right tabular-nums"
      :placeholder="placeholder ?? '0'"
      :aria-invalid="(teks.trim() !== '' && modelValue === null) || (wajib && disentuh && modelValue === null) || undefined"
      @update:model-value="ketik"
      @blur="selesai"
    />
  </div>
</template>
