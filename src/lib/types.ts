// Cermin struct Rust. Uang: Rupiah bulat. Jumlah barang: per seperseribu satuan.

export type Modul = "barang" | "barang_masuk" | "barang_keluar" | "kas" | "laporan";
/** 0 tidak ada · 1 lihat · 2 tambah · 3 ubah & batalkan */
export type Tingkat = 0 | 1 | 2 | 3;

export interface Sesi {
  pengguna_id: number;
  nama_pengguna: string;
  nama: string;
  pemilik: boolean;
  lihat_harga_beli: boolean;
  hak: Partial<Record<Modul, Tingkat>>;
}

export interface Pengaturan {
  nama_toko: string;
  kunci_otomatis_menit: number;
}

export interface Pengguna {
  id: number;
  nama_pengguna: string;
  nama: string;
  peran: "pemilik" | "staf";
  lihat_harga_beli: boolean;
  aktif: boolean;
  hak: Partial<Record<Modul, Tingkat>>;
}
export type PenggunaMasukan = Omit<Pengguna, "id">;

export interface BarisLog {
  id: number;
  waktu: string;
  pengguna: string | null;
  tindakan: string;
  ringkasan: string;
}

export interface Barang {
  id: number;
  kode: string;
  nama: string;
  kategori_id: number | null;
  kategori: string | null;
  satuan: string;
  harga_jual: number;
  stok_minimum: number;
  foto: string | null;
  aktif: boolean;
  stok: number;
  nilai_stok: number | null;
}
export interface BarangMasukan {
  kode: string;
  nama: string;
  kategori_id: number | null;
  satuan: string;
  harga_jual: number;
  stok_minimum: number;
  foto_sumber: string | null;
  aktif: boolean;
}
export interface Kategori {
  id: number;
  nama: string;
}

export interface Kiriman {
  id: number;
  barang_id: number;
  barang_nama: string;
  satuan: string;
  tanggal_masuk: string;
  jumlah_awal: number;
  sisa: number;
  harga_beli: number | null;
  spek: string | null;
  kedaluwarsa: string | null;
  pemasok: string | null;
  foto: string | null;
  stok_awal: boolean;
  batal: boolean;
  alasan_batal: string | null;
  sudah_kedaluwarsa: boolean;
  hari_ke_kedaluwarsa: number | null;
  pernah_keluar: boolean;
}
export interface MasukMasukan {
  barang_id: number;
  tanggal_masuk: string;
  jumlah: number;
  harga_beli: number;
  spek: string | null;
  kedaluwarsa: string | null;
  pemasok: string | null;
  foto_sumber: string | null;
  stok_awal: boolean;
  akun_kas_id: number | null;
}
export interface PerbaikanKiriman {
  harga_beli: number;
  spek: string | null;
  kedaluwarsa: string | null;
  pemasok: string | null;
  foto_sumber: string | null;
}

export interface Alokasi {
  kiriman_id: number;
  jumlah: number;
}
export interface KeluarMasukan {
  tanggal: string;
  akun_kas_id: number;
  catatan: string | null;
  baris: { barang_id: number; harga_jual: number; alokasi: Alokasi[] }[];
  izinkan_kedaluwarsa: boolean;
}
export interface RincianKeluar {
  barang_id: number;
  barang_nama: string;
  satuan: string;
  kiriman_id: number;
  tanggal_masuk: string;
  spek: string | null;
  jumlah: number;
  harga_jual: number;
  subtotal: number;
  harga_beli: number | null;
}
export interface TransaksiKeluar {
  id: number;
  tanggal: string;
  akun: string;
  catatan: string | null;
  batal: boolean;
  alasan_batal: string | null;
  total: number;
  laba: number | null;
  dibuat_oleh_nama: string | null;
  baris: RincianKeluar[];
}
export interface Penyesuaian {
  id: number;
  kiriman_id: number;
  barang_id: number;
  barang_nama: string;
  satuan: string;
  tanggal_masuk: string;
  tanggal: string;
  selisih: number;
  alasan: string;
  catatan: string | null;
  batal: boolean;
  nilai: number | null;
  dibuat_oleh_nama: string | null;
}

export interface AkunKas {
  id: number;
  nama: string;
  aktif: boolean;
}
export interface KategoriKas {
  id: number;
  nama: string;
  bawaan: boolean;
}
export interface KasMasukan {
  tanggal: string;
  arah: "masuk" | "keluar";
  akun_kas_id: number;
  kategori_id: number;
  jumlah: number;
  keterangan: string | null;
}
export interface SaringKas {
  dari: string;
  sampai: string;
  akun_kas_id: number | null;
  kategori_id: number | null;
  arah: "masuk" | "keluar" | null;
}
export interface BarisKas {
  id: number;
  tanggal: string;
  arah: "masuk" | "keluar";
  akun_kas_id: number;
  akun: string;
  kategori_id: number;
  kategori: string;
  jumlah: number;
  keterangan: string | null;
  sumber: "manual" | "otomatis";
  ref_jenis: string | null;
  ref_id: number | null;
  batal: boolean;
  alasan_batal: string | null;
  saldo: number;
  dibuat_oleh_nama: string | null;
}
export interface SaldoAkun {
  akun_kas_id: number;
  nama: string;
  aktif: boolean;
  saldo: number;
}
export interface Saldo {
  per_akun: SaldoAkun[];
  total: number;
}

export type Butir = "hari" | "minggu" | "bulan";
export interface Ringkasan {
  kas: { saldo_total: number; saldo_per_akun: SaldoAkun[]; masuk: number; keluar: number } | null;
  nilai_stok: number | null;
  laba: number | null;
  deret_kas: { label: string; masuk: number; keluar: number; saldo_akhir: number }[] | null;
  deret_barang: { label: string; masuk: number; keluar: number }[];
  terlaris: [string, number][];
  stok_menipis: Barang[];
  hampir_kedaluwarsa: Kiriman[];
  stok_lama: Kiriman[];
}

export type Sel =
  | { t: "teks"; v: string }
  | { t: "rupiah"; v: number }
  | { t: "jumlah"; v: number }
  | { t: "tanggal"; v: string }
  | { t: "kosong" };
export interface Laporan {
  judul: string;
  nama_toko: string;
  periode: string;
  kolom: string[];
  baris: Sel[][];
  total: Sel[] | null;
  ringkasan: [string, Sel][];
}
export type JenisLaporan =
  | { jenis: "arus_kas" }
  | { jenis: "posisi_stok" }
  | { jenis: "kartu_stok"; barang_id: number }
  | { jenis: "barang_masuk" }
  | { jenis: "barang_keluar" }
  | { jenis: "laba_kotor" };
