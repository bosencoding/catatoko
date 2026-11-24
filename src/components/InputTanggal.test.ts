import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import { h, ref } from "vue";
import InputTanggal from "./InputTanggal.vue";

function pasang(awal: string) {
  const nilai = ref(awal);
  const w = mount({ render: () => h(InputTanggal, { modelValue: nilai.value, "onUpdate:modelValue": (v: string) => (nilai.value = v) }) });
  return { w, nilai, input: () => w.get("input") };
}

describe("InputTanggal", () => {
  it("tanggal sah menjadi YYYY-MM-DD", async () => {
    const { nilai, input } = pasang("");
    await input().setValue("3/9/2026");
    expect(nilai.value).toBe("2026-09-03");
    await input().trigger("blur");
    expect((input().element as HTMLInputElement).value).toBe("03/09/2026");
  });

  it("kosong menjadi teks kosong", async () => {
    const { nilai, input } = pasang("2026-09-03");
    expect((input().element as HTMLInputElement).value).toBe("03/09/2026");
    await input().setValue("");
    expect(nilai.value).toBe("");
  });

  it("isian tidak sah TIDAK sama dengan kosong: nilainya teks asli, supaya ditolak saat disimpan", async () => {
    const { nilai, input } = pasang("2027-01-31");
    await input().setValue("31/11/2027");
    expect(nilai.value).toBe("31/11/2027");
    expect(input().attributes("aria-invalid")).toBe("true");
    // ketikan pengguna tidak dihapus oleh pembaruan nilai dari induk
    expect((input().element as HTMLInputElement).value).toBe("31/11/2027");
    await input().setValue("01/2027");
    expect(nilai.value).toBe("01/2027");
  });
});
