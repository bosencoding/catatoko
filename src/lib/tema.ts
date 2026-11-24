import { ref } from "vue";

type Tema = "terang" | "gelap";

function awal(): Tema {
  try {
    const simpan = localStorage.getItem("tema");
    if (simpan === "terang" || simpan === "gelap") return simpan;
  } catch {
    // penyimpanan lokal tidak tersedia: ikuti sistem
  }
  return window.matchMedia?.("(prefers-color-scheme: dark)").matches ? "gelap" : "terang";
}

export const tema = ref<Tema>(awal());

function terapkan() {
  document.documentElement.classList.toggle("dark", tema.value === "gelap");
}
terapkan();

export function gantiTema() {
  tema.value = tema.value === "gelap" ? "terang" : "gelap";
  try {
    localStorage.setItem("tema", tema.value);
  } catch {
    // abaikan
  }
  terapkan();
}
