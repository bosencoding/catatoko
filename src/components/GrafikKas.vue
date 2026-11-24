<script setup lang="ts">
import { computed } from "vue";
import VChart from "vue-echarts";
import { rupiah } from "@/lib/format";
import { ringkas, warna } from "@/lib/grafik";
import type { Ringkasan } from "@/lib/types";

const props = defineProps<{ deret: NonNullable<Ringkasan["deret_kas"]> }>();

const opsi = computed(() => {
  const w = warna.value;
  return {
    textStyle: { color: w.teks, fontFamily: "inherit" },
    grid: { left: 8, right: 8, top: 36, bottom: 4, containLabel: true },
    legend: { top: 0, left: 0, textStyle: { color: w.teks }, itemWidth: 12, itemHeight: 8 },
    tooltip: {
      trigger: "axis",
      backgroundColor: w.latar,
      borderColor: w.garis,
      textStyle: { color: w.teks },
      valueFormatter: (v: number) => rupiah(v),
    },
    xAxis: { type: "category", data: props.deret.map((t) => t.label), axisLine: { lineStyle: { color: w.garis } }, axisTick: { show: false } },
    yAxis: { type: "value", axisLabel: { formatter: (v: number) => ringkas(v) }, splitLine: { lineStyle: { color: w.garis } } },
    series: [
      { name: "Kas masuk", type: "bar", data: props.deret.map((t) => t.masuk), itemStyle: { color: w.masuk, borderRadius: [3, 3, 0, 0] }, barMaxWidth: 18 },
      { name: "Kas keluar", type: "bar", data: props.deret.map((t) => t.keluar), itemStyle: { color: w.keluar, borderRadius: [3, 3, 0, 0] }, barMaxWidth: 18 },
      { name: "Saldo", type: "line", data: props.deret.map((t) => t.saldo_akhir), smooth: true, showSymbol: props.deret.length <= 31, symbolSize: 5, lineStyle: { color: w.saldo, width: 2 }, itemStyle: { color: w.saldo } },
    ],
  };
});
</script>

<template>
  <VChart :option="opsi" autoresize class="h-72 w-full" data-uji="grafik-kas" />
</template>
