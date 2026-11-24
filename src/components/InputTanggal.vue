<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { Input } from "@/components/ui/input";
import { bacaTanggal, isiTanggal, tanggal } from "@/lib/format";

/**
 * Tanggal diketik hari/bulan/tahun supaya tidak bergantung pada bahasa sistem.
 * Nilai: YYYY-MM-DD bila sah, "" bila kosong, dan TEKS ASLINYA bila tidak sah. Dengan begitu
 * isian yang salah tidak pernah diam-diam dianggap kosong; Rust menolaknya saat disimpan.
 */
const props = defineProps<{ modelValue: string | null }>();
const emit = defineEmits<{ "update:modelValue": [nilai: string] }>();
defineOptions({ inheritAttrs: false });

const ISO = /^\d{4}-\d{2}-\d{2}$/;
const nilaiDari = (t: string) => bacaTanggal(t) ?? t.trim();
const teks = ref(isiTanggal(props.modelValue));
const salah = computed(() => teks.value.trim() !== "" && bacaTanggal(teks.value) === null);

watch(
  () => props.modelValue,
  (v) => {
    // Hanya menimpa ketikan bila induk benar-benar mengganti nilainya.
    if ((v ?? "") !== nilaiDari(teks.value)) teks.value = isiTanggal(v);
  },
);

function ketik(v: string | number) {
  teks.value = String(v);
  emit("update:modelValue", nilaiDari(teks.value));
}
function selesai() {
  const iso = bacaTanggal(teks.value);
  if (iso) teks.value = isiTanggal(iso);
}
</script>

<template>
  <div class="relative">
    <Input v-bind="$attrs" :model-value="teks" inputmode="numeric" placeholder="hh/bb/tttt" class="tabular-nums" :title="salah ? 'Tulis tanggal sebagai hari/bulan/tahun, misalnya 31/01/2027' : undefined" :aria-invalid="salah || undefined" @update:model-value="ketik" @blur="selesai" />
    <span v-if="!salah && modelValue && ISO.test(modelValue)" class="pointer-events-none absolute inset-y-0 right-2.5 flex items-center text-xs text-muted-foreground">{{ tanggal(modelValue) }}</span>
  </div>
</template>
