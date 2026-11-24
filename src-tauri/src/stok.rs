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

/// Barang masuk yang paling baru DICATAT (bukan menurut tanggal masuk), termasuk yang batal.
pub fn masuk_terbaru(conn: &Connection, s: &Sesi, batas: u32) -> Hasil<Vec<Kiriman>> {
    wajib(s, Modul::BarangMasuk, Tingkat::Lihat)?;
    let hari = waktu::tanggal(&waktu::hari_ini())?;
    let harga = s.boleh_harga_beli();
    let mut q = conn.prepare(&format!("{PILIH_KIRIMAN} ORDER BY k.id DESC LIMIT ?1"))?;
    let v = q.query_map([batas.min(500)], |r| kiriman_dari_baris(r, harga, hari))?.collect::<Result<_, _>>()?;
    Ok(v)
}

// ---------------------------------------------------------------------------
// Barang keluar
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Alokasi {
    pub kiriman_id: i64,
    pub jumlah: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BarisKeluarMasukan {
    pub barang_id: i64,
    pub harga_jual: i64,
    pub alokasi: Vec<Alokasi>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KeluarMasukan {
    pub tanggal: String,
    pub akun_kas_id: i64,
    pub catatan: Option<String>,
    pub baris: Vec<BarisKeluarMasukan>,
    /// Harus `true` bila ada alokasi dari kiriman yang sudah kedaluwarsa.
    pub izinkan_kedaluwarsa: bool,
}

#[derive(Debug, Serialize)]
pub struct RincianKeluar {
    pub barang_id: i64,
    pub barang_nama: String,
    pub satuan: String,
    pub kiriman_id: i64,
    pub tanggal_masuk: String,
    pub spek: Option<String>,
    pub jumlah: i64,
    pub harga_jual: i64,
    pub subtotal: i64,
    pub harga_beli: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct TransaksiKeluar {
    pub id: i64,
    pub tanggal: String,
    pub akun: String,
    pub catatan: Option<String>,
    pub batal: bool,
    pub alasan_batal: Option<String>,
    pub total: i64,
    /// `None` tanpa hak harga beli.
    pub laba: Option<i64>,
    pub dibuat_oleh_nama: Option<String>,
    pub baris: Vec<RincianKeluar>,
}

/// Mengusulkan kiriman yang diambil: tanggal masuk terlama dulu, melewati yang sudah kedaluwarsa.
pub fn usul_alokasi(conn: &Connection, s: &Sesi, barang_id: i64, jumlah: i64, hari_ini: &str) -> Hasil<Vec<Alokasi>> {
    wajib(s, Modul::BarangKeluar, Tingkat::Lihat)?;
    waktu::tanggal(hari_ini)?;
    if jumlah <= 0 {
        return Err(Galat::validasi("Jumlah barang harus lebih dari nol."));
    }
    let (nama, satuan) = barang_aktif(conn, barang_id)?;
    let mut q = conn.prepare(&format!(
        "SELECT k.id, k.sisa, (k.kedaluwarsa IS NOT NULL AND k.kedaluwarsa < ?2) FROM kiriman k
         WHERE k.barang_id=?1 AND k.batal=0 AND k.sisa>0 ORDER BY {URUT_KIRIMAN}"
    ))?;
    let kiriman: Vec<(i64, i64, bool)> = q.query_map(params![barang_id, hari_ini], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?.collect::<Result<_, _>>()?;
    let (mut usul, mut kurang, mut tersedia, mut basi) = (Vec::new(), jumlah, 0, 0);
    for (id, sisa, kedaluwarsa) in kiriman {
        if kedaluwarsa {
            basi += sisa;
            continue;
        }
        tersedia += sisa;
        if kurang > 0 {
            let ambil = kurang.min(sisa);
            usul.push(Alokasi { kiriman_id: id, jumlah: ambil });
            kurang -= ambil;
        }
    }
    if kurang > 0 {
        let mut pesan = format!("Stok {nama} tidak cukup: diminta {} {satuan}, tersedia {} {satuan}.", teks_jumlah(jumlah), teks_jumlah(tersedia));
        if basi > 0 {
            pesan.push_str(&format!(" Ada {} {satuan} lagi yang sudah kedaluwarsa; pilih lewat 'Ganti kiriman' bila tetap ingin dikeluarkan.", teks_jumlah(basi)));
        }
        return Err(Galat::baru("STOK_KURANG", pesan));
    }
    Ok(usul)
}

/// Mencatat barang keluar. Semuanya tersimpan atau tidak sama sekali.
pub fn keluar(conn: &Connection, s: &Sesi, m: KeluarMasukan, hari_ini: &str) -> Hasil<i64> {
    wajib(s, Modul::BarangKeluar, Tingkat::Tambah)?;
    let tanggal = waktu::tanggal(&m.tanggal)?;
    waktu::tanggal(hari_ini)?;
    kas::akun_aktif(conn, m.akun_kas_id)?;
    if m.baris.is_empty() {
        return Err(Galat::validasi("Belum ada barang yang dipilih."));
    }
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO transaksi_keluar(tanggal,akun_kas_id,catatan,dibuat_oleh,dibuat_pada) VALUES(?1,?2,?3,?4,?5)",
        params![m.tanggal, m.akun_kas_id, rapikan(m.catatan.clone()), s.pengguna_id, waktu::kini()],
    )?;
    let id = tx.last_insert_rowid();
    let (mut total, mut ringkas) = (0i64, Vec::new());
    for baris in &m.baris {
        let (nama, satuan) = barang_aktif(&tx, baris.barang_id)?;
        if baris.harga_jual < 0 {
            return Err(Galat::validasi(format!("Harga jual {nama} tidak boleh negatif.")));
        }
        if baris.alokasi.is_empty() {
            return Err(Galat::validasi(format!("Jumlah {nama} belum diisi.")));
        }
        let mut jumlah_baris = 0;
        for a in &baris.alokasi {
            if a.jumlah <= 0 {
                return Err(Galat::validasi(format!("Jumlah {nama} harus lebih dari nol.")));
            }
            let k: Option<(i64, i64, bool, String, Option<String>)> = tx
                .query_row("SELECT barang_id, sisa, batal, tanggal_masuk, kedaluwarsa FROM kiriman WHERE id=?1", [a.kiriman_id], |r| {
                    Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
                })
                .optional()?;
            let Some((barang_kiriman, sisa, batal, tanggal_masuk, kedaluwarsa)) = k else {
                return Err(Galat::validasi(format!("Kiriman {nama} yang dipilih tidak ditemukan. Muat ulang lalu pilih lagi.")));
            };
            if barang_kiriman != baris.barang_id {
                return Err(Galat::validasi(format!("Kiriman yang dipilih bukan milik {nama}.")));
            }
            if batal {
                return Err(Galat::validasi(format!("Kiriman {nama} tanggal {tanggal_masuk} sudah dibatalkan.")));
            }
            if tanggal < waktu::tanggal(&tanggal_masuk)? {
                return Err(Galat::validasi(format!("Tanggal keluar tidak boleh sebelum kiriman {nama} masuk ({tanggal_masuk}).")));
            }
            if !m.izinkan_kedaluwarsa && kedaluwarsa.as_deref().is_some_and(|k| k < hari_ini) {
                return Err(Galat::validasi(format!("Kiriman {nama} tanggal {tanggal_masuk} sudah kedaluwarsa. Konfirmasi dulu bila tetap ingin dikeluarkan.")));
            }
            if a.jumlah > sisa {
                return Err(Galat::baru(
                    "STOK_KURANG",
                    format!("Stok {nama} dari kiriman {tanggal_masuk} tinggal {} {satuan}, diminta {} {satuan}.", teks_jumlah(sisa), teks_jumlah(a.jumlah)),
                ));
            }
            tx.execute("UPDATE kiriman SET sisa=sisa-?1 WHERE id=?2", params![a.jumlah, a.kiriman_id])?;
            tx.execute("INSERT INTO baris_keluar(transaksi_id,kiriman_id,jumlah,harga_jual) VALUES(?1,?2,?3,?4)", params![id, a.kiriman_id, a.jumlah, baris.harga_jual])?;
            total += nilai(a.jumlah, baris.harga_jual);
            jumlah_baris += a.jumlah;
        }
        ringkas.push(format!("{} {satuan} {nama} @ {}", teks_jumlah(jumlah_baris), kas::rupiah(baris.harga_jual)));
    }
    if total > 0 {
        kas::catat_otomatis(&tx, &m.tanggal, "masuk", m.akun_kas_id, kas::KAT_PENJUALAN, total, &format!("Jual {}", ringkas.join(", ")), "transaksi_keluar", id, s.pengguna_id)?;
    }
    log::catat(&tx, Some(s.pengguna_id), "keluar.tambah", &format!("Barang keluar #{id} tanggal {}: {} (total {total})", m.tanggal, ringkas.join(", ")))?;
    tx.commit()?;
    Ok(id)
}

/// Riwayat barang keluar dalam rentang tanggal, terbaru dulu (termasuk yang batal).
pub fn daftar_keluar(conn: &Connection, s: &Sesi, dari: &str, sampai: &str) -> Hasil<Vec<TransaksiKeluar>> {
    wajib(s, Modul::BarangKeluar, Tingkat::Lihat)?;
    waktu::rentang(dari, sampai)?;
    muat_transaksi(conn, s, "WHERE t.tanggal BETWEEN ?1 AND ?2 ORDER BY t.tanggal DESC, t.id DESC", params![dari, sampai])
}

/// Barang keluar yang paling baru DICATAT (bukan menurut tanggal transaksi), termasuk yang batal.
pub fn keluar_terbaru(conn: &Connection, s: &Sesi, batas: u32) -> Hasil<Vec<TransaksiKeluar>> {
    wajib(s, Modul::BarangKeluar, Tingkat::Lihat)?;
    muat_transaksi(conn, s, "ORDER BY t.id DESC LIMIT ?1", params![batas.min(500)])
}

fn muat_transaksi(conn: &Connection, s: &Sesi, ekor: &str, p: &[&dyn rusqlite::ToSql]) -> Hasil<Vec<TransaksiKeluar>> {
    let harga = s.boleh_harga_beli();
    let mut q = conn.prepare(&format!(
        "SELECT t.id, t.tanggal, a.nama, t.catatan, t.batal, t.alasan_batal, p.nama
         FROM transaksi_keluar t JOIN akun_kas a ON a.id=t.akun_kas_id LEFT JOIN pengguna p ON p.id=t.dibuat_oleh {ekor}"
    ))?;
    let mut hasil: Vec<TransaksiKeluar> = q
        .query_map(p, |r| {
            Ok(TransaksiKeluar {
                id: r.get(0)?,
                tanggal: r.get(1)?,
                akun: r.get(2)?,
                catatan: r.get(3)?,
                batal: r.get(4)?,
                alasan_batal: r.get(5)?,
                total: 0,
                laba: None,
                dibuat_oleh_nama: r.get(6)?,
                baris: Vec::new(),
            })
        })?
        .collect::<Result<_, _>>()?;
    let mut qb = conn.prepare(
        "SELECT b.id, b.nama, b.satuan, k.id, k.tanggal_masuk, k.spek, bk.jumlah, bk.harga_jual, k.harga_beli
         FROM baris_keluar bk JOIN kiriman k ON k.id=bk.kiriman_id JOIN barang b ON b.id=k.barang_id
         WHERE bk.transaksi_id=?1 ORDER BY bk.id",
    )?;
    for t in &mut hasil {
        let mut pokok = 0;
        let baris = qb.query_map([t.id], |r| {
            let (jumlah, harga_jual, harga_beli): (i64, i64, i64) = (r.get(6)?, r.get(7)?, r.get(8)?);
            Ok((
                RincianKeluar {
                    barang_id: r.get(0)?,
                    barang_nama: r.get(1)?,
                    satuan: r.get(2)?,
                    kiriman_id: r.get(3)?,
                    tanggal_masuk: r.get(4)?,
                    spek: r.get(5)?,
                    jumlah,
                    harga_jual,
                    subtotal: nilai(jumlah, harga_jual),
                    harga_beli: harga.then_some(harga_beli),
                },
                nilai(jumlah, harga_beli),
            ))
        })?;
        for b in baris {
            let (rinci, hp) = b?;
            t.total += rinci.subtotal;
            pokok += hp;
            t.baris.push(rinci);
        }
        t.laba = harga.then_some(t.total - pokok);
    }
    Ok(hasil)
}

// ---------------------------------------------------------------------------
// Pembatalan, perbaikan kiriman, penyesuaian stok
// ---------------------------------------------------------------------------

const ALASAN_PENYESUAIAN: [&str; 5] = ["rusak", "kedaluwarsa", "hilang", "pakai_sendiri", "koreksi"];

/// Membatalkan barang keluar: stok kembali ke kirimannya, kas penjualannya ikut batal.
pub fn batal_keluar(conn: &Connection, s: &Sesi, id: i64, alasan: &str) -> Hasil<()> {
    wajib(s, Modul::BarangKeluar, Tingkat::Ubah)?;
    let alasan = kas::periksa_alasan(alasan)?;
    let batal: bool = conn.query_row("SELECT batal FROM transaksi_keluar WHERE id=?1", [id], |r| r.get(0)).map_err(|_| Galat::tidak_ditemukan("Transaksi barang keluar"))?;
    if batal {
        return Err(Galat::validasi("Transaksi ini sudah dibatalkan."));
    }
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE kiriman SET sisa = sisa + (SELECT SUM(jumlah) FROM baris_keluar WHERE transaksi_id=?1 AND kiriman_id=kiriman.id)
         WHERE id IN (SELECT kiriman_id FROM baris_keluar WHERE transaksi_id=?1)",
        [id],
    )?;
    tx.execute("UPDATE transaksi_keluar SET batal=1, alasan_batal=?1 WHERE id=?2", params![alasan, id])?;
    kas::batal_otomatis(&tx, "transaksi_keluar", id)?;
    log::catat(&tx, Some(s.pengguna_id), "keluar.batal", &format!("Barang keluar #{id} dibatalkan: {alasan}"))?;
    tx.commit()?;
    Ok(())
}

/// (batal, pernah_keluar, stok_awal, jumlah_awal, sisa, harga_beli, tanggal_masuk, akun_kas_id, foto, nama barang, satuan)
struct KepalaKiriman {
    batal: bool,
    pernah_keluar: bool,
    stok_awal: bool,
    jumlah_awal: i64,
    sisa: i64,
    harga_beli: i64,
    tanggal_masuk: String,
    akun_kas_id: Option<i64>,
    foto: Option<String>,
    spek: Option<String>,
    kedaluwarsa: Option<String>,
    pemasok: Option<String>,
    nama: String,
    satuan: String,
}

fn kepala_kiriman(conn: &Connection, id: i64) -> Hasil<KepalaKiriman> {
    conn.query_row(
        "SELECT k.batal,
           (EXISTS(SELECT 1 FROM baris_keluar bk JOIN transaksi_keluar t ON t.id=bk.transaksi_id WHERE bk.kiriman_id=k.id AND t.batal=0)
            OR EXISTS(SELECT 1 FROM penyesuaian_stok p WHERE p.kiriman_id=k.id AND p.batal=0)),
           k.stok_awal, k.jumlah_awal, k.sisa, k.harga_beli, k.tanggal_masuk, k.akun_kas_id, k.foto, k.spek, k.kedaluwarsa, k.pemasok, b.nama, b.satuan
         FROM kiriman k JOIN barang b ON b.id=k.barang_id WHERE k.id=?1",
        [id],
        |r| {
            Ok(KepalaKiriman {
                batal: r.get(0)?,
                pernah_keluar: r.get(1)?,
                stok_awal: r.get(2)?,
                jumlah_awal: r.get(3)?,
                sisa: r.get(4)?,
                harga_beli: r.get(5)?,
                tanggal_masuk: r.get(6)?,
                akun_kas_id: r.get(7)?,
                foto: r.get(8)?,
                spek: r.get(9)?,
                kedaluwarsa: r.get(10)?,
                pemasok: r.get(11)?,
                nama: r.get(12)?,
                satuan: r.get(13)?,
            })
        },
    )
    .map_err(|_| Galat::tidak_ditemukan("Kiriman"))
}

/// Membatalkan barang masuk. Hanya bila kirimannya belum pernah keluar atau disesuaikan.
pub fn batal_masuk(conn: &Connection, s: &Sesi, kiriman_id: i64, alasan: &str) -> Hasil<()> {
    wajib(s, Modul::BarangMasuk, Tingkat::Ubah)?;
    let alasan = kas::periksa_alasan(alasan)?;
    let k = kepala_kiriman(conn, kiriman_id)?;
    if k.batal {
        return Err(Galat::validasi("Barang masuk ini sudah dibatalkan."));
    }
    if k.pernah_keluar {
        return Err(Galat::validasi(
            "Kiriman ini sudah pernah keluar atau disesuaikan, jadi tidak bisa dibatalkan. Batalkan dulu transaksi keluarnya, atau gunakan 'Perbaiki kiriman' / 'Penyesuaian stok'.",
        ));
    }
    let tx = conn.unchecked_transaction()?;
    tx.execute("UPDATE kiriman SET batal=1, sisa=0, alasan_batal=?1 WHERE id=?2", params![alasan, kiriman_id])?;
    kas::batal_otomatis(&tx, "kiriman", kiriman_id)?;
    log::catat(
        &tx,
        Some(s.pengguna_id),
        "masuk.batal",
        &format!("Barang masuk #{kiriman_id} ({} {} {}) dibatalkan: {alasan}", teks_jumlah(k.jumlah_awal), k.satuan, k.nama),
    )?;
    tx.commit()?;
    Ok(())
}

/// Nilai baru untuk kiriman. Semua field menggantikan nilai lama (kosong = dihapus), kecuali foto.
#[derive(Debug, Clone, Deserialize)]
pub struct PerbaikanKiriman {
    pub harga_beli: i64,
    pub spek: Option<String>,
    pub kedaluwarsa: Option<String>,
    pub pemasok: Option<String>,
    pub foto_sumber: Option<PathBuf>,
}

fn beda(nama: &str, lama: &Option<String>, baru: &Option<String>, catatan: &mut Vec<String>) {
    if lama != baru {
        catatan.push(format!("{nama} {} → {}", lama.as_deref().unwrap_or("-"), baru.as_deref().unwrap_or("-")));
    }
}

/// Memperbaiki kiriman tanpa mengubah jumlahnya. Kas pembelian mengikuti harga beli yang baru.
pub fn perbaiki_kiriman(conn: &Connection, data: &Path, s: &Sesi, id: i64, mut p: PerbaikanKiriman) -> Hasil<()> {
    wajib(s, Modul::BarangMasuk, Tingkat::Ubah)?;
    crate::akses::wajib_harga_beli(s)?;
    let k = kepala_kiriman(conn, id)?;
    if k.batal {
        return Err(Galat::validasi("Kiriman yang sudah dibatalkan tidak bisa diperbaiki."));
    }
    if p.harga_beli < 0 {
        return Err(Galat::validasi("Harga beli tidak boleh negatif."));
    }
    p.spek = rapikan(p.spek);
    p.kedaluwarsa = rapikan(p.kedaluwarsa);
    p.pemasok = rapikan(p.pemasok);
    periksa_kedaluwarsa(&p.kedaluwarsa, waktu::tanggal(&k.tanggal_masuk)?)?;
    let foto_baru = p.foto_sumber.as_deref().map(|f| foto::simpan(data, f)).transpose()?;
    let foto_akhir = foto_baru.clone().or(k.foto.clone());
    let hasil = (|| {
        let tx = conn.unchecked_transaction()?;
        tx.execute(
            "UPDATE kiriman SET harga_beli=?1, spek=?2, kedaluwarsa=?3, pemasok=?4, foto=?5 WHERE id=?6",
            params![p.harga_beli, p.spek, p.kedaluwarsa, p.pemasok, foto_akhir, id],
        )?;
        let mut catatan = Vec::new();
        if p.harga_beli != k.harga_beli {
            catatan.push(format!("harga beli {} → {}", k.harga_beli, p.harga_beli));
            if !k.stok_awal {
                let total = nilai(k.jumlah_awal, p.harga_beli);
                let ada = kas::sesuaikan_otomatis(&tx, "kiriman", id, total)?;
                if let (false, true, Some(akun)) = (ada, total > 0, k.akun_kas_id) {
                    let ket = format!("Beli {} {} {}", teks_jumlah(k.jumlah_awal), k.satuan, k.nama);
                    kas::catat_otomatis(&tx, &k.tanggal_masuk, "keluar", akun, kas::KAT_PEMBELIAN, total, &ket, "kiriman", id, s.pengguna_id)?;
                }
            }
        }
        beda("spek", &k.spek, &p.spek, &mut catatan);
        beda("kedaluwarsa", &k.kedaluwarsa, &p.kedaluwarsa, &mut catatan);
        beda("pemasok", &k.pemasok, &p.pemasok, &mut catatan);
        if foto_baru.is_some() {
            catatan.push("foto diganti".into());
        }
        if catatan.is_empty() {
            catatan.push("tanpa perubahan".into());
        }
        log::catat(&tx, Some(s.pengguna_id), "kiriman.perbaiki", &format!("Kiriman #{id} ({}) diperbaiki: {}", k.nama, catatan.join("; ")))?;
        tx.commit()?;
        Ok(())
    })();
    match (&hasil, &foto_baru) {
        (Ok(()), Some(_)) => {
            if let Some(l) = &k.foto {
                foto::hapus(data, l);
            }
        }
        (Err(_), Some(f)) => foto::hapus(data, f),
        _ => {}
    }
    hasil
}

#[derive(Debug, Serialize)]
pub struct Penyesuaian {
    pub id: i64,
    pub kiriman_id: i64,
    pub barang_id: i64,
    pub barang_nama: String,
    pub satuan: String,
    pub tanggal_masuk: String,
    pub tanggal: String,
    pub selisih: i64,
    pub alasan: String,
    pub catatan: Option<String>,
    pub batal: bool,
    /// Nilai selisih menurut harga beli. `None` tanpa hak harga beli.
    pub nilai: Option<i64>,
    pub dibuat_oleh_nama: Option<String>,
}

/// Menambah atau mengurangi sisa satu kiriman (rusak, hilang, pakai sendiri, koreksi hitung). Tidak memengaruhi kas.
pub fn sesuaikan(conn: &Connection, s: &Sesi, kiriman_id: i64, tanggal: &str, selisih: i64, alasan: &str, catatan: Option<&str>) -> Hasil<i64> {
    wajib(s, Modul::BarangMasuk, Tingkat::Ubah)?;
    waktu::tanggal(tanggal)?;
    if selisih == 0 {
        return Err(Galat::validasi("Selisih penyesuaian tidak boleh nol."));
    }
    if !ALASAN_PENYESUAIAN.contains(&alasan) {
        return Err(Galat::validasi("Pilih alasan penyesuaian: rusak, kedaluwarsa, hilang, pakai sendiri, atau koreksi hitung."));
    }
    let k = kepala_kiriman(conn, kiriman_id)?;
    if k.batal {
        return Err(Galat::validasi("Kiriman yang sudah dibatalkan tidak bisa disesuaikan."));
    }
    if k.sisa + selisih < 0 {
        return Err(Galat::baru(
            "STOK_KURANG",
            format!("Sisa kiriman {} tanggal {} hanya {} {}, tidak bisa dikurangi {} {}.", k.nama, k.tanggal_masuk, teks_jumlah(k.sisa), k.satuan, teks_jumlah(-selisih), k.satuan),
        ));
    }
    let tx = conn.unchecked_transaction()?;
    tx.execute("UPDATE kiriman SET sisa=sisa+?1 WHERE id=?2", params![selisih, kiriman_id])?;
    tx.execute(
        "INSERT INTO penyesuaian_stok(kiriman_id,tanggal,selisih,alasan,catatan,dibuat_oleh,dibuat_pada) VALUES(?1,?2,?3,?4,?5,?6,?7)",
        params![kiriman_id, tanggal, selisih, alasan, rapikan(catatan.map(str::to_string)), s.pengguna_id, waktu::kini()],
    )?;
    let id = tx.last_insert_rowid();
    log::catat(
        &tx,
        Some(s.pengguna_id),
        "stok.sesuaikan",
        &format!("Penyesuaian #{id}: {} {} {} kiriman {} ({alasan})", teks_jumlah(selisih), k.satuan, k.nama, k.tanggal_masuk),
    )?;
    tx.commit()?;
    Ok(id)
}

pub fn batal_penyesuaian(conn: &Connection, s: &Sesi, id: i64, alasan: &str) -> Hasil<()> {
    wajib(s, Modul::BarangMasuk, Tingkat::Ubah)?;
    let alasan = kas::periksa_alasan(alasan)?;
    let (kiriman_id, selisih, batal): (i64, i64, bool) = conn
        .query_row("SELECT kiriman_id, selisih, batal FROM penyesuaian_stok WHERE id=?1", [id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .map_err(|_| Galat::tidak_ditemukan("Penyesuaian stok"))?;
    if batal {
        return Err(Galat::validasi("Penyesuaian ini sudah dibatalkan."));
    }
    let k = kepala_kiriman(conn, kiriman_id)?;
    if k.sisa - selisih < 0 {
        return Err(Galat::baru(
            "STOK_KURANG",
            format!("Penyesuaian ini tidak bisa dibatalkan karena sebagian barangnya sudah keluar (sisa {} {}).", teks_jumlah(k.sisa), k.satuan),
        ));
    }
    let tx = conn.unchecked_transaction()?;
    tx.execute("UPDATE kiriman SET sisa=sisa-?1 WHERE id=?2", params![selisih, kiriman_id])?;
    tx.execute("UPDATE penyesuaian_stok SET batal=1, alasan_batal=?1 WHERE id=?2", params![alasan, id])?;
    log::catat(&tx, Some(s.pengguna_id), "stok.sesuaikan_batal", &format!("Penyesuaian #{id} ({}) dibatalkan: {alasan}", k.nama))?;
    tx.commit()?;
    Ok(())
}

pub fn daftar_penyesuaian(conn: &Connection, s: &Sesi, dari: &str, sampai: &str) -> Hasil<Vec<Penyesuaian>> {
    wajib(s, Modul::BarangMasuk, Tingkat::Lihat)?;
    waktu::rentang(dari, sampai)?;
    let harga = s.boleh_harga_beli();
    let mut q = conn.prepare(
        "SELECT p.id, p.kiriman_id, b.id, b.nama, b.satuan, k.tanggal_masuk, p.tanggal, p.selisih, p.alasan, p.catatan, p.batal, k.harga_beli, u.nama
         FROM penyesuaian_stok p JOIN kiriman k ON k.id=p.kiriman_id JOIN barang b ON b.id=k.barang_id LEFT JOIN pengguna u ON u.id=p.dibuat_oleh
         WHERE p.tanggal BETWEEN ?1 AND ?2 ORDER BY p.tanggal DESC, p.id DESC",
    )?;
    let v = q
        .query_map(params![dari, sampai], |r| {
            let (selisih, harga_beli): (i64, i64) = (r.get(7)?, r.get(11)?);
            Ok(Penyesuaian {
                id: r.get(0)?,
                kiriman_id: r.get(1)?,
                barang_id: r.get(2)?,
                barang_nama: r.get(3)?,
                satuan: r.get(4)?,
                tanggal_masuk: r.get(5)?,
                tanggal: r.get(6)?,
                selisih,
                alasan: r.get(8)?,
                catatan: r.get(9)?,
                batal: r.get(10)?,
                nilai: harga.then(|| selisih.signum() * nilai(selisih.abs(), harga_beli)),
                dibuat_oleh_nama: r.get(12)?,
            })
        })?
        .collect::<Result<_, _>>()?;
    Ok(v)
}
