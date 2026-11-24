//! Log aktivitas: hanya bisa ditambah (dijaga trigger), hanya pemilik yang bisa membaca.

use crate::akses::{wajib_pemilik, Sesi};
use crate::galat::Hasil;
use crate::waktu;
use rusqlite::{params, Connection};
use serde::Serialize;

pub fn catat(conn: &Connection, pengguna_id: Option<i64>, tindakan: &str, ringkasan: &str) -> Hasil<()> {
    conn.execute(
        "INSERT INTO log_aktivitas(waktu, pengguna_id, tindakan, ringkasan) VALUES(?1,?2,?3,?4)",
        params![waktu::kini(), pengguna_id, tindakan, ringkasan],
    )?;
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct BarisLog {
    pub id: i64,
    pub waktu: String,
    pub pengguna: Option<String>,
    pub tindakan: String,
    pub ringkasan: String,
}

pub fn daftar(conn: &Connection, s: &Sesi, dari: &str, sampai: &str) -> Hasil<Vec<BarisLog>> {
    wajib_pemilik(s)?;
    waktu::rentang(dari, sampai)?;
    let mut q = conn.prepare(
        "SELECT l.id, l.waktu, p.nama, l.tindakan, l.ringkasan
         FROM log_aktivitas l LEFT JOIN pengguna p ON p.id = l.pengguna_id
         WHERE substr(l.waktu,1,10) BETWEEN ?1 AND ?2 ORDER BY l.id DESC LIMIT 2000",
    )?;
    let v = q
        .query_map(params![dari, sampai], |r| {
            Ok(BarisLog { id: r.get(0)?, waktu: r.get(1)?, pengguna: r.get(2)?, tindakan: r.get(3)?, ringkasan: r.get(4)? })
        })?
        .collect::<Result<_, _>>()?;
    Ok(v)
}
