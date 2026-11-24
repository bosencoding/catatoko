import { describe, expect, it } from "vitest";
import { bacaJumlah, bacaRupiah, jumlah, rupiah, tanggal } from "./format";

describe("rupiah", () => {
  it("memformat gaya Indonesia", () => {
    expect(rupiah(385000)).toBe("Rp 385.000");
    expect(rupiah(1250000)).toBe("Rp 1.250.000");
    expect(rupiah(0)).toBe("Rp 0");
    expect(rupiah(-5000)).toBe("-Rp 5.000");
  });
  it("menampilkan garis untuk nilai tersembunyi", () => {
    expect(rupiah(null)).toBe("—");
    expect(rupiah(undefined)).toBe("—");
  });
});

describe("jumlah", () => {
  it("mengubah seperseribu menjadi satuan", () => {
    expect(jumlah(2000)).toBe("2");
    expect(jumlah(500)).toBe("0,5");
    expect(jumlah(1250)).toBe("1,25");
    expect(jumlah(1250750)).toBe("1.250,75");
    expect(jumlah(0)).toBe("0");
    expect(jumlah(-1000)).toBe("-1");
  });
  it("menambahkan satuan", () => {
    expect(jumlah(10000, "karung")).toBe("10 karung");
  });
});

describe("tanggal", () => {
  it("menampilkan nama bulan Indonesia", () => {
    expect(tanggal("2026-09-03")).toBe("03 Sep 2026");
    expect(tanggal("2026-05-31")).toBe("31 Mei 2026");
    expect(tanggal("2026-08-01")).toBe("01 Agu 2026");
  });
  it("mengembalikan teks asli bila bukan tanggal", () => {
    expect(tanggal("")).toBe("");
    expect(tanggal("kemarin")).toBe("kemarin");
  });
});

describe("bacaRupiah", () => {
  it("membaca titik sebagai pemisah ribuan", () => {
    expect(bacaRupiah("385.000")).toBe(385000);
    expect(bacaRupiah("Rp 1.250.000")).toBe(1250000);
    expect(bacaRupiah("385000")).toBe(385000);
    expect(bacaRupiah(" 15.000 ")).toBe(15000);
    expect(bacaRupiah("0")).toBe(0);
  });
  it("menolak isian yang tidak jelas", () => {
    for (const t of ["", "  ", "abc", "-5", "12,5", "38.5", "1.25.000", "12e3", "1 000 x"]) {
      expect(bacaRupiah(t), t).toBeNull();
    }
  });
});

describe("bacaJumlah", () => {
  it("membaca koma sebagai desimal", () => {
    expect(bacaJumlah("0,5")).toBe(500);
    expect(bacaJumlah("1.250,75")).toBe(1250750);
    expect(bacaJumlah("10")).toBe(10000);
    expect(bacaJumlah("2,125")).toBe(2125);
  });
  it("satu titik tanpa koma selalu desimal (angka timbangan), tidak pernah ribuan", () => {
    expect(bacaJumlah("0.5")).toBe(500);
    expect(bacaJumlah("12.25")).toBe(12250);
    expect(bacaJumlah("0.500")).toBe(500);
    expect(bacaJumlah("0.125")).toBe(125);
    expect(bacaJumlah("1.250")).toBe(1250);
    expect(bacaJumlah("12.5")).toBe(12500);
  });
  it("titik sebagai ribuan hanya bila tidak mungkin desimal", () => {
    expect(bacaJumlah("1.250.000")).toBe(1250000000);
    expect(bacaJumlah("1.250,5")).toBe(1250500);
    expect(bacaJumlah("0.1234")).toBeNull();
    expect(bacaJumlah("1.25.000")).toBeNull();
  });
  it("menolak nol, negatif, kosong, dan lebih dari 3 desimal", () => {
    for (const t of ["", "0", "0,0", "-1", "0,0005", "abc", "1,2,3", "1.2.3", ","]) {
      expect(bacaJumlah(t), t).toBeNull();
    }
  });
  it("tidak salah karena pembulatan pecahan", () => {
    expect(bacaJumlah("0,1")).toBe(100);
    expect(bacaJumlah("1,005")).toBe(1005);
    expect(bacaJumlah("4,35")).toBe(4350);
  });
});

import { bacaTanggal, isiTanggal } from "./format";

describe("bacaTanggal", () => {
  it("membaca hari/bulan/tahun", () => {
    expect(bacaTanggal("24/11/2026")).toBe("2026-11-24");
    expect(bacaTanggal("3/9/2026")).toBe("2026-09-03");
    expect(bacaTanggal("03-09-2026")).toBe("2026-09-03");
    expect(bacaTanggal(" 31.01.2027 ")).toBe("2027-01-31");
    expect(bacaTanggal("24/11/26")).toBe("2026-11-24");
    expect(bacaTanggal("29/02/2028")).toBe("2028-02-29");
  });
  it("menolak tanggal yang tidak ada atau tidak jelas", () => {
    for (const t of ["", "31/02/2026", "29/02/2027", "0/1/2026", "1/13/2026", "2026-11-24", "24/11", "24 Nov 2026", "24/11/126", "aa/bb/cccc"]) {
      expect(bacaTanggal(t), t).toBeNull();
    }
  });
});

describe("isiTanggal", () => {
  it("menulis hari/bulan/tahun", () => {
    expect(isiTanggal("2026-09-03")).toBe("03/09/2026");
    expect(isiTanggal("")).toBe("");
    expect(isiTanggal(null)).toBe("");
  });
});
