//! Barang dan kategorinya. Barang tidak pernah dihapus, hanya dinonaktifkan.

use crate::akses::{wajib, Modul, Sesi, Tingkat};
use crate::galat::{Galat, Hasil};
use crate::{foto, log, waktu};
use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub struct BarangMasukan {
    /// Kosong = dibuat otomatis (B0001, B0002, …).
    pub kode: String,
    pub nama: String,
    pub kategori_id: Option<i64>,
    pub satuan: String,
    pub harga_jual: i64,
    pub stok_minimum: i64,
    /// File foto baru. `None` saat mengubah = foto lama dipertahankan.
    pub foto_sumber: Option<PathBuf>,
    pub aktif: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Barang {
    pub id: i64,
    pub kode: String,
    pub nama: String,
    pub kategori_id: Option<i64>,
    pub kategori: Option<String>,
    pub satuan: String,
    pub harga_jual: i64,
    pub stok_minimum: i64,
    pub foto: Option<String>,
    pub aktif: bool,
    /// Jumlah sisa semua kiriman yang tidak batal (per seperseribu satuan).
    pub stok: i64,
    /// Nilai stok menurut harga beli. `None` tanpa hak harga beli.
    pub nilai_stok: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct Kategori {
    pub id: i64,
    pub nama: String,
}

const PILIH: &str = "SELECT b.id, b.kode, b.nama, b.kategori_id, k.nama, b.satuan, b.harga_jual, b.stok_minimum, b.foto, b.aktif,
    COALESCE((SELECT SUM(sisa) FROM kiriman WHERE barang_id=b.id AND batal=0), 0),
    COALESCE((SELECT SUM((sisa * harga_beli + 500) / 1000) FROM kiriman WHERE barang_id=b.id AND batal=0), 0)
  FROM barang b LEFT JOIN kategori_barang k ON k.id = b.kategori_id";

fn dari_baris(r: &Row, harga: bool) -> rusqlite::Result<Barang> {
    Ok(Barang {
        id: r.get(0)?,
        kode: r.get(1)?,
        nama: r.get(2)?,
        kategori_id: r.get(3)?,
        kategori: r.get(4)?,
        satuan: r.get(5)?,
        harga_jual: r.get(6)?,
        stok_minimum: r.get(7)?,
        foto: r.get(8)?,
        aktif: r.get(9)?,
        stok: r.get(10)?,
        nilai_stok: if harga { Some(r.get(11)?) } else { None },
    })
}

/// Melihat daftar barang boleh bagi yang punya hak Barang, atau yang perlu memilih barang
/// saat mencatat barang masuk/keluar.
fn wajib_lihat(s: &Sesi) -> Hasil<()> {
    if s.boleh(Modul::BarangKeluar, Tingkat::Lihat) || s.boleh(Modul::BarangMasuk, Tingkat::Lihat) {
        return Ok(());
    }
    wajib(s, Modul::Barang, Tingkat::Lihat)
}

pub fn daftar(conn: &Connection, s: &Sesi, cari: Option<&str>, termasuk_nonaktif: bool) -> Hasil<Vec<Barang>> {
    wajib_lihat(s)?;
    let pola = format!("%{}%", cari.unwrap_or("").trim());
    let mut q = conn.prepare(&format!(
        "{PILIH} WHERE (b.aktif=1 OR ?1) AND (b.nama LIKE ?2 OR b.kode LIKE ?2) ORDER BY b.nama COLLATE NOCASE"
    ))?;
    let harga = s.boleh_harga_beli();
    let v = q.query_map(params![termasuk_nonaktif, pola], |r| dari_baris(r, harga))?.collect::<Result<_, _>>()?;
    Ok(v)
}

pub fn ambil(conn: &Connection, s: &Sesi, id: i64) -> Hasil<Barang> {
    wajib_lihat(s)?;
    let harga = s.boleh_harga_beli();
    conn.query_row(&format!("{PILIH} WHERE b.id=?1"), [id], |r| dari_baris(r, harga)).map_err(|_| Galat::tidak_ditemukan("Barang"))
}

fn kode_otomatis(conn: &Connection) -> Hasil<String> {
    let mut n: i64 = conn.query_row("SELECT count(*) FROM barang", [], |r| r.get(0))?;
    loop {
        n += 1;
        let kode = format!("B{n:04}");
        if !kode_dipakai(conn, &kode, 0)? {
            return Ok(kode);
        }
    }
}

fn kode_dipakai(conn: &Connection, kode: &str, kecuali: i64) -> Hasil<bool> {
    let n: i64 = conn.query_row("SELECT count(*) FROM barang WHERE kode=?1 AND id<>?2", params![kode, kecuali], |r| r.get(0))?;
    Ok(n > 0)
}

fn periksa(conn: &Connection, m: &mut BarangMasukan, id: i64) -> Hasil<()> {
    m.kode = m.kode.trim().to_string();
    m.nama = m.nama.trim().to_string();
    m.satuan = m.satuan.trim().to_string();
    if m.nama.is_empty() {
        return Err(Galat::validasi("Nama barang wajib diisi."));
    }
    if m.satuan.is_empty() {
        return Err(Galat::validasi("Satuan wajib diisi, misalnya karung, kg, atau buah."));
    }
    if m.harga_jual < 0 || m.stok_minimum < 0 {
        return Err(Galat::validasi("Harga jual dan stok minimum tidak boleh negatif."));
    }
    if let Some(k) = m.kategori_id {
        let ada: i64 = conn.query_row("SELECT count(*) FROM kategori_barang WHERE id=?1", [k], |r| r.get(0))?;
        if ada == 0 {
            return Err(Galat::validasi("Kategori barang tidak ditemukan."));
        }
    }
    if m.kode.is_empty() {
        m.kode = kode_otomatis(conn)?;
    } else if kode_dipakai(conn, &m.kode, id)? {
        return Err(Galat::validasi(format!("Kode barang '{}' sudah dipakai barang lain.", m.kode)));
    }
    Ok(())
}

pub fn tambah(conn: &Connection, data: &Path, s: &Sesi, mut m: BarangMasukan) -> Hasil<i64> {
    wajib(s, Modul::Barang, Tingkat::Tambah)?;
    periksa(conn, &mut m, 0)?;
    let foto_baru = m.foto_sumber.as_deref().map(|p| foto::simpan(data, p)).transpose()?;
    let hasil = (|| {
        let tx = conn.unchecked_transaction()?;
        tx.execute(
            "INSERT INTO barang(kode,nama,kategori_id,satuan,harga_jual,stok_minimum,foto,aktif,dibuat_oleh,dibuat_pada) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
            params![m.kode, m.nama, m.kategori_id, m.satuan, m.harga_jual, m.stok_minimum, foto_baru, m.aktif, s.pengguna_id, waktu::kini()],
        )?;
        let id = tx.last_insert_rowid();
        log::catat(&tx, Some(s.pengguna_id), "barang.tambah", &format!("Barang {} '{}' ditambahkan", m.kode, m.nama))?;
        tx.commit()?;
        Ok(id)
    })();
    if hasil.is_err() {
        if let Some(f) = &foto_baru {
            foto::hapus(data, f);
        }
    }
    hasil
}

pub fn ubah(conn: &Connection, data: &Path, s: &Sesi, id: i64, mut m: BarangMasukan) -> Hasil<()> {
    wajib(s, Modul::Barang, Tingkat::Ubah)?;
    let foto_lama: Option<String> = conn.query_row("SELECT foto FROM barang WHERE id=?1", [id], |r| r.get(0)).map_err(|_| Galat::tidak_ditemukan("Barang"))?;
    periksa(conn, &mut m, id)?;
    let foto_baru = m.foto_sumber.as_deref().map(|p| foto::simpan(data, p)).transpose()?;
    let foto_akhir = foto_baru.clone().or(foto_lama.clone());
    let hasil = (|| {
        let tx = conn.unchecked_transaction()?;
        tx.execute(
            "UPDATE barang SET kode=?1,nama=?2,kategori_id=?3,satuan=?4,harga_jual=?5,stok_minimum=?6,foto=?7,aktif=?8 WHERE id=?9",
            params![m.kode, m.nama, m.kategori_id, m.satuan, m.harga_jual, m.stok_minimum, foto_akhir, m.aktif, id],
        )?;
        log::catat(
            &tx,
            Some(s.pengguna_id),
            "barang.ubah",
            &format!("Barang {} '{}' diubah (harga jual {}, aktif {})", m.kode, m.nama, m.harga_jual, m.aktif),
        )?;
        tx.commit()?;
        Ok(())
    })();
    match (&hasil, &foto_baru) {
        (Ok(()), Some(_)) => {
            if let Some(l) = &foto_lama {
                foto::hapus(data, l);
            }
        }
        (Err(_), Some(f)) => foto::hapus(data, f),
        _ => {}
    }
    hasil
}

pub fn kategori_daftar(conn: &Connection, s: &Sesi) -> Hasil<Vec<Kategori>> {
    wajib_lihat(s)?;
    let mut q = conn.prepare("SELECT id, nama FROM kategori_barang ORDER BY nama COLLATE NOCASE")?;
    let v = q.query_map([], |r| Ok(Kategori { id: r.get(0)?, nama: r.get(1)? }))?.collect::<Result<_, _>>()?;
    Ok(v)
}

pub fn kategori_tambah(conn: &Connection, s: &Sesi, nama: &str) -> Hasil<i64> {
    wajib(s, Modul::Barang, Tingkat::Tambah)?;
    let nama = nama.trim();
    if nama.is_empty() {
        return Err(Galat::validasi("Nama kategori wajib diisi."));
    }
    let ada: i64 = conn.query_row("SELECT count(*) FROM kategori_barang WHERE nama=?1", [nama], |r| r.get(0))?;
    if ada > 0 {
        return Err(Galat::validasi(format!("Kategori '{nama}' sudah ada.")));
    }
    conn.execute("INSERT INTO kategori_barang(nama) VALUES(?1)", [nama])?;
    Ok(conn.last_insert_rowid())
}
