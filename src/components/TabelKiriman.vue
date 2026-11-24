<script setup lang="ts">
import { computed } from "vue";
import { Badge } from "@/components/ui/badge";
import { jumlah, rupiah, tanggal } from "@/lib/format";
import type { Kiriman } from "@/lib/types";
import { useSesi } from "@/stores/sesi";

const props = defineProps<{ kiriman: Kiriman[] }>();
const sesi = useSesi();

/** Kiriman yang akan diambil lebih dulu: yang pertama masih bersisa dan belum kedaluwarsa. */
const duluan = computed(() => props.kiriman.find((k) => !k.batal && k.sisa > 0 && !k.sudah_kedaluwarsa)?.id ?? null);

function warnaExp(k: Kiriman) {
  if (k.sudah_kedaluwarsa) return "text-destructive font-medium";
  if (k.hari_ke_kedaluwarsa !== null && k.hari_ke_kedaluwarsa <= 30) return "text-amber-600 dark:text-amber-400 font-medium";
  return "";
}
</script>

<template>
  <div class="overflow-auto rounded-lg border">
    <table class="w-full text-sm">
      <thead class="bg-muted/60 text-left text-xs text-muted-foreground">
        <tr>
          <th class="px-3 py-2 font-medium">Masuk</th>
          <th class="px-3 py-2 text-right font-medium">Sisa</th>
          <th v-if="sesi.bolehHargaBeli" class="px-3 py-2 text-right font-medium">Harga beli</th>
          <th class="px-3 py-2 font-medium">Spek</th>
          <th class="px-3 py-2 font-medium">Kedaluwarsa</th>
          <th class="px-3 py-2"></th>
        </tr>
      </thead>
      <tbody>
        <tr v-if="!kiriman.length">
          <td colspan="6" class="px-3 py-6 text-center text-muted-foreground">Belum ada kiriman. Catat lewat Barang Masuk.</td>
        </tr>
        <tr v-for="k in kiriman" :key="k.id" class="border-t" :class="k.batal ? 'text-muted-foreground line-through' : ''" :data-uji="`kiriman-${k.id}`">
          <td class="px-3 py-2 whitespace-nowrap">
            {{ tanggal(k.tanggal_masuk) }}
            <Badge v-if="k.id === duluan" class="ml-1 no-underline" data-uji="keluar-dulu">keluar dulu</Badge>
            <Badge v-if="k.stok_awal" variant="outline" class="ml-1">stok awal</Badge>
          </td>
          <td class="px-3 py-2 text-right tabular-nums whitespace-nowrap">
            {{ jumlah(k.sisa, k.satuan) }}
            <span class="text-xs text-muted-foreground">/ {{ jumlah(k.jumlah_awal) }}</span>
          </td>
          <td v-if="sesi.bolehHargaBeli" class="px-3 py-2 text-right tabular-nums whitespace-nowrap">{{ rupiah(k.harga_beli) }}</td>
          <td class="px-3 py-2">
            {{ k.spek ?? "—" }}
            <div v-if="k.pemasok" class="text-xs text-muted-foreground">{{ k.pemasok }}</div>
          </td>
          <td class="px-3 py-2 whitespace-nowrap" :class="warnaExp(k)">
            {{ k.kedaluwarsa ? tanggal(k.kedaluwarsa) : "—" }}
            <span v-if="k.sudah_kedaluwarsa" class="text-xs">(lewat)</span>
          </td>
          <td class="px-3 py-2 text-right whitespace-nowrap no-underline"><slot name="aksi" :kiriman="k" /></td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
