<script setup lang="ts">
import { ref, watch } from "vue";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { kelasKontrol } from "@/lib/gaya";

const props = defineProps<{ buka: boolean; judul: string; keterangan?: string; sibuk?: boolean }>();
const emit = defineEmits<{ "update:buka": [v: boolean]; konfirmasi: [alasan: string] }>();

const alasan = ref("");
watch(
  () => props.buka,
  (b) => {
    if (b) alasan.value = "";
  },
);
</script>

<template>
  <Dialog :open="buka" @update:open="emit('update:buka', $event)">
    <DialogContent class="sm:max-w-md">
      <DialogHeader>
        <DialogTitle>{{ judul }}</DialogTitle>
        <DialogDescription>{{ keterangan ?? "Catatan yang dibatalkan tetap terlihat (dicoret) dan tidak dihitung lagi." }}</DialogDescription>
      </DialogHeader>
      <label class="flex flex-col gap-1 text-sm">
        <span class="font-medium">Alasan pembatalan</span>
        <textarea v-model="alasan" :class="[kelasKontrol, 'h-20 py-2']" data-uji="alasan-batal" placeholder="Misalnya: salah ketik jumlah" />
      </label>
      <DialogFooter>
        <Button variant="outline" @click="emit('update:buka', false)">Tidak jadi</Button>
        <Button variant="destructive" :disabled="alasan.trim().length < 3 || sibuk" data-uji="konfirmasi-batal" @click="emit('konfirmasi', alasan.trim())">Batalkan</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
