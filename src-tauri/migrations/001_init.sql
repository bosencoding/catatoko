CREATE TABLE pengguna(
  id INTEGER PRIMARY KEY,
  nama_pengguna TEXT NOT NULL UNIQUE COLLATE NOCASE,
  nama TEXT NOT NULL,
  hash_sandi TEXT NOT NULL,
  peran TEXT NOT NULL CHECK(peran IN('pemilik','staf')),
  lihat_harga_beli INTEGER NOT NULL DEFAULT 0,
  aktif INTEGER NOT NULL DEFAULT 1,
  gagal_login INTEGER NOT NULL DEFAULT 0,
  ditahan_sampai TEXT,
  dibuat_pada TEXT NOT NULL
) STRICT;

CREATE TABLE hak_akses(
  pengguna_id INTEGER NOT NULL REFERENCES pengguna(id),
  modul TEXT NOT NULL CHECK(modul IN('barang','barang_masuk','barang_keluar','kas','laporan')),
  tingkat INTEGER NOT NULL CHECK(tingkat BETWEEN 0 AND 3),
  PRIMARY KEY(pengguna_id, modul)
) STRICT;

CREATE TABLE pengaturan(kunci TEXT PRIMARY KEY, nilai TEXT NOT NULL) STRICT;

CREATE TABLE kategori_barang(id INTEGER PRIMARY KEY, nama TEXT NOT NULL UNIQUE COLLATE NOCASE) STRICT;

CREATE TABLE barang(
  id INTEGER PRIMARY KEY,
  kode TEXT NOT NULL UNIQUE COLLATE NOCASE,
  nama TEXT NOT NULL,
  kategori_id INTEGER REFERENCES kategori_barang(id),
  satuan TEXT NOT NULL,
  harga_jual INTEGER NOT NULL DEFAULT 0 CHECK(harga_jual >= 0),
  stok_minimum INTEGER NOT NULL DEFAULT 0 CHECK(stok_minimum >= 0),
  foto TEXT,
  aktif INTEGER NOT NULL DEFAULT 1,
  dibuat_oleh INTEGER REFERENCES pengguna(id),
  dibuat_pada TEXT NOT NULL
) STRICT;

CREATE TABLE akun_kas(id INTEGER PRIMARY KEY, nama TEXT NOT NULL UNIQUE COLLATE NOCASE, aktif INTEGER NOT NULL DEFAULT 1) STRICT;

CREATE TABLE kategori_kas(id INTEGER PRIMARY KEY, nama TEXT NOT NULL UNIQUE COLLATE NOCASE, bawaan INTEGER NOT NULL DEFAULT 0) STRICT;

CREATE TABLE kiriman(
  id INTEGER PRIMARY KEY,
  barang_id INTEGER NOT NULL REFERENCES barang(id),
  tanggal_masuk TEXT NOT NULL,
  jumlah_awal INTEGER NOT NULL CHECK(jumlah_awal > 0),
  sisa INTEGER NOT NULL CHECK(sisa >= 0),
  harga_beli INTEGER NOT NULL CHECK(harga_beli >= 0),
  spek TEXT,
  kedaluwarsa TEXT,
  pemasok TEXT,
  foto TEXT,
  stok_awal INTEGER NOT NULL DEFAULT 0,
  akun_kas_id INTEGER REFERENCES akun_kas(id),
  batal INTEGER NOT NULL DEFAULT 0,
  alasan_batal TEXT,
  dibuat_oleh INTEGER REFERENCES pengguna(id),
  dibuat_pada TEXT NOT NULL
) STRICT;

CREATE TABLE transaksi_keluar(
  id INTEGER PRIMARY KEY,
  tanggal TEXT NOT NULL,
  akun_kas_id INTEGER NOT NULL REFERENCES akun_kas(id),
  catatan TEXT,
  batal INTEGER NOT NULL DEFAULT 0,
  alasan_batal TEXT,
  dibuat_oleh INTEGER REFERENCES pengguna(id),
  dibuat_pada TEXT NOT NULL
) STRICT;

CREATE TABLE baris_keluar(
  id INTEGER PRIMARY KEY,
  transaksi_id INTEGER NOT NULL REFERENCES transaksi_keluar(id),
  kiriman_id INTEGER NOT NULL REFERENCES kiriman(id),
  jumlah INTEGER NOT NULL CHECK(jumlah > 0),
  harga_jual INTEGER NOT NULL CHECK(harga_jual >= 0)
) STRICT;

CREATE TABLE penyesuaian_stok(
  id INTEGER PRIMARY KEY,
  kiriman_id INTEGER NOT NULL REFERENCES kiriman(id),
  tanggal TEXT NOT NULL,
  selisih INTEGER NOT NULL CHECK(selisih <> 0),
  alasan TEXT NOT NULL CHECK(alasan IN('rusak','kedaluwarsa','hilang','pakai_sendiri','koreksi')),
  catatan TEXT,
  batal INTEGER NOT NULL DEFAULT 0,
  alasan_batal TEXT,
  dibuat_oleh INTEGER REFERENCES pengguna(id),
  dibuat_pada TEXT NOT NULL
) STRICT;

CREATE TABLE kas(
  id INTEGER PRIMARY KEY,
  tanggal TEXT NOT NULL,
  arah TEXT NOT NULL CHECK(arah IN('masuk','keluar')),
  akun_kas_id INTEGER NOT NULL REFERENCES akun_kas(id),
  kategori_id INTEGER NOT NULL REFERENCES kategori_kas(id),
  jumlah INTEGER NOT NULL CHECK(jumlah > 0),
  keterangan TEXT,
  sumber TEXT NOT NULL CHECK(sumber IN('manual','otomatis')),
  ref_jenis TEXT CHECK(ref_jenis IN('kiriman','transaksi_keluar','pindah')),
  ref_id INTEGER,
  batal INTEGER NOT NULL DEFAULT 0,
  alasan_batal TEXT,
  dibuat_oleh INTEGER REFERENCES pengguna(id),
  dibuat_pada TEXT NOT NULL
) STRICT;

CREATE TABLE log_aktivitas(
  id INTEGER PRIMARY KEY,
  waktu TEXT NOT NULL,
  pengguna_id INTEGER REFERENCES pengguna(id),
  tindakan TEXT NOT NULL,
  ringkasan TEXT NOT NULL
) STRICT;

CREATE TRIGGER log_tak_diubah BEFORE UPDATE ON log_aktivitas
BEGIN SELECT RAISE(ABORT, 'log aktivitas tidak bisa diubah'); END;
CREATE TRIGGER log_tak_dihapus BEFORE DELETE ON log_aktivitas
BEGIN SELECT RAISE(ABORT, 'log aktivitas tidak bisa dihapus'); END;

CREATE INDEX idx_kiriman_barang ON kiriman(barang_id, batal, tanggal_masuk);
CREATE INDEX idx_kas_tanggal ON kas(tanggal);
CREATE INDEX idx_kas_ref ON kas(ref_jenis, ref_id);
CREATE INDEX idx_baris_kiriman ON baris_keluar(kiriman_id);
CREATE INDEX idx_baris_transaksi ON baris_keluar(transaksi_id);
CREATE INDEX idx_penyesuaian_kiriman ON penyesuaian_stok(kiriman_id);

INSERT INTO akun_kas(nama) VALUES('Tunai');
INSERT INTO kategori_kas(nama, bawaan) VALUES
  ('Penjualan',1),('Pembelian barang',1),('Pindah akun',1),
  ('Modal',0),('Listrik/Air',0),('Gaji',0),('Lain-lain',0);
INSERT INTO pengaturan(kunci, nilai) VALUES('kunci_otomatis_menit','10');
