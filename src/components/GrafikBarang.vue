<script setup lang="ts">
import { computed } from "vue";
import VChart from "vue-echarts";
import { jumlah } from "@/lib/format";
import { warna } from "@/lib/grafik";
import type { Ringkasan } from "@/lib/types";

const props = defineProps<{ deret: Ringkasan["deret_barang"] }>();

const opsi = computed(() => {
  const w = warna.value;
  return {
    textStyle: { color: w.teks, fontFamily: "inherit" },
    grid: { left: 8, right: 8, top: 36, bottom: 4, containLabel: true },
    legend: { top: 0, left: 0, textStyle: { color: w.teks }, itemWidth: 12, itemHeight: 8 },
    tooltip: { trigger: "axis", backgroundColor: w.latar, borderColor: w.garis, textStyle: { color: w.teks }, valueFormatter: (v: number) => jumlah(Math.round(v * 1000)) },
    xAxis: { type: "category", data: props.deret.map((t) => t.label), axisLine: { lineStyle: { color: w.garis } }, axisTick: { show: false } },
    yAxis: { type: "value", splitLine: { lineStyle: { color: w.garis } } },
    series: [
      { name: "Barang masuk", type: "bar", data: props.deret.map((t) => t.masuk / 1000), itemStyle: { color: w.utama, borderRadius: [3, 3, 0, 0] }, barMaxWidth: 18 },
      { name: "Barang keluar", type: "bar", data: props.deret.map((t) => t.keluar / 1000), itemStyle: { color: "#f59e0b", borderRadius: [3, 3, 0, 0] }, barMaxWidth: 18 },
    ],
  };
});
</script>

<template>
  <VChart :option="opsi" autoresize class="h-72 w-full" data-uji="grafik-barang" />
</template>
