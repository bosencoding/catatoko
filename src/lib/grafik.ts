import { use } from "echarts/core";
import { BarChart, LineChart } from "echarts/charts";
import { GridComponent, LegendComponent, TooltipComponent } from "echarts/components";
import { CanvasRenderer } from "echarts/renderers";
import { computed } from "vue";
import { tema } from "@/lib/tema";

use([BarChart, LineChart, GridComponent, LegendComponent, TooltipComponent, CanvasRenderer]);

/** Warna teks dan garis grafik mengikuti mode terang/gelap. */
export const warna = computed(() =>
  tema.value === "gelap"
    ? { teks: "#a1a1aa", garis: "#3f3f46", latar: "#18181b", masuk: "#34d399", keluar: "#fb7185", saldo: "#38bdf8", utama: "#2dd4bf" }
    : { teks: "#71717a", garis: "#e4e4e7", latar: "#ffffff", masuk: "#059669", keluar: "#e11d48", saldo: "#0284c7", utama: "#0f766e" },
);

/** 1250000 → "1,25 jt" untuk sumbu grafik. */
export function ringkas(n: number): string {
  const a = Math.abs(n);
  const tulis = (x: number, s: string) => `${(n < 0 ? -x : x).toLocaleString("id-ID", { maximumFractionDigits: 1 })} ${s}`;
  if (a >= 1e9) return tulis(a / 1e9, "M");
  if (a >= 1e6) return tulis(a / 1e6, "jt");
  if (a >= 1e3) return tulis(a / 1e3, "rb");
  return n.toString();
}
