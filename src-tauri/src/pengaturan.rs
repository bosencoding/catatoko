//! Pengaturan toko yang boleh diubah pemilik.

use crate::akses::{wajib_pemilik, Sesi};
use crate::galat::{Galat, Hasil};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;

pub const KUNCI_MENIT: &str = "kunci_otomatis_menit";

pub(crate) fn baca(conn: &Connection, kunci: &str) -> Hasil<Option<String>> {
    Ok(conn.query_row("SELECT nilai FROM pengaturan WHERE kunci=?1", [kunci], |r| r.get(0)).optional()?)
}

pub(crate) fn tulis(conn: &Connection, kunci: &str, nilai: &str) -> Hasil<()> {
    conn.execute(
        "INSERT INTO pengaturan(kunci,nilai) VALUES(?1,?2) ON CONFLICT(kunci) DO UPDATE SET nilai=excluded.nilai",
        params![kunci, nilai],
    )?;
    Ok(())
}

pub fn menit_kunci(conn: &Connection) -> u64 {
    baca(conn, KUNCI_MENIT).ok().flatten().and_then(|v| v.parse().ok()).unwrap_or(10)
}

#[derive(Debug, Serialize)]
pub struct Pengaturan {
    pub nama_toko: String,
    pub kunci_otomatis_menit: u64,
}

pub fn ambil(conn: &Connection) -> Hasil<Pengaturan> {
    Ok(Pengaturan { nama_toko: baca(conn, "nama_toko")?.unwrap_or_default(), kunci_otomatis_menit: menit_kunci(conn) })
}

pub fn simpan(conn: &Connection, s: &Sesi, kunci: &str, nilai: &str) -> Hasil<()> {
    wajib_pemilik(s)?;
    let nilai = nilai.trim();
    match kunci {
        "nama_toko" => {
            if nilai.is_empty() {
                return Err(Galat::validasi("Nama toko tidak boleh kosong."));
            }
        }
        KUNCI_MENIT => {
            let n: u64 = nilai.parse().map_err(|_| Galat::validasi("Menit kunci otomatis harus berupa angka."))?;
            if !(1..=120).contains(&n) {
                return Err(Galat::validasi("Kunci otomatis harus antara 1 dan 120 menit."));
            }
        }
        _ => return Err(Galat::validasi("Pengaturan ini tidak bisa diubah.")),
    }
    tulis(conn, kunci, nilai)?;
    crate::log::catat(conn, Some(s.pengguna_id), "pengaturan.simpan", &format!("{kunci} = {nilai}"))
}
