//! Pengelolaan akun dan hak akses. Semuanya khusus pemilik.

use crate::akses::{wajib_pemilik, Modul, Sesi, Tingkat};
use crate::auth;
use crate::galat::{Galat, Hasil};
use crate::{log, waktu};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct PenggunaMasukan {
    pub nama_pengguna: String,
    pub nama: String,
    pub peran: String,
    pub lihat_harga_beli: bool,
    pub hak: HashMap<Modul, Tingkat>,
    pub aktif: bool,
}

#[derive(Debug, Serialize)]
pub struct Pengguna {
    pub id: i64,
    pub nama_pengguna: String,
    pub nama: String,
    pub peran: String,
    pub lihat_harga_beli: bool,
    pub aktif: bool,
    pub hak: HashMap<Modul, Tingkat>,
}

/// Hak akun staf baru: kasir.
pub fn hak_bawaan_staf() -> HashMap<Modul, Tingkat> {
    Modul::SEMUA
        .into_iter()
        .map(|m| {
            let t = match m {
                Modul::Barang => Tingkat::Lihat,
                Modul::BarangKeluar => Tingkat::Tambah,
                _ => Tingkat::TidakAda,
            };
            (m, t)
        })
        .collect()
}

fn periksa(m: &mut PenggunaMasukan) -> Hasil<()> {
    m.nama_pengguna = m.nama_pengguna.trim().to_string();
    m.nama = m.nama.trim().to_string();
    if m.nama_pengguna.is_empty() || m.nama.is_empty() {
        return Err(Galat::validasi("Nama dan nama pengguna wajib diisi."));
    }
    match m.peran.as_str() {
        "pemilik" => {
            m.lihat_harga_beli = true;
            m.hak.clear();
        }
        "staf" => {
            let butuh = |modul| m.hak.get(&modul).copied().unwrap_or(Tingkat::TidakAda) > Tingkat::TidakAda;
            if !m.lihat_harga_beli && (butuh(Modul::BarangMasuk) || butuh(Modul::Laporan)) {
                return Err(Galat::validasi(
                    "Hak Barang Masuk dan Laporan memuat harga beli, jadi 'Lihat harga beli dan laba' harus ikut dicentang.",
                ));
            }
        }
        _ => return Err(Galat::validasi("Peran harus pemilik atau staf.")),
    }
    Ok(())
}

fn nama_dipakai(conn: &Connection, nama_pengguna: &str, kecuali: i64) -> Hasil<bool> {
    let n: i64 = conn.query_row("SELECT count(*) FROM pengguna WHERE nama_pengguna=?1 AND id<>?2", params![nama_pengguna, kecuali], |r| r.get(0))?;
    Ok(n > 0)
}

fn tulis_hak(conn: &Connection, id: i64, hak: &HashMap<Modul, Tingkat>) -> Hasil<()> {
    conn.execute("DELETE FROM hak_akses WHERE pengguna_id=?1", [id])?;
    for (m, t) in hak {
        conn.execute("INSERT INTO hak_akses(pengguna_id,modul,tingkat) VALUES(?1,?2,?3)", params![id, m.kunci(), *t as u8])?;
    }
    Ok(())
}

fn ringkas_hak(m: &PenggunaMasukan) -> String {
    let mut bagian: Vec<String> = Modul::SEMUA.iter().map(|k| format!("{}={}", k.kunci(), m.hak.get(k).map(|t| *t as u8).unwrap_or(0))).collect();
    bagian.push(format!("harga_beli={}", m.lihat_harga_beli as u8));
    bagian.join(", ")
}

pub fn daftar(conn: &Connection, s: &Sesi) -> Hasil<Vec<Pengguna>> {
    wajib_pemilik(s)?;
    let mut q = conn.prepare("SELECT id FROM pengguna ORDER BY peran, nama")?;
    let id: Vec<i64> = q.query_map([], |r| r.get(0))?.collect::<Result<_, _>>()?;
    id.into_iter()
        .map(|id| {
            let aktif: bool = conn.query_row("SELECT aktif FROM pengguna WHERE id=?1", [id], |r| r.get(0))?;
            let ses = auth::muat_sesi(conn, id)?;
            Ok(Pengguna {
                id,
                nama_pengguna: ses.nama_pengguna,
                nama: ses.nama,
                peran: if ses.pemilik { "pemilik".into() } else { "staf".into() },
                lihat_harga_beli: ses.lihat_harga_beli,
                aktif,
                hak: ses.hak,
            })
        })
        .collect()
}

pub fn tambah(conn: &Connection, s: &Sesi, mut m: PenggunaMasukan, sandi: &str) -> Hasil<i64> {
    wajib_pemilik(s)?;
    periksa(&mut m)?;
    auth::periksa_sandi_baru(sandi)?;
    if nama_dipakai(conn, &m.nama_pengguna, 0)? {
        return Err(Galat::validasi(format!("Nama pengguna '{}' sudah dipakai.", m.nama_pengguna)));
    }
    let h = auth::hash(sandi)?;
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO pengguna(nama_pengguna,nama,hash_sandi,peran,lihat_harga_beli,aktif,dibuat_pada) VALUES(?1,?2,?3,?4,?5,?6,?7)",
        params![m.nama_pengguna, m.nama, h, m.peran, m.lihat_harga_beli, m.aktif, waktu::kini()],
    )?;
    let id = tx.last_insert_rowid();
    tulis_hak(&tx, id, &m.hak)?;
    log::catat(&tx, Some(s.pengguna_id), "pengguna.tambah", &format!("Akun {} '{}' dibuat ({})", m.peran, m.nama_pengguna, ringkas_hak(&m)))?;
    tx.commit()?;
    Ok(id)
}

pub fn ubah(conn: &Connection, s: &Sesi, id: i64, mut m: PenggunaMasukan) -> Hasil<()> {
    wajib_pemilik(s)?;
    periksa(&mut m)?;
    let (peran_lama, aktif_lama): (String, bool) =
        conn.query_row("SELECT peran, aktif FROM pengguna WHERE id=?1", [id], |r| Ok((r.get(0)?, r.get(1)?))).map_err(|_| Galat::tidak_ditemukan("Pengguna"))?;
    if nama_dipakai(conn, &m.nama_pengguna, id)? {
        return Err(Galat::validasi(format!("Nama pengguna '{}' sudah dipakai.", m.nama_pengguna)));
    }
    let tetap_pemilik_aktif = m.peran == "pemilik" && m.aktif;
    if peran_lama == "pemilik" && aktif_lama && !tetap_pemilik_aktif {
        let lain: i64 = conn.query_row("SELECT count(*) FROM pengguna WHERE peran='pemilik' AND aktif=1 AND id<>?1", [id], |r| r.get(0))?;
        if lain == 0 {
            return Err(Galat::validasi("Harus selalu ada minimal satu akun pemilik yang aktif."));
        }
    }
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE pengguna SET nama_pengguna=?1, nama=?2, peran=?3, lihat_harga_beli=?4, aktif=?5 WHERE id=?6",
        params![m.nama_pengguna, m.nama, m.peran, m.lihat_harga_beli, m.aktif, id],
    )?;
    tulis_hak(&tx, id, &m.hak)?;
    log::catat(
        &tx,
        Some(s.pengguna_id),
        "pengguna.ubah",
        &format!("Akun '{}' diubah: peran {}, aktif {} ({})", m.nama_pengguna, m.peran, m.aktif, ringkas_hak(&m)),
    )?;
    tx.commit()?;
    Ok(())
}

pub fn atur_ulang_sandi(conn: &Connection, s: &Sesi, id: i64, sandi_baru: &str) -> Hasil<()> {
    wajib_pemilik(s)?;
    auth::periksa_sandi_baru(sandi_baru)?;
    let n = conn.execute(
        "UPDATE pengguna SET hash_sandi=?1, gagal_login=0, ditahan_sampai=NULL WHERE id=?2",
        params![auth::hash(sandi_baru)?, id],
    )?;
    if n == 0 {
        return Err(Galat::tidak_ditemukan("Pengguna"));
    }
    log::catat(conn, Some(s.pengguna_id), "pengguna.atur_ulang_sandi", &format!("Kata sandi pengguna #{id} diatur ulang"))
}
