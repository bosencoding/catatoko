<script setup lang="ts">
defineProps<{ judul: string; keterangan?: string }>();
</script>

<template>
  <div class="flex h-full flex-col">
    <header class="flex flex-wrap items-center gap-3 border-b px-6 py-4">
      <div class="mr-auto">
        <h1 class="text-xl font-semibold tracking-tight">{{ judul }}</h1>
        <p v-if="keterangan" class="text-sm text-muted-foreground">{{ keterangan }}</p>
      </div>
      <slot name="aksi" />
    </header>
    <div class="min-h-0 flex-1 overflow-auto p-6">
      <slot />
    </div>
  </div>
</template>
