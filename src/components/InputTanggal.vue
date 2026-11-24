<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { Input } from "@/components/ui/input";
import { bacaTanggal, isiTanggal, tanggal } from "@/lib/format";

/** Tanggal diketik hari/bulan/tahun supaya tidak bergantung pada bahasa sistem. Nilainya YYYY-MM-DD, atau "" bila kosong/tidak sah. */
const props = defineProps<{ modelValue: string | null }>();
const emit = defineEmits<{ "update:modelValue": [iso: string] }>();
defineOptions({ inheritAttrs: false });

const teks = ref(isiTanggal(props.modelValue));
const salah = computed(() => teks.value.trim() !== "" && bacaTanggal(teks.value) === null);

watch(
  () => props.modelValue,
  (iso) => {
    if ((iso ?? "") !== (bacaTanggal(teks.value) ?? "")) teks.value = isiTanggal(iso);
  },
);

function ketik(v: string | number) {
  teks.value = String(v);
  emit("update:modelValue", bacaTanggal(teks.value) ?? "");
}
function selesai() {
  const iso = bacaTanggal(teks.value);
  if (iso) teks.value = isiTanggal(iso);
}
</script>

<template>
  <div class="relative">
    <Input v-bind="$attrs" :model-value="teks" inputmode="numeric" placeholder="hh/bb/tttt" class="tabular-nums" :aria-invalid="salah || undefined" @update:model-value="ketik" @blur="selesai" />
    <span v-if="!salah && modelValue" class="pointer-events-none absolute inset-y-0 right-2.5 flex items-center text-xs text-muted-foreground">{{ tanggal(modelValue) }}</span>
  </div>
</template>
