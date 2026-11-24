use crate::akses::{Modul, Sesi, Tingkat};
use rusqlite::Connection;

mod auth;
mod barang;
mod db;

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
