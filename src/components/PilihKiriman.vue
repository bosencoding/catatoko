<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import InputJumlah from "@/components/InputJumlah.vue";
import { jumlah, tanggal } from "@/lib/format";
import type { Alokasi, Kiriman } from "@/lib/types";

const props = defineProps<{ buka: boolean; nama: string; satuan: string; kiriman: Kiriman[]; awal: Alokasi[] }>();
const emit = defineEmits<{ "update:buka": [v: boolean]; pakai: [alokasi: Alokasi[], adaKedaluwarsa: boolean] }>();

const isi = ref<Record<number, number | null>>({});
const setuju = ref(false);

watch(
  () => props.buka,
  (b) => {
    if (!b) return;
    isi.value = Object.fromEntries(props.kiriman.map((k) => [k.id, props.awal.find((a) => a.kiriman_id === k.id)?.jumlah ?? null]));
    setuju.value = false;
  },
);

const pilihan = computed(() => props.kiriman.filter((k) => (isi.value[k.id] ?? 0) > 0).map((k) => ({ k, jumlah: isi.value[k.id] as number })));
const total = computed(() => pilihan.value.reduce((n, p) => n + p.jumlah, 0));
const lebih = computed(() => pilihan.value.find((p) => p.jumlah > p.k.sisa)?.k ?? null);
const adaKedaluwarsa = computed(() => pilihan.value.some((p) => p.k.sudah_kedaluwarsa));
const bisa = computed(() => total.value > 0 && !lebih.value && (!adaKedaluwarsa.value || setuju.value));

function pakai() {
  emit("pakai", pilihan.value.map((p) => ({ kiriman_id: p.k.id, jumlah: p.jumlah })), adaKedaluwarsa.value);
  emit("update:buka", false);
}
</script>

<template>
  <Dialog :open="buka" @update:open="emit('update:buka', $event)">
    <DialogContent class="sm:max-w-2xl">
      <DialogHeader>
        <DialogTitle>Pilih kiriman: {{ nama }}</DialogTitle>
        <DialogDescription>Isi jumlah yang diambil dari tiap kiriman. Kiriman teratas adalah stok terlama.</DialogDescription>
      </DialogHeader>
      <div class="max-h-80 overflow-auto rounded-lg border">
        <table class="w-full text-sm">
          <thead class="bg-muted/60 text-left text-xs text-muted-foreground">
            <tr>
              <th class="px-3 py-2 font-medium">Masuk</th>
              <th class="px-3 py-2 font-medium">Spek</th>
              <th class="px-3 py-2 font-medium">Kedaluwarsa</th>
              <th class="px-3 py-2 text-right font-medium">Sisa</th>
              <th class="w-36 px-3 py-2 text-right font-medium">Diambil</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="k in kiriman" :key="k.id" class="border-t">
              <td class="px-3 py-2 whitespace-nowrap">{{ tanggal(k.tanggal_masuk) }}</td>
              <td class="px-3 py-2">{{ k.spek ?? "—" }}</td>
              <td class="px-3 py-2 whitespace-nowrap" :class="k.sudah_kedaluwarsa ? 'font-medium text-destructive' : ''">
                {{ k.kedaluwarsa ? tanggal(k.kedaluwarsa) : "—" }}<span v-if="k.sudah_kedaluwarsa"> (lewat)</span>
              </td>
              <td class="px-3 py-2 text-right tabular-nums whitespace-nowrap">{{ jumlah(k.sisa, satuan) }}</td>
              <td class="px-3 py-1.5"><InputJumlah v-model="isi[k.id]" :data-uji="`ambil-${k.id}`" /></td>
            </tr>
          </tbody>
        </table>
      </div>
      <p v-if="lebih" class="text-sm text-destructive">Jumlah dari kiriman {{ tanggal(lebih.tanggal_masuk) }} melebihi sisanya.</p>
      <label v-if="adaKedaluwarsa" class="flex items-start gap-2 rounded-lg border border-destructive/40 bg-destructive/5 p-3 text-sm">
        <input v-model="setuju" type="checkbox" class="mt-0.5 size-4 accent-primary" data-uji="setuju-kedaluwarsa" />
        <span>Ada kiriman yang sudah lewat tanggal kedaluwarsa. Centang bila tetap ingin mengeluarkannya.</span>
      </label>
      <DialogFooter class="items-center">
        <span class="mr-auto text-sm">Total diambil: <b class="tabular-nums">{{ jumlah(total, satuan) }}</b></span>
        <Button variant="outline" @click="emit('update:buka', false)">Tidak jadi</Button>
        <Button :disabled="!bisa" data-uji="pakai-kiriman" @click="pakai">Pakai pilihan ini</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
