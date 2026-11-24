//! Stok per kiriman: barang masuk, barang keluar (stok lama dulu), pembatalan,
//! perbaikan kiriman, dan penyesuaian stok.

use crate::akses::{wajib, Modul, Sesi, Tingkat};
use crate::galat::{Galat, Hasil};
use crate::{foto, kas, log, waktu};
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Nilai Rupiah dari `jumlah` (per seperseribu satuan) × `harga` per satuan, dibulatkan.
pub(crate) fn nilai(jumlah: i64, harga: i64) -> i64 {
    (jumlah * harga + 500) / 1000
}

/// Menulis jumlah seperseribu sebagai teks: 2000 → "2", 500 → "0,5".
pub(crate) fn teks_jumlah(milli: i64) -> String {
    let (utuh, pecahan) = (milli / 1000, (milli % 1000).abs());
    if pecahan == 0 {
        return utuh.to_string();
    }
    let p = format!("{pecahan:03}");
    format!("{}{utuh},{}", if milli < 0 && utuh == 0 { "-" } else { "" }, p.trim_end_matches('0'))
}

fn rapikan(teks: Option<String>) -> Option<String> {
    teks.map(|t| t.trim().to_string()).filter(|t| !t.is_empty())
}

#[derive(Debug, Clone, Deserialize)]
pub struct MasukMasukan {
    pub barang_id: i64,
    pub tanggal_masuk: String,
    pub jumlah: i64,
    pub harga_beli: i64,
    pub spek: Option<String>,
    pub kedaluwarsa: Option<String>,
    pub pemasok: Option<String>,
    pub foto_sumber: Option<PathBuf>,
    /// Stok yang sudah ada sebelum aplikasi dipakai: tidak memengaruhi kas.
    pub stok_awal: bool,
    pub akun_kas_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Kiriman {
    pub id: i64,
    pub barang_id: i64,
    pub barang_nama: String,
    pub satuan: String,
    pub tanggal_masuk: String,
    pub jumlah_awal: i64,
    pub sisa: i64,
    /// `None` tanpa hak harga beli.
    pub harga_beli: Option<i64>,
    pub spek: Option<String>,
    pub kedaluwarsa: Option<String>,
    pub pemasok: Option<String>,
    pub foto: Option<String>,
    pub stok_awal: bool,
    pub batal: bool,
    pub alasan_batal: Option<String>,
    pub sudah_kedaluwarsa: bool,
    pub hari_ke_kedaluwarsa: Option<i64>,
    pub pernah_keluar: bool,
}

pub(crate) const PILIH_KIRIMAN: &str = "SELECT k.id, k.barang_id, b.nama, b.satuan, k.tanggal_masuk, k.jumlah_awal, k.sisa, k.harga_beli,
    k.spek, k.kedaluwarsa, k.pemasok, k.foto, k.stok_awal, k.batal, k.alasan_batal,
    (EXISTS(SELECT 1 FROM baris_keluar bk JOIN transaksi_keluar t ON t.id=bk.transaksi_id WHERE bk.kiriman_id=k.id AND t.batal=0)
     OR EXISTS(SELECT 1 FROM penyesuaian_stok p WHERE p.kiriman_id=k.id AND p.batal=0))
  FROM kiriman k JOIN barang b ON b.id=k.barang_id";

pub(crate) const URUT_KIRIMAN: &str = "k.tanggal_masuk, (k.kedaluwarsa IS NULL), k.kedaluwarsa, k.id";

pub(crate) fn kiriman_dari_baris(r: &Row, harga: bool, hari_ini: chrono::NaiveDate) -> rusqlite::Result<Kiriman> {
    let kedaluwarsa: Option<String> = r.get(9)?;
    let hari = kedaluwarsa.as_deref().and_then(|k| waktu::tanggal(k).ok()).map(|k| (k - hari_ini).num_days());
    Ok(Kiriman {
        id: r.get(0)?,
        barang_id: r.get(1)?,
        barang_nama: r.get(2)?,
        satuan: r.get(3)?,
        tanggal_masuk: r.get(4)?,
        jumlah_awal: r.get(5)?,
        sisa: r.get(6)?,
        harga_beli: if harga { Some(r.get(7)?) } else { None },
        spek: r.get(8)?,
        kedaluwarsa,
        pemasok: r.get(10)?,
        foto: r.get(11)?,
        stok_awal: r.get(12)?,
        batal: r.get(13)?,
        alasan_batal: r.get(14)?,
        sudah_kedaluwarsa: hari.is_some_and(|h| h < 0),
        hari_ke_kedaluwarsa: hari,
        pernah_keluar: r.get(15)?,
    })
}

/// (nama, satuan) barang yang masih aktif.
fn barang_aktif(conn: &Connection, id: i64) -> Hasil<(String, String)> {
    let baris: Option<(String, String, bool)> =
        conn.query_row("SELECT nama, satuan, aktif FROM barang WHERE id=?1", [id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).optional()?;
    match baris {
        None => Err(Galat::validasi("Barang tidak ditemukan.")),
        Some((nama, _, false)) => Err(Galat::validasi(format!("Barang '{nama}' sudah dinonaktifkan."))),
        Some((nama, satuan, true)) => Ok((nama, satuan)),
    }
}

fn periksa_kedaluwarsa(kedaluwarsa: &Option<String>, tanggal_masuk: chrono::NaiveDate) -> Hasil<()> {
    if let Some(k) = kedaluwarsa {
        if waktu::tanggal(k)? < tanggal_masuk {
            return Err(Galat::validasi("Tanggal kedaluwarsa tidak boleh sebelum tanggal masuk."));
        }
    }
    Ok(())
}

/// Mencatat barang masuk sebagai kiriman baru, dan kas keluar "Pembelian barang" bila bukan stok awal.
pub fn masuk(conn: &Connection, data: &Path, s: &Sesi, mut m: MasukMasukan) -> Hasil<i64> {
    wajib(s, Modul::BarangMasuk, Tingkat::Tambah)?;
    let tanggal = waktu::tanggal(&m.tanggal_masuk)?;
    if m.jumlah <= 0 {
        return Err(Galat::validasi("Jumlah barang harus lebih dari nol."));
    }
    if m.harga_beli < 0 {
        return Err(Galat::validasi("Harga beli tidak boleh negatif."));
    }
    let (nama, satuan) = barang_aktif(conn, m.barang_id)?;
    m.spek = rapikan(m.spek);
    m.pemasok = rapikan(m.pemasok);
    m.kedaluwarsa = rapikan(m.kedaluwarsa);
    periksa_kedaluwarsa(&m.kedaluwarsa, tanggal)?;
    let akun = if m.stok_awal {
        None
    } else {
        let akun = m.akun_kas_id.ok_or_else(|| Galat::validasi("Pilih akun kas yang dipakai membayar, atau centang 'Stok awal'."))?;
        kas::akun_aktif(conn, akun)?;
        Some(akun)
    };
    let foto_baru = m.foto_sumber.as_deref().map(|p| foto::simpan(data, p)).transpose()?;
    let hasil = (|| {
        let tx = conn.unchecked_transaction()?;
        tx.execute(
            "INSERT INTO kiriman(barang_id,tanggal_masuk,jumlah_awal,sisa,harga_beli,spek,kedaluwarsa,pemasok,foto,stok_awal,akun_kas_id,dibuat_oleh,dibuat_pada)
             VALUES(?1,?2,?3,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
            params![m.barang_id, m.tanggal_masuk, m.jumlah, m.harga_beli, m.spek, m.kedaluwarsa, m.pemasok, foto_baru, m.stok_awal, akun, s.pengguna_id, waktu::kini()],
        )?;
        let id = tx.last_insert_rowid();
        let total = nilai(m.jumlah, m.harga_beli);
        if let Some(akun) = akun.filter(|_| total > 0) {
            let ket = format!("Beli {} {satuan} {nama}", teks_jumlah(m.jumlah));
            kas::catat_otomatis(&tx, &m.tanggal_masuk, "keluar", akun, kas::KAT_PEMBELIAN, total, &ket, "kiriman", id, s.pengguna_id)?;
        }
        log::catat(
            &tx,
            Some(s.pengguna_id),
            "masuk.tambah",
            &format!(
                "Barang masuk #{id}: {} {satuan} {nama} @ {} tanggal {}{}",
                teks_jumlah(m.jumlah),
                m.harga_beli,
                m.tanggal_masuk,
                if m.stok_awal { " (stok awal)" } else { "" }
            ),
        )?;
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

fn wajib_lihat_kiriman(s: &Sesi) -> Hasil<()> {
    if s.boleh(Modul::BarangKeluar, Tingkat::Lihat) || s.boleh(Modul::BarangMasuk, Tingkat::Lihat) {
        return Ok(());
    }
    wajib(s, Modul::Barang, Tingkat::Lihat)
}

/// Kiriman satu barang, urut stok lama dulu. `hanya_bersisa` menyembunyikan yang habis atau batal.
pub fn kiriman_barang(conn: &Connection, s: &Sesi, barang_id: i64, hanya_bersisa: bool, hari_ini: &str) -> Hasil<Vec<Kiriman>> {
    wajib_lihat_kiriman(s)?;
    let hari = waktu::tanggal(hari_ini)?;
    let harga = s.boleh_harga_beli();
    let mut q = conn.prepare(&format!(
        "{PILIH_KIRIMAN} WHERE k.barang_id=?1 AND (?2=0 OR (k.batal=0 AND k.sisa>0)) ORDER BY {URUT_KIRIMAN}"
    ))?;
    let v = q.query_map(params![barang_id, hanya_bersisa], |r| kiriman_dari_baris(r, harga, hari))?.collect::<Result<_, _>>()?;
    Ok(v)
}

/// Riwayat barang masuk dalam rentang tanggal, terbaru dulu (termasuk yang batal).
pub fn daftar_masuk(conn: &Connection, s: &Sesi, dari: &str, sampai: &str) -> Hasil<Vec<Kiriman>> {
    wajib(s, Modul::BarangMasuk, Tingkat::Lihat)?;
    waktu::rentang(dari, sampai)?;
    let hari = waktu::tanggal(&waktu::hari_ini())?;
    let harga = s.boleh_harga_beli();
    let mut q = conn.prepare(&format!("{PILIH_KIRIMAN} WHERE k.tanggal_masuk BETWEEN ?1 AND ?2 ORDER BY k.tanggal_masuk DESC, k.id DESC"))?;
    let v = q.query_map(params![dari, sampai], |r| kiriman_dari_baris(r, harga, hari))?.collect::<Result<_, _>>()?;
    Ok(v)
}
