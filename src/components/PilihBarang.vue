<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { Input } from "@/components/ui/input";
import { jumlah } from "@/lib/format";
import type { Barang } from "@/lib/types";

const props = defineProps<{ modelValue: number | null; daftar: Barang[]; uji?: string | number }>();
const emit = defineEmits<{ "update:modelValue": [id: number | null] }>();

const terpilih = computed(() => props.daftar.find((b) => b.id === props.modelValue) ?? null);
const teks = ref(terpilih.value?.nama ?? "");
const buka = ref(false);

watch(terpilih, (b) => {
  teks.value = b?.nama ?? "";
});

const cocok = computed(() => {
  const cari = teks.value.trim().toLowerCase();
  const semua = !cari || cari === terpilih.value?.nama.toLowerCase();
  return props.daftar.filter((b) => semua || b.nama.toLowerCase().includes(cari) || b.kode.toLowerCase().includes(cari)).slice(0, 50);
});

function ketik(v: string | number) {
  teks.value = String(v);
  buka.value = true;
  if (props.modelValue !== null && teks.value !== terpilih.value?.nama) emit("update:modelValue", null);
}
function pilih(b: Barang) {
  teks.value = b.nama;
  buka.value = false;
  emit("update:modelValue", b.id);
}
function tutup() {
  buka.value = false;
  teks.value = terpilih.value?.nama ?? "";
}
function enter() {
  if (buka.value && cocok.value.length === 1) pilih(cocok.value[0]);
}
</script>

<template>
  <div class="relative">
    <Input
      :model-value="teks"
      :data-uji="`pilih-barang-${uji ?? ''}`"
      placeholder="Ketik nama atau kode barang…"
      autocomplete="off"
      @update:model-value="ketik"
      @focus="buka = true"
      @blur="tutup"
      @keydown.enter.prevent="enter"
      @keydown.esc="tutup"
    />
    <ul v-if="buka" class="absolute z-50 mt-1 max-h-64 w-full min-w-64 overflow-auto rounded-lg border bg-popover p-1 text-sm shadow-md">
      <li v-if="!cocok.length" class="px-2 py-1.5 text-muted-foreground">Tidak ada barang yang cocok.</li>
      <li
        v-for="b in cocok"
        :key="b.id"
        :data-uji="`opsi-barang-${b.id}`"
        class="flex cursor-pointer items-center justify-between gap-3 rounded-md px-2 py-1.5 hover:bg-accent"
        :class="b.id === modelValue ? 'bg-accent' : ''"
        @mousedown.prevent="pilih(b)"
      >
        <span class="truncate">{{ b.nama }} <span class="text-xs text-muted-foreground">{{ b.kode }}</span></span>
        <span class="shrink-0 text-xs tabular-nums" :class="b.stok > 0 ? 'text-muted-foreground' : 'text-destructive'">stok {{ jumlah(b.stok, b.satuan) }}</span>
      </li>
    </ul>
  </div>
</template>
