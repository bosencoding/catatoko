//! Kas: akun, kategori, catatan manual, pindah antar akun, dan catatan otomatis dari stok.

use crate::akses::{wajib, wajib_pemilik, Modul, Sesi, Tingkat};
use crate::galat::{Galat, Hasil};
use crate::{log, waktu};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

pub const KAT_PENJUALAN: &str = "Penjualan";
pub const KAT_PEMBELIAN: &str = "Pembelian barang";
pub const KAT_PINDAH: &str = "Pindah akun";

#[derive(Debug, Clone, Deserialize)]
pub struct KasMasukan {
    pub tanggal: String,
    pub arah: String,
    pub akun_kas_id: i64,
    pub kategori_id: i64,
    pub jumlah: i64,
    pub keterangan: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SaringKas {
    pub dari: String,
    pub sampai: String,
    pub akun_kas_id: Option<i64>,
    pub kategori_id: Option<i64>,
    pub arah: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BarisKas {
    pub id: i64,
    pub tanggal: String,
    pub arah: String,
    pub akun_kas_id: i64,
    pub akun: String,
    pub kategori_id: i64,
    pub kategori: String,
    pub jumlah: i64,
    pub keterangan: Option<String>,
    pub sumber: String,
    pub ref_jenis: Option<String>,
    pub ref_id: Option<i64>,
    pub batal: bool,
    pub alasan_batal: Option<String>,
    /// Saldo berjalan setelah baris ini (baris batal tidak dihitung).
    pub saldo: i64,
    pub dibuat_oleh_nama: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SaldoAkun {
    pub akun_kas_id: i64,
    pub nama: String,
    pub aktif: bool,
    pub saldo: i64,
}

#[derive(Debug, Serialize)]
pub struct Saldo {
    pub per_akun: Vec<SaldoAkun>,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct AkunKas {
    pub id: i64,
    pub nama: String,
    pub aktif: bool,
}

#[derive(Debug, Serialize)]
pub struct KategoriKas {
    pub id: i64,
    pub nama: String,
    pub bawaan: bool,
}

const TANDA: &str = "CASE arah WHEN 'masuk' THEN jumlah ELSE -jumlah END";

pub(crate) fn saldo_akun(conn: &Connection, akun: i64) -> Hasil<i64> {
    Ok(conn.query_row(&format!("SELECT COALESCE(SUM({TANDA}),0) FROM kas WHERE batal=0 AND akun_kas_id=?1"), [akun], |r| r.get(0))?)
}

pub(crate) fn akun_aktif(conn: &Connection, id: i64) -> Hasil<String> {
    conn.query_row("SELECT nama FROM akun_kas WHERE id=?1 AND aktif=1", [id], |r| r.get(0))
        .optional()?
        .ok_or_else(|| Galat::validasi("Akun kas tidak ditemukan atau sudah dinonaktifkan."))
}

fn id_kategori(conn: &Connection, nama: &str) -> Hasil<i64> {
    Ok(conn.query_row("SELECT id FROM kategori_kas WHERE nama=?1", [nama], |r| r.get(0))?)
}

fn rupiah(n: i64) -> String {
    let s = n.abs().to_string();
    let mut hasil = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            hasil.push('.');
        }
        hasil.push(ch);
    }
    format!("{}Rp {hasil}", if n < 0 { "-" } else { "" })
}

pub fn daftar(conn: &Connection, s: &Sesi, f: &SaringKas) -> Hasil<Vec<BarisKas>> {
    wajib(s, Modul::Kas, Tingkat::Lihat)?;
    waktu::rentang(&f.dari, &f.sampai)?;
    let mut saldo: i64 = conn.query_row(
        &format!("SELECT COALESCE(SUM({TANDA}),0) FROM kas WHERE batal=0 AND tanggal < ?1 AND (?2 IS NULL OR akun_kas_id=?2)"),
        params![f.dari, f.akun_kas_id],
        |r| r.get(0),
    )?;
    let mut q = conn.prepare(
        "SELECT k.id, k.tanggal, k.arah, k.akun_kas_id, a.nama, k.kategori_id, g.nama, k.jumlah, k.keterangan, k.sumber,
                k.ref_jenis, k.ref_id, k.batal, k.alasan_batal, p.nama
         FROM kas k JOIN akun_kas a ON a.id=k.akun_kas_id JOIN kategori_kas g ON g.id=k.kategori_id
              LEFT JOIN pengguna p ON p.id=k.dibuat_oleh
         WHERE k.tanggal BETWEEN ?1 AND ?2 AND (?3 IS NULL OR k.akun_kas_id=?3)
         ORDER BY k.tanggal, k.id",
    )?;
    let baris = q.query_map(params![f.dari, f.sampai, f.akun_kas_id], |r| {
        Ok(BarisKas {
            id: r.get(0)?,
            tanggal: r.get(1)?,
            arah: r.get(2)?,
            akun_kas_id: r.get(3)?,
            akun: r.get(4)?,
            kategori_id: r.get(5)?,
            kategori: r.get(6)?,
            jumlah: r.get(7)?,
            keterangan: r.get(8)?,
            sumber: r.get(9)?,
            ref_jenis: r.get(10)?,
            ref_id: r.get(11)?,
            batal: r.get(12)?,
            alasan_batal: r.get(13)?,
            saldo: 0,
            dibuat_oleh_nama: r.get(14)?,
        })
    })?;
    let mut hasil = Vec::new();
    for b in baris {
        let mut b = b?;
        if !b.batal {
            saldo += if b.arah == "masuk" { b.jumlah } else { -b.jumlah };
        }
        b.saldo = saldo;
        let lolos = f.kategori_id.map_or(true, |k| k == b.kategori_id) && f.arah.as_deref().map_or(true, |a| a == b.arah);
        if lolos {
            hasil.push(b);
        }
    }
    Ok(hasil)
}

pub fn saldo(conn: &Connection, s: &Sesi) -> Hasil<Saldo> {
    wajib(s, Modul::Kas, Tingkat::Lihat)?;
    saldo_tanpa_cek(conn)
}

pub(crate) fn saldo_tanpa_cek(conn: &Connection) -> Hasil<Saldo> {
    let mut q = conn.prepare(&format!(
        "SELECT a.id, a.nama, a.aktif, COALESCE((SELECT SUM({TANDA}) FROM kas WHERE batal=0 AND akun_kas_id=a.id),0)
         FROM akun_kas a ORDER BY a.id"
    ))?;
    let per_akun: Vec<SaldoAkun> = q
        .query_map([], |r| Ok(SaldoAkun { akun_kas_id: r.get(0)?, nama: r.get(1)?, aktif: r.get(2)?, saldo: r.get(3)? }))?
        .collect::<Result<_, _>>()?;
    let total = per_akun.iter().map(|a| a.saldo).sum();
    Ok(Saldo { per_akun, total })
}

fn periksa(conn: &Connection, m: &KasMasukan) -> Hasil<()> {
    waktu::tanggal(&m.tanggal)?;
    if m.arah != "masuk" && m.arah != "keluar" {
        return Err(Galat::validasi("Arah kas harus masuk atau keluar."));
    }
    if m.jumlah <= 0 {
        return Err(Galat::validasi("Jumlah uang harus lebih dari nol."));
    }
    akun_aktif(conn, m.akun_kas_id)?;
    let bawaan: Option<bool> = conn.query_row("SELECT bawaan FROM kategori_kas WHERE id=?1", [m.kategori_id], |r| r.get(0)).optional()?;
    match bawaan {
        None => Err(Galat::validasi("Kategori kas tidak ditemukan.")),
        Some(true) => Err(Galat::validasi("Kategori ini diisi otomatis oleh aplikasi. Pilih kategori lain.")),
        Some(false) => Ok(()),
    }
}

pub fn tambah_manual(conn: &Connection, s: &Sesi, m: KasMasukan) -> Hasil<i64> {
    wajib(s, Modul::Kas, Tingkat::Tambah)?;
    periksa(conn, &m)?;
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO kas(tanggal,arah,akun_kas_id,kategori_id,jumlah,keterangan,sumber,dibuat_oleh,dibuat_pada) VALUES(?1,?2,?3,?4,?5,?6,'manual',?7,?8)",
        params![m.tanggal, m.arah, m.akun_kas_id, m.kategori_id, m.jumlah, m.keterangan, s.pengguna_id, waktu::kini()],
    )?;
    let id = tx.last_insert_rowid();
    log::catat(&tx, Some(s.pengguna_id), "kas.tambah", &format!("Kas {} {} tanggal {} (#{id})", m.arah, rupiah(m.jumlah), m.tanggal))?;
    tx.commit()?;
    Ok(id)
}

/// (sumber, ref_jenis, ref_id, batal, arah, jumlah, tanggal) baris kas.
type Kepala = (String, Option<String>, Option<i64>, bool, String, i64, String);

fn kepala(conn: &Connection, id: i64) -> Hasil<Kepala> {
    conn.query_row("SELECT sumber, ref_jenis, ref_id, batal, arah, jumlah, tanggal FROM kas WHERE id=?1", [id], |r| {
        Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?))
    })
    .map_err(|_| Galat::tidak_ditemukan("Catatan kas"))
}

pub fn ubah_manual(conn: &Connection, s: &Sesi, id: i64, m: KasMasukan) -> Hasil<()> {
    wajib(s, Modul::Kas, Tingkat::Ubah)?;
    let (sumber, ref_jenis, _, batal, arah_lama, jumlah_lama, tanggal_lama) = kepala(conn, id)?;
    if sumber != "manual" {
        return Err(Galat::validasi("Catatan kas otomatis mengikuti transaksi barangnya. Ubah atau batalkan transaksi barangnya."));
    }
    if ref_jenis.is_some() {
        return Err(Galat::validasi("Pindah antar akun tidak bisa diubah. Batalkan lalu catat ulang."));
    }
    if batal {
        return Err(Galat::validasi("Catatan kas ini sudah dibatalkan."));
    }
    periksa(conn, &m)?;
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE kas SET tanggal=?1, arah=?2, akun_kas_id=?3, kategori_id=?4, jumlah=?5, keterangan=?6 WHERE id=?7",
        params![m.tanggal, m.arah, m.akun_kas_id, m.kategori_id, m.jumlah, m.keterangan, id],
    )?;
    log::catat(
        &tx,
        Some(s.pengguna_id),
        "kas.ubah",
        &format!("Kas #{id} diubah: {arah_lama} {jumlah_lama} tanggal {tanggal_lama} → {} {} tanggal {}", m.arah, m.jumlah, m.tanggal),
    )?;
    tx.commit()?;
    Ok(())
}

pub(crate) fn periksa_alasan(alasan: &str) -> Hasil<&str> {
    let alasan = alasan.trim();
    if alasan.chars().count() < 3 {
        return Err(Galat::validasi("Tulis alasan pembatalan (minimal 3 huruf)."));
    }
    Ok(alasan)
}

pub fn batal_manual(conn: &Connection, s: &Sesi, id: i64, alasan: &str) -> Hasil<()> {
    wajib(s, Modul::Kas, Tingkat::Ubah)?;
    let alasan = periksa_alasan(alasan)?;
    let (sumber, ref_jenis, ref_id, batal, arah, jumlah, _) = kepala(conn, id)?;
    if sumber != "manual" {
        return Err(Galat::validasi("Catatan kas otomatis mengikuti transaksi barangnya. Batalkan transaksi barangnya."));
    }
    if batal {
        return Err(Galat::validasi("Catatan kas ini sudah dibatalkan."));
    }
    let tx = conn.unchecked_transaction()?;
    if ref_jenis.as_deref() == Some("pindah") {
        tx.execute("UPDATE kas SET batal=1, alasan_batal=?1 WHERE ref_jenis='pindah' AND ref_id=?2", params![alasan, ref_id])?;
    } else {
        tx.execute("UPDATE kas SET batal=1, alasan_batal=?1 WHERE id=?2", params![alasan, id])?;
    }
    log::catat(&tx, Some(s.pengguna_id), "kas.batal", &format!("Kas #{id} ({arah} {}) dibatalkan: {alasan}", rupiah(jumlah)))?;
    tx.commit()?;
    Ok(())
}

/// Memindahkan uang antar akun: dua baris kas yang saling menghapus pada total.
pub fn pindah(conn: &Connection, s: &Sesi, tanggal: &str, dari_akun: i64, ke_akun: i64, jumlah: i64, ket: Option<&str>) -> Hasil<i64> {
    wajib(s, Modul::Kas, Tingkat::Tambah)?;
    waktu::tanggal(tanggal)?;
    if dari_akun == ke_akun {
        return Err(Galat::validasi("Akun asal dan akun tujuan harus berbeda."));
    }
    if jumlah <= 0 {
        return Err(Galat::validasi("Jumlah uang harus lebih dari nol."));
    }
    let nama_asal = akun_aktif(conn, dari_akun)?;
    let nama_tujuan = akun_aktif(conn, ke_akun)?;
    let saldo_asal = saldo_akun(conn, dari_akun)?;
    if jumlah > saldo_asal {
        return Err(Galat::validasi(format!("Saldo {nama_asal} hanya {}, tidak cukup untuk dipindahkan {}.", rupiah(saldo_asal), rupiah(jumlah))));
    }
    let kategori = id_kategori(conn, KAT_PINDAH)?;
    let keterangan = ket.map(str::trim).filter(|k| !k.is_empty()).map(str::to_string).unwrap_or_else(|| format!("{nama_asal} → {nama_tujuan}"));
    let tx = conn.unchecked_transaction()?;
    let sisip = "INSERT INTO kas(tanggal,arah,akun_kas_id,kategori_id,jumlah,keterangan,sumber,ref_jenis,ref_id,dibuat_oleh,dibuat_pada)
                 VALUES(?1,?2,?3,?4,?5,?6,'manual','pindah',?7,?8,?9)";
    tx.execute(sisip, params![tanggal, "keluar", dari_akun, kategori, jumlah, keterangan, None::<i64>, s.pengguna_id, waktu::kini()])?;
    let id = tx.last_insert_rowid();
    tx.execute("UPDATE kas SET ref_id=?1 WHERE id=?1", [id])?;
    tx.execute(sisip, params![tanggal, "masuk", ke_akun, kategori, jumlah, keterangan, id, s.pengguna_id, waktu::kini()])?;
    log::catat(&tx, Some(s.pengguna_id), "kas.pindah", &format!("Pindah {} dari {nama_asal} ke {nama_tujuan} (#{id})", rupiah(jumlah)))?;
    tx.commit()?;
    Ok(id)
}

/// Catatan kas yang lahir dari transaksi barang. Sengaja tanpa pemeriksaan hak Kas:
/// penjualan oleh kasir tetap harus tercatat di kas.
#[allow(clippy::too_many_arguments)]
pub(crate) fn catat_otomatis(
    conn: &Connection,
    tanggal: &str,
    arah: &str,
    akun: i64,
    kategori: &str,
    jumlah: i64,
    ket: &str,
    ref_jenis: &str,
    ref_id: i64,
    oleh: i64,
) -> Hasil<i64> {
    conn.execute(
        "INSERT INTO kas(tanggal,arah,akun_kas_id,kategori_id,jumlah,keterangan,sumber,ref_jenis,ref_id,dibuat_oleh,dibuat_pada)
         VALUES(?1,?2,?3,?4,?5,?6,'otomatis',?7,?8,?9,?10)",
        params![tanggal, arah, akun, id_kategori(conn, kategori)?, jumlah, ket, ref_jenis, ref_id, oleh, waktu::kini()],
    )?;
    Ok(conn.last_insert_rowid())
}

pub(crate) fn batal_otomatis(conn: &Connection, ref_jenis: &str, ref_id: i64) -> Hasil<()> {
    conn.execute(
        "UPDATE kas SET batal=1, alasan_batal='Transaksi barangnya dibatalkan' WHERE sumber='otomatis' AND ref_jenis=?1 AND ref_id=?2",
        params![ref_jenis, ref_id],
    )?;
    Ok(())
}

/// Menyesuaikan jumlah catatan otomatis yang masih berlaku. `false` bila tidak ada catatannya.
pub(crate) fn sesuaikan_otomatis(conn: &Connection, ref_jenis: &str, ref_id: i64, jumlah_baru: i64) -> Hasil<bool> {
    let id: Option<i64> = conn
        .query_row("SELECT id FROM kas WHERE sumber='otomatis' AND batal=0 AND ref_jenis=?1 AND ref_id=?2", params![ref_jenis, ref_id], |r| r.get(0))
        .optional()?;
    let Some(id) = id else { return Ok(false) };
    if jumlah_baru > 0 {
        conn.execute("UPDATE kas SET jumlah=?1 WHERE id=?2", params![jumlah_baru, id])?;
    } else {
        conn.execute("UPDATE kas SET batal=1, alasan_batal='Nilai menjadi nol setelah perbaikan' WHERE id=?1", [id])?;
    }
    Ok(true)
}

pub fn akun_daftar(conn: &Connection) -> Hasil<Vec<AkunKas>> {
    let mut q = conn.prepare("SELECT id, nama, aktif FROM akun_kas ORDER BY id")?;
    let v = q.query_map([], |r| Ok(AkunKas { id: r.get(0)?, nama: r.get(1)?, aktif: r.get(2)? }))?.collect::<Result<_, _>>()?;
    Ok(v)
}

fn nama_unik(conn: &Connection, tabel: &str, nama: &str, kecuali: i64) -> Hasil<()> {
    if nama.is_empty() {
        return Err(Galat::validasi("Nama wajib diisi."));
    }
    let n: i64 = conn.query_row(&format!("SELECT count(*) FROM {tabel} WHERE nama=?1 AND id<>?2"), params![nama, kecuali], |r| r.get(0))?;
    if n > 0 {
        return Err(Galat::validasi(format!("Nama '{nama}' sudah dipakai.")));
    }
    Ok(())
}

pub fn akun_tambah(conn: &Connection, s: &Sesi, nama: &str) -> Hasil<i64> {
    wajib_pemilik(s)?;
    let nama = nama.trim();
    nama_unik(conn, "akun_kas", nama, 0)?;
    conn.execute("INSERT INTO akun_kas(nama) VALUES(?1)", [nama])?;
    let id = conn.last_insert_rowid();
    log::catat(conn, Some(s.pengguna_id), "kas.akun_tambah", &format!("Akun kas '{nama}' ditambahkan"))?;
    Ok(id)
}

pub fn akun_ubah(conn: &Connection, s: &Sesi, id: i64, nama: &str, aktif: bool) -> Hasil<()> {
    wajib_pemilik(s)?;
    let nama = nama.trim();
    nama_unik(conn, "akun_kas", nama, id)?;
    if !aktif {
        let lain: i64 = conn.query_row("SELECT count(*) FROM akun_kas WHERE aktif=1 AND id<>?1", [id], |r| r.get(0))?;
        if lain == 0 {
            return Err(Galat::validasi("Harus ada minimal satu akun kas yang aktif."));
        }
    }
    if conn.execute("UPDATE akun_kas SET nama=?1, aktif=?2 WHERE id=?3", params![nama, aktif, id])? == 0 {
        return Err(Galat::tidak_ditemukan("Akun kas"));
    }
    log::catat(conn, Some(s.pengguna_id), "kas.akun_ubah", &format!("Akun kas #{id} menjadi '{nama}', aktif {aktif}"))
}

pub fn kategori_daftar(conn: &Connection) -> Hasil<Vec<KategoriKas>> {
    let mut q = conn.prepare("SELECT id, nama, bawaan FROM kategori_kas ORDER BY bawaan DESC, nama COLLATE NOCASE")?;
    let v = q.query_map([], |r| Ok(KategoriKas { id: r.get(0)?, nama: r.get(1)?, bawaan: r.get(2)? }))?.collect::<Result<_, _>>()?;
    Ok(v)
}

pub fn kategori_tambah(conn: &Connection, s: &Sesi, nama: &str) -> Hasil<i64> {
    wajib_pemilik(s)?;
    let nama = nama.trim();
    nama_unik(conn, "kategori_kas", nama, 0)?;
    conn.execute("INSERT INTO kategori_kas(nama) VALUES(?1)", [nama])?;
    Ok(conn.last_insert_rowid())
}
