<script setup lang="ts">
import { ref, watch } from "vue";
import InputTanggal from "@/components/InputTanggal.vue";
import { hariIni } from "@/lib/format";
import { kelasKontrol } from "@/lib/gaya";

export interface Periode {
  dari: string;
  sampai: string;
}
type Pilihan = "bulan_ini" | "bulan_lalu" | "30_hari" | "tahun_ini" | "bebas";

const props = defineProps<{ modelValue: Periode }>();
const emit = defineEmits<{ "update:modelValue": [p: Periode] }>();

const pilihan = ref<Pilihan>("bulan_ini");
const dari = ref(props.modelValue.dari);
const sampai = ref(props.modelValue.sampai);

function hitung(p: Pilihan): Periode | null {
  const kini = new Date();
  const [t, b] = [kini.getFullYear(), kini.getMonth()];
  switch (p) {
    case "bulan_ini":
      return { dari: hariIni(new Date(t, b, 1)), sampai: hariIni(new Date(t, b + 1, 0)) };
    case "bulan_lalu":
      return { dari: hariIni(new Date(t, b - 1, 1)), sampai: hariIni(new Date(t, b, 0)) };
    case "30_hari":
      return { dari: hariIni(new Date(Date.now() - 29 * 86_400_000)), sampai: hariIni() };
    case "tahun_ini":
      return { dari: `${t}-01-01`, sampai: `${t}-12-31` };
    default:
      return null;
  }
}

function terapkan() {
  const p = hitung(pilihan.value);
  if (p) [dari.value, sampai.value] = [p.dari, p.sampai];
  const iso = /^\d{4}-\d{2}-\d{2}$/;
  if (iso.test(dari.value) && iso.test(sampai.value) && dari.value <= sampai.value) emit("update:modelValue", { dari: dari.value, sampai: sampai.value });
}
watch(pilihan, terapkan);
watch([dari, sampai], () => pilihan.value === "bebas" && terapkan());
</script>

<script lang="ts">
import { hariIni as _hariIni } from "@/lib/format";
/** Periode bawaan: bulan berjalan. */
export function bulanIni(): { dari: string; sampai: string } {
  const k = new Date();
  return { dari: _hariIni(new Date(k.getFullYear(), k.getMonth(), 1)), sampai: _hariIni(new Date(k.getFullYear(), k.getMonth() + 1, 0)) };
}
</script>

<template>
  <div class="flex flex-wrap items-center gap-2">
    <div class="w-36">
      <select v-model="pilihan" :class="kelasKontrol" data-uji="periode">
        <option value="bulan_ini">Bulan ini</option>
        <option value="bulan_lalu">Bulan lalu</option>
        <option value="30_hari">30 hari terakhir</option>
        <option value="tahun_ini">Tahun ini</option>
        <option value="bebas">Pilih tanggal…</option>
      </select>
    </div>
    <template v-if="pilihan === 'bebas'">
      <div class="w-48"><InputTanggal v-model="dari" data-uji="periode-dari" /></div>
      <span class="text-sm text-muted-foreground">s.d.</span>
      <div class="w-48"><InputTanggal v-model="sampai" data-uji="periode-sampai" /></div>
      <span v-if="dari && sampai && dari > sampai" class="text-xs text-destructive">Tanggal awal setelah tanggal akhir.</span>
    </template>
  </div>
</template>
