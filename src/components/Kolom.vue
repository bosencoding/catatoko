<script setup lang="ts">
defineProps<{ label: string; galat?: string; bantuan?: string }>();
</script>

<template>
  <label class="flex flex-col gap-1 text-sm">
    <span class="font-medium">{{ label }}</span>
    <slot />
    <span v-if="galat" class="text-xs text-destructive">{{ galat }}</span>
    <span v-else-if="bantuan" class="text-xs text-muted-foreground">{{ bantuan }}</span>
  </label>
</template>
