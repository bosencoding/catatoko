//! Lima laporan, semuanya dalam satu bentuk `Laporan` yang dipakai pratinjau dan kedua ekspor.

use crate::akses::{wajib, wajib_harga_beli, Modul, Sesi, Tingkat};
use crate::galat::{Galat, Hasil};
use crate::stok::{nilai, URUT_KIRIMAN};
use crate::{pengaturan, waktu};
use chrono::{Datelike, NaiveDate};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "t", content = "v", rename_all = "snake_case")]
pub enum Sel {
    Teks(String),
    Rupiah(i64),
    /// Jumlah barang per seperseribu satuan.
    Jumlah(i64),
    /// `YYYY-MM-DD`
    Tanggal(String),
    Kosong,
}

#[derive(Debug, Clone, Serialize)]
pub struct Laporan {
    pub judul: String,
    pub nama_toko: String,
    pub periode: String,
    pub kolom: Vec<String>,
    pub baris: Vec<Vec<Sel>>,
    /// Sama panjang dengan `kolom`.
    pub total: Option<Vec<Sel>>,
    pub ringkasan: Vec<(String, Sel)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(tag = "jenis", rename_all = "snake_case")]
pub enum Jenis {
    ArusKas,
    PosisiStok,
    KartuStok { barang_id: i64 },
    BarangMasuk,
    BarangKeluar,
    LabaKotor,
}

const BULAN: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "Mei", "Jun", "Jul", "Agu", "Sep", "Okt", "Nov", "Des"];

pub fn nama_bulan(d: NaiveDate) -> &'static str {
    BULAN[d.month0() as usize]
}

/// `2026-09-03` → `03 Sep 2026`.
pub fn tanggal_panjang(d: NaiveDate) -> String {
    format!("{:02} {} {}", d.day(), nama_bulan(d), d.year())
}

fn teks(s: impl Into<String>) -> Sel {
    Sel::Teks(s.into())
}

fn teks_opsional(s: Option<String>) -> Sel {
    s.map(Sel::Teks).unwrap_or(Sel::Kosong)
}

fn kolom(nama: &[&str]) -> Vec<String> {
    nama.iter().map(|s| s.to_string()).collect()
}

/// Baris total: "Total" di kolom pertama, lalu nilai pada kolom yang disebut.
fn baris_total(lebar: usize, isi: &[(usize, Sel)]) -> Vec<Sel> {
    let mut v = vec![Sel::Kosong; lebar];
    v[0] = teks("Total");
    for (i, s) in isi {
        v[*i] = s.clone();
    }
    v
}

pub fn buat(conn: &Connection, s: &Sesi, jenis: Jenis, dari: &str, sampai: &str) -> Hasil<Laporan> {
    wajib(s, Modul::Laporan, Tingkat::Lihat)?;
    if matches!(jenis, Jenis::PosisiStok | Jenis::BarangMasuk | Jenis::LabaKotor) {
        wajib_harga_beli(s)?;
    }
    let (a, b) = waktu::rentang(dari, sampai)?;
    let mut l = Laporan {
        judul: String::new(),
        nama_toko: pengaturan::baca(conn, "nama_toko")?.unwrap_or_default(),
        periode: format!("{} s.d. {}", tanggal_panjang(a), tanggal_panjang(b)),
        kolom: Vec::new(),
        baris: Vec::new(),
        total: None,
        ringkasan: Vec::new(),
    };
    match jenis {
        Jenis::ArusKas => arus_kas(conn, &mut l, dari, sampai)?,
        Jenis::PosisiStok => posisi_stok(conn, &mut l)?,
        Jenis::KartuStok { barang_id } => kartu_stok(conn, &mut l, barang_id, dari, sampai)?,
        Jenis::BarangMasuk => barang_masuk(conn, &mut l, dari, sampai)?,
        Jenis::BarangKeluar => barang_keluar(conn, &mut l, dari, sampai, s.boleh_harga_beli())?,
        Jenis::LabaKotor => laba_kotor(conn, &mut l, dari, sampai)?,
    }
    Ok(l)
}

fn arus_kas(conn: &Connection, l: &mut Laporan, dari: &str, sampai: &str) -> Hasil<()> {
    l.judul = "Laporan Arus Kas".into();
    l.kolom = kolom(&["Tanggal", "Akun", "Kategori", "Keterangan", "Masuk", "Keluar"]);
    let mut q = conn.prepare(
        "SELECT k.tanggal, a.nama, g.nama, k.keterangan, k.arah, k.jumlah
         FROM kas k JOIN akun_kas a ON a.id=k.akun_kas_id JOIN kategori_kas g ON g.id=k.kategori_id
         WHERE k.batal=0 AND k.ref_jenis IS NOT 'pindah' AND k.tanggal BETWEEN ?1 AND ?2 ORDER BY k.tanggal, k.id",
    )?;
    let (mut masuk, mut keluar) = (0i64, 0i64);
    let mut per_kategori: Vec<(String, i64)> = Vec::new();
    let baris = q.query_map(params![dari, sampai], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, Option<String>>(3)?, r.get::<_, String>(4)?, r.get::<_, i64>(5)?))
    })?;
    for b in baris {
        let (tgl, akun, kategori, ket, arah, jumlah) = b?;
        let ini_masuk = arah == "masuk";
        if ini_masuk { masuk += jumlah } else { keluar += jumlah }
        let label = format!("{}: {kategori}", if ini_masuk { "Masuk" } else { "Keluar" });
        match per_kategori.iter_mut().find(|(k, _)| *k == label) {
            Some((_, n)) => *n += jumlah,
            None => per_kategori.push((label, jumlah)),
        }
        let (sel_masuk, sel_keluar) = if ini_masuk { (Sel::Rupiah(jumlah), Sel::Kosong) } else { (Sel::Kosong, Sel::Rupiah(jumlah)) };
        l.baris.push(vec![Sel::Tanggal(tgl), teks(akun), teks(kategori), teks_opsional(ket), sel_masuk, sel_keluar]);
    }
    l.total = Some(baris_total(6, &[(4, Sel::Rupiah(masuk)), (5, Sel::Rupiah(keluar))]));
    let saldo_awal: i64 = conn.query_row(
        "SELECT COALESCE(SUM(CASE arah WHEN 'masuk' THEN jumlah ELSE -jumlah END),0) FROM kas WHERE batal=0 AND tanggal < ?1",
        [dari],
        |r| r.get(0),
    )?;
    per_kategori.sort();
    per_kategori.reverse(); // "Masuk: …" dulu, lalu "Keluar: …"
    l.ringkasan.push(("Saldo awal".into(), Sel::Rupiah(saldo_awal)));
    l.ringkasan.extend(per_kategori.into_iter().map(|(k, n)| (k, Sel::Rupiah(n))));
    l.ringkasan.push(("Total masuk".into(), Sel::Rupiah(masuk)));
    l.ringkasan.push(("Total keluar".into(), Sel::Rupiah(keluar)));
    l.ringkasan.push(("Saldo akhir".into(), Sel::Rupiah(saldo_awal + masuk - keluar)));
    Ok(())
}

fn posisi_stok(conn: &Connection, l: &mut Laporan) -> Hasil<()> {
    l.judul = "Laporan Posisi Stok".into();
    l.periode = format!("Keadaan per {}", tanggal_panjang(waktu::tanggal(&waktu::hari_ini())?));
    l.kolom = kolom(&["Barang", "Tanggal masuk", "Spek", "Kedaluwarsa", "Sisa", "Satuan", "Harga beli", "Nilai"]);
    let mut q = conn.prepare(&format!(
        "SELECT b.nama, k.tanggal_masuk, k.spek, k.kedaluwarsa, k.sisa, b.satuan, k.harga_beli
         FROM kiriman k JOIN barang b ON b.id=k.barang_id WHERE k.batal=0 AND k.sisa>0 ORDER BY b.nama COLLATE NOCASE, {URUT_KIRIMAN}"
    ))?;
    let mut total = 0;
    let baris = q.query_map([], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, Option<String>>(2)?, r.get::<_, Option<String>>(3)?, r.get::<_, i64>(4)?, r.get::<_, String>(5)?, r.get::<_, i64>(6)?))
    })?;
    for b in baris {
        let (nama, masuk, spek, exp, sisa, satuan, harga) = b?;
        let n = nilai(sisa, harga);
        total += n;
        l.baris.push(vec![teks(nama), Sel::Tanggal(masuk), teks_opsional(spek), exp.map(Sel::Tanggal).unwrap_or(Sel::Kosong), Sel::Jumlah(sisa), teks(satuan), Sel::Rupiah(harga), Sel::Rupiah(n)]);
    }
    l.total = Some(baris_total(8, &[(7, Sel::Rupiah(total))]));
    l.ringkasan.push(("Jumlah kiriman bersisa".into(), teks(l.baris.len().to_string())));
    l.ringkasan.push(("Total nilai stok".into(), Sel::Rupiah(total)));
    Ok(())
}

fn kartu_stok(conn: &Connection, l: &mut Laporan, barang_id: i64, dari: &str, sampai: &str) -> Hasil<()> {
    let (nama, satuan): (String, String) =
        conn.query_row("SELECT nama, satuan FROM barang WHERE id=?1", [barang_id], |r| Ok((r.get(0)?, r.get(1)?))).map_err(|_| Galat::tidak_ditemukan("Barang"))?;
    l.judul = format!("Kartu Stok: {nama} ({satuan})");
    l.kolom = kolom(&["Tanggal", "Jenis", "Keterangan", "Masuk", "Keluar", "Saldo"]);
    // (tanggal, urutan jenis, id, jenis, keterangan, perubahan)
    let mut q = conn.prepare(
        "SELECT tanggal, urut, id, jenis, ket, ubah FROM (
           SELECT k.tanggal_masuk AS tanggal, 0 AS urut, k.id AS id, CASE k.stok_awal WHEN 1 THEN 'Stok awal' ELSE 'Masuk' END AS jenis,
                  COALESCE(k.spek,'') || CASE WHEN k.pemasok IS NULL THEN '' ELSE ' · ' || k.pemasok END AS ket, k.jumlah_awal AS ubah
             FROM kiriman k WHERE k.barang_id=?1 AND k.batal=0
           UNION ALL
           SELECT p.tanggal, 1, p.id, 'Penyesuaian', p.alasan || ' (kiriman ' || k.tanggal_masuk || ')' || COALESCE(' · ' || p.catatan, ''), p.selisih
             FROM penyesuaian_stok p JOIN kiriman k ON k.id=p.kiriman_id WHERE k.barang_id=?1 AND p.batal=0
           UNION ALL
           SELECT t.tanggal, 2, bk.id, 'Keluar', 'Transaksi #' || t.id || ' (kiriman ' || k.tanggal_masuk || ')', -bk.jumlah
             FROM baris_keluar bk JOIN transaksi_keluar t ON t.id=bk.transaksi_id JOIN kiriman k ON k.id=bk.kiriman_id
             WHERE k.barang_id=?1 AND t.batal=0
         ) WHERE tanggal <= ?2 ORDER BY tanggal, urut, id",
    )?;
    let (mut saldo, mut saldo_awal, mut masuk, mut keluar) = (0i64, 0i64, 0i64, 0i64);
    let baris = q.query_map(params![barang_id, sampai], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(3)?, r.get::<_, String>(4)?, r.get::<_, i64>(5)?)))?;
    for b in baris {
        let (tgl, jenis, ket, ubah) = b?;
        saldo += ubah;
        if tgl.as_str() < dari {
            saldo_awal = saldo;
            continue;
        }
        let (sel_masuk, sel_keluar) = if ubah >= 0 {
            masuk += ubah;
            (Sel::Jumlah(ubah), Sel::Kosong)
        } else {
            keluar -= ubah;
            (Sel::Kosong, Sel::Jumlah(-ubah))
        };
        l.baris.push(vec![Sel::Tanggal(tgl), teks(jenis), if ket.trim().is_empty() { Sel::Kosong } else { teks(ket.trim()) }, sel_masuk, sel_keluar, Sel::Jumlah(saldo)]);
    }
    l.total = Some(baris_total(6, &[(3, Sel::Jumlah(masuk)), (4, Sel::Jumlah(keluar)), (5, Sel::Jumlah(saldo))]));
    l.ringkasan.push(("Saldo awal".into(), Sel::Jumlah(saldo_awal)));
    l.ringkasan.push(("Total masuk".into(), Sel::Jumlah(masuk)));
    l.ringkasan.push(("Total keluar".into(), Sel::Jumlah(keluar)));
    l.ringkasan.push(("Saldo akhir".into(), Sel::Jumlah(saldo)));
    Ok(())
}

fn barang_masuk(conn: &Connection, l: &mut Laporan, dari: &str, sampai: &str) -> Hasil<()> {
    l.judul = "Laporan Barang Masuk".into();
    l.kolom = kolom(&["Tanggal", "Barang", "Jumlah", "Satuan", "Harga beli", "Total", "Pemasok", "Spek"]);
    let mut q = conn.prepare(
        "SELECT k.tanggal_masuk, b.nama, k.jumlah_awal, b.satuan, k.harga_beli, k.pemasok, k.spek, k.stok_awal
         FROM kiriman k JOIN barang b ON b.id=k.barang_id WHERE k.batal=0 AND k.tanggal_masuk BETWEEN ?1 AND ?2 ORDER BY k.tanggal_masuk, k.id",
    )?;
    let mut total = 0;
    let baris = q.query_map(params![dari, sampai], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, i64>(2)?, r.get::<_, String>(3)?, r.get::<_, i64>(4)?, r.get::<_, Option<String>>(5)?, r.get::<_, Option<String>>(6)?, r.get::<_, bool>(7)?))
    })?;
    for b in baris {
        let (tgl, nama, jumlah, satuan, harga, pemasok, spek, stok_awal) = b?;
        let n = nilai(jumlah, harga);
        total += n;
        let pemasok = if stok_awal { Some(pemasok.map(|p| format!("{p} (stok awal)")).unwrap_or_else(|| "Stok awal".into())) } else { pemasok };
        l.baris.push(vec![Sel::Tanggal(tgl), teks(nama), Sel::Jumlah(jumlah), teks(satuan), Sel::Rupiah(harga), Sel::Rupiah(n), teks_opsional(pemasok), teks_opsional(spek)]);
    }
    l.total = Some(baris_total(8, &[(5, Sel::Rupiah(total))]));
    l.ringkasan.push(("Total nilai barang masuk".into(), Sel::Rupiah(total)));
    Ok(())
}

/// (tanggal, barang, tanggal masuk kiriman, jumlah, satuan, harga jual, harga beli) tiap baris keluar yang tidak batal.
type BarisJual = (String, String, String, i64, String, i64, i64);

fn baris_jual(conn: &Connection, dari: &str, sampai: &str) -> Hasil<Vec<BarisJual>> {
    let mut q = conn.prepare(
        "SELECT t.tanggal, b.nama, k.tanggal_masuk, bk.jumlah, b.satuan, bk.harga_jual, k.harga_beli
         FROM baris_keluar bk JOIN transaksi_keluar t ON t.id=bk.transaksi_id JOIN kiriman k ON k.id=bk.kiriman_id JOIN barang b ON b.id=k.barang_id
         WHERE t.batal=0 AND t.tanggal BETWEEN ?1 AND ?2 ORDER BY t.tanggal, t.id, bk.id",
    )?;
    let v = q
        .query_map(params![dari, sampai], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?)))?
        .collect::<Result<_, _>>()?;
    Ok(v)
}

fn barang_keluar(conn: &Connection, l: &mut Laporan, dari: &str, sampai: &str, harga: bool) -> Hasil<()> {
    l.judul = "Laporan Barang Keluar".into();
    let mut nama = vec!["Tanggal", "Barang", "Kiriman", "Jumlah", "Satuan", "Harga jual", "Subtotal"];
    if harga {
        nama.extend(["Harga beli", "Laba"]);
    }
    l.kolom = kolom(&nama);
    let (mut total, mut laba_total) = (0, 0);
    for (tgl, barang, masuk, jumlah, satuan, harga_jual, harga_beli) in baris_jual(conn, dari, sampai)? {
        let sub = nilai(jumlah, harga_jual);
        let laba = sub - nilai(jumlah, harga_beli);
        total += sub;
        laba_total += laba;
        let mut baris = vec![Sel::Tanggal(tgl), teks(barang), Sel::Tanggal(masuk), Sel::Jumlah(jumlah), teks(satuan), Sel::Rupiah(harga_jual), Sel::Rupiah(sub)];
        if harga {
            baris.extend([Sel::Rupiah(harga_beli), Sel::Rupiah(laba)]);
        }
        l.baris.push(baris);
    }
    let mut isi = vec![(6, Sel::Rupiah(total))];
    if harga {
        isi.push((8, Sel::Rupiah(laba_total)));
    }
    l.total = Some(baris_total(l.kolom.len(), &isi));
    l.ringkasan.push(("Total penjualan".into(), Sel::Rupiah(total)));
    Ok(())
}

fn laba_kotor(conn: &Connection, l: &mut Laporan, dari: &str, sampai: &str) -> Hasil<()> {
    l.judul = "Laporan Laba Kotor".into();
    l.kolom = kolom(&["Tanggal", "Barang", "Jumlah", "Satuan", "Harga jual", "Harga beli", "Penjualan", "Harga pokok", "Laba"]);
    let (mut jual, mut pokok) = (0, 0);
    for (tgl, barang, _, jumlah, satuan, harga_jual, harga_beli) in baris_jual(conn, dari, sampai)? {
        let (sub, hp) = (nilai(jumlah, harga_jual), nilai(jumlah, harga_beli));
        jual += sub;
        pokok += hp;
        l.baris.push(vec![Sel::Tanggal(tgl), teks(barang), Sel::Jumlah(jumlah), teks(satuan), Sel::Rupiah(harga_jual), Sel::Rupiah(harga_beli), Sel::Rupiah(sub), Sel::Rupiah(hp), Sel::Rupiah(sub - hp)]);
    }
    l.total = Some(baris_total(9, &[(6, Sel::Rupiah(jual)), (7, Sel::Rupiah(pokok)), (8, Sel::Rupiah(jual - pokok))]));
    let rugi: i64 = conn.query_row(
        "SELECT COALESCE(SUM((-p.selisih * k.harga_beli + 500) / 1000),0) FROM penyesuaian_stok p JOIN kiriman k ON k.id=p.kiriman_id
         WHERE p.batal=0 AND p.selisih<0 AND p.tanggal BETWEEN ?1 AND ?2",
        params![dari, sampai],
        |r| r.get(0),
    )?;
    l.ringkasan.push(("Penjualan".into(), Sel::Rupiah(jual)));
    l.ringkasan.push(("Harga pokok".into(), Sel::Rupiah(pokok)));
    l.ringkasan.push(("Laba kotor".into(), Sel::Rupiah(jual - pokok)));
    l.ringkasan.push(("Kerugian stok".into(), Sel::Rupiah(rugi)));
    l.ringkasan.push(("Laba setelah kerugian stok".into(), Sel::Rupiah(jual - pokok - rugi)));
    Ok(())
}
