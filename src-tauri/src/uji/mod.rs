use crate::akses::{Modul, Sesi, Tingkat};
use rusqlite::Connection;

mod auth;
mod barang;
mod db;
mod batal;
mod kas;
mod keluar;
mod masuk;

/// DB memori dengan pemilik (id 1) dan staf (id 2), tanpa lewat Argon2.
pub fn db_uji() -> Connection {
    let c = crate::db::buka_memori();
    c.execute_batch(
        "INSERT INTO pengguna(id,nama_pengguna,nama,hash_sandi,peran,lihat_harga_beli,dibuat_pada)
           VALUES(1,'pemilik','Pemilik Uji','x','pemilik',1,'2026-01-01T00:00:00');
         INSERT INTO pengguna(id,nama_pengguna,nama,hash_sandi,peran,dibuat_pada)
           VALUES(2,'staf','Staf Uji','x','staf','2026-01-01T00:00:00');
         INSERT INTO pengaturan(kunci,nilai) VALUES('nama_toko','Toko Uji');",
    )
    .unwrap();
    c
}

pub fn pemilik() -> Sesi {
    Sesi::pemilik_uji()
}

pub fn staf(hak: &[(Modul, Tingkat)], harga: bool) -> Sesi {
    Sesi::staf_uji(hak, harga)
}

pub fn kasir() -> Sesi {
    Sesi::staf_uji(&[(Modul::Barang, Tingkat::Lihat), (Modul::BarangKeluar, Tingkat::Tambah)], false)
}

pub fn hitung(c: &Connection, sql: &str) -> i64 {
    c.query_row(sql, [], |r| r.get(0)).unwrap()
}

pub const HARI_INI: &str = "2026-11-24";

pub fn folder_data() -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    crate::jalur::siapkan(d.path()).unwrap();
    d
}

pub fn masukan_masuk(barang_id: i64, tanggal: &str, jumlah: i64, harga: i64) -> crate::stok::MasukMasukan {
    crate::stok::MasukMasukan {
        barang_id,
        tanggal_masuk: tanggal.into(),
        jumlah,
        harga_beli: harga,
        spek: None,
        kedaluwarsa: None,
        pemasok: None,
        foto_sumber: None,
        stok_awal: false,
        akun_kas_id: Some(1),
    }
}

/// Contoh di spec: Pakan Apung 781-2 dengan sisa 2 / 10 / 20 karung (03 Sep, 15 Okt, 20 Nov),
/// dimasukkan sebagai stok awal sehingga kas tetap kosong.
pub struct Skenario {
    pub c: Connection,
    pub d: tempfile::TempDir,
    pub barang: i64,
    pub sep: i64,
    pub okt: i64,
    pub nov: i64,
}

pub fn skenario() -> Skenario {
    let (c, d) = (db_uji(), folder_data());
    let barang = crate::barang::tambah(&c, d.path(), &pemilik(), barang::masukan("Pakan Apung 781-2")).unwrap();
    let mut id = Vec::new();
    for (tgl, jumlah, harga, spek, exp) in [
        ("2026-09-03", 2_000, 385_000, "protein 30%", "2027-01-31"),
        ("2026-10-15", 10_000, 392_000, "protein 31%", "2027-03-31"),
        ("2026-11-20", 20_000, 398_000, "protein 31%", "2027-05-31"),
    ] {
        let mut m = masukan_masuk(barang, tgl, jumlah, harga);
        m.spek = Some(spek.into());
        m.kedaluwarsa = Some(exp.into());
        m.stok_awal = true;
        m.akun_kas_id = None;
        id.push(crate::stok::masuk(&c, d.path(), &pemilik(), m).unwrap());
    }
    Skenario { c, d, barang, sep: id[0], okt: id[1], nov: id[2] }
}

pub fn sisa(c: &Connection, kiriman: i64) -> i64 {
    c.query_row("SELECT sisa FROM kiriman WHERE id=?1", [kiriman], |r| r.get(0)).unwrap()
}

pub fn jual(barang: i64, harga: i64, alokasi: &[(i64, i64)]) -> crate::stok::KeluarMasukan {
    crate::stok::KeluarMasukan {
        tanggal: HARI_INI.into(),
        akun_kas_id: 1,
        catatan: None,
        baris: vec![crate::stok::BarisKeluarMasukan {
            barang_id: barang,
            harga_jual: harga,
            alokasi: alokasi.iter().map(|&(kiriman_id, jumlah)| crate::stok::Alokasi { kiriman_id, jumlah }).collect(),
        }],
        izinkan_kedaluwarsa: false,
    }
}

/// Menjual 3 karung @ Rp 450.000 dari skenario: 2 dari Sep, 1 dari Okt.
pub fn jual_tiga(s: &Skenario) -> i64 {
    crate::stok::keluar(&s.c, &pemilik(), jual(s.barang, 450_000, &[(s.sep, 2_000), (s.okt, 1_000)]), HARI_INI).unwrap()
}
