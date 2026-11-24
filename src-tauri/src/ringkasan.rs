//! Data beranda. Tiap bagian hanya diisi bila sesi berhak melihatnya.

use crate::akses::{Modul, Sesi, Tingkat};
use crate::barang::{self, Barang};
use crate::galat::{Galat, Hasil};
use crate::kas::{self, SaldoAkun};
use crate::laporan::nama_bulan;
use crate::stok::{kiriman_dari_baris, Kiriman, PILIH_KIRIMAN, URUT_KIRIMAN};
use crate::waktu;
use chrono::{Datelike, Duration, NaiveDate};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Butir {
    Hari,
    Minggu,
    Bulan,
}

#[derive(Debug, Serialize)]
pub struct RingkasKas {
    pub saldo_total: i64,
    pub saldo_per_akun: Vec<SaldoAkun>,
    /// Kas masuk/keluar dalam periode (pindah akun dan yang batal tidak dihitung).
    pub masuk: i64,
    pub keluar: i64,
}

#[derive(Debug, Serialize)]
pub struct TitikKas {
    pub label: String,
    pub masuk: i64,
    pub keluar: i64,
    pub saldo_akhir: i64,
}

#[derive(Debug, Serialize)]
pub struct TitikBarang {
    pub label: String,
    /// Per seperseribu satuan, dijumlahkan dari semua barang.
    pub masuk: i64,
    pub keluar: i64,
}

#[derive(Debug, Serialize)]
pub struct Ringkasan {
    pub kas: Option<RingkasKas>,
    pub nilai_stok: Option<i64>,
    /// Laba kotor penjualan dalam periode.
    pub laba: Option<i64>,
    pub deret_kas: Option<Vec<TitikKas>>,
    pub deret_barang: Vec<TitikBarang>,
    pub terlaris: Vec<(String, i64)>,
    pub stok_menipis: Vec<Barang>,
    pub hampir_kedaluwarsa: Vec<Kiriman>,
    pub stok_lama: Vec<Kiriman>,
}

const BATAS_TITIK: usize = 400;

/// Membagi rentang menjadi ember (awal, akhir, label).
fn ember(dari: NaiveDate, sampai: NaiveDate, butir: Butir) -> Hasil<Vec<(String, String, String)>> {
    let mut hasil = Vec::new();
    let mut awal = dari;
    while awal <= sampai {
        let (akhir, label) = match butir {
            Butir::Hari => (awal, format!("{:02} {}", awal.day(), nama_bulan(awal))),
            Butir::Minggu => (awal + Duration::days(6), format!("{:02} {}", awal.day(), nama_bulan(awal))),
            Butir::Bulan => {
                let (y, m) = if awal.month() == 12 { (awal.year() + 1, 1) } else { (awal.year(), awal.month() + 1) };
                let akhir_bulan = NaiveDate::from_ymd_opt(y, m, 1).expect("tanggal sah") - Duration::days(1);
                (akhir_bulan, format!("{} {}", nama_bulan(awal), awal.year()))
            }
        };
        let akhir = akhir.min(sampai);
        hasil.push((awal.to_string(), akhir.to_string(), label));
        if hasil.len() > BATAS_TITIK {
            return Err(Galat::validasi("Rentang tanggal terlalu panjang untuk grafik harian. Persempit rentangnya atau pilih tampilan per minggu/bulan."));
        }
        awal = akhir + Duration::days(1);
    }
    Ok(hasil)
}

const TANDA: &str = "CASE arah WHEN 'masuk' THEN jumlah ELSE -jumlah END";
const BUKAN_PINDAH: &str = "batal=0 AND ref_jenis IS NOT 'pindah'";

pub fn ringkasan(conn: &Connection, s: &Sesi, dari: &str, sampai: &str, butir: Butir, hari_ini: &str) -> Hasil<Ringkasan> {
    let (a, b) = waktu::rentang(dari, sampai)?;
    let hari = waktu::tanggal(hari_ini)?;
    let ember = ember(a, b, butir)?;
    let lihat_kas = s.boleh(Modul::Kas, Tingkat::Lihat);
    let lihat_barang = s.boleh(Modul::Barang, Tingkat::Lihat) || s.boleh(Modul::BarangKeluar, Tingkat::Lihat) || s.boleh(Modul::BarangMasuk, Tingkat::Lihat);
    let harga = s.boleh_harga_beli();
    let jumlah = |sql: &str, p: &[&dyn rusqlite::ToSql]| -> Hasil<i64> { Ok(conn.query_row(sql, p, |r| r.get(0))?) };

    let (mut kas_ringkas, mut deret_kas) = (None, None);
    if lihat_kas {
        let saldo = kas::saldo_tanpa_cek(conn)?;
        let arah = |arah: &str| jumlah(&format!("SELECT COALESCE(SUM(jumlah),0) FROM kas WHERE {BUKAN_PINDAH} AND arah=?1 AND tanggal BETWEEN ?2 AND ?3"), &[&arah, &dari, &sampai]);
        kas_ringkas = Some(RingkasKas { saldo_total: saldo.total, saldo_per_akun: saldo.per_akun, masuk: arah("masuk")?, keluar: arah("keluar")? });
        let mut berjalan = jumlah(&format!("SELECT COALESCE(SUM({TANDA}),0) FROM kas WHERE batal=0 AND tanggal < ?1"), &[&dari])?;
        let mut titik = Vec::with_capacity(ember.len());
        for (awal, akhir, label) in &ember {
            let arah = |arah: &str| jumlah(&format!("SELECT COALESCE(SUM(jumlah),0) FROM kas WHERE {BUKAN_PINDAH} AND arah=?1 AND tanggal BETWEEN ?2 AND ?3"), &[&arah, awal, akhir]);
            let (masuk, keluar) = (arah("masuk")?, arah("keluar")?);
            berjalan += masuk - keluar;
            titik.push(TitikKas { label: label.clone(), masuk, keluar, saldo_akhir: berjalan });
        }
        deret_kas = Some(titik);
    }

    let mut r = Ringkasan { kas: kas_ringkas, nilai_stok: None, laba: None, deret_kas, deret_barang: Vec::new(), terlaris: Vec::new(), stok_menipis: Vec::new(), hampir_kedaluwarsa: Vec::new(), stok_lama: Vec::new() };
    if !lihat_barang {
        return Ok(r);
    }
    const JUAL: &str = "FROM baris_keluar bk JOIN transaksi_keluar t ON t.id=bk.transaksi_id JOIN kiriman k ON k.id=bk.kiriman_id WHERE t.batal=0 AND t.tanggal BETWEEN ?1 AND ?2";
    if harga {
        r.nilai_stok = Some(jumlah("SELECT COALESCE(SUM((sisa*harga_beli+500)/1000),0) FROM kiriman WHERE batal=0", &[])?);
        r.laba = Some(jumlah(
            &format!("SELECT COALESCE(SUM((bk.jumlah*bk.harga_jual+500)/1000 - (bk.jumlah*k.harga_beli+500)/1000),0) {JUAL}"),
            &[&dari, &sampai],
        )?);
    }
    for (awal, akhir, label) in &ember {
        r.deret_barang.push(TitikBarang {
            label: label.clone(),
            masuk: jumlah("SELECT COALESCE(SUM(jumlah_awal),0) FROM kiriman WHERE batal=0 AND tanggal_masuk BETWEEN ?1 AND ?2", &[awal, akhir])?,
            keluar: jumlah(&format!("SELECT COALESCE(SUM(bk.jumlah),0) {JUAL}"), &[awal, akhir])?,
        });
    }
    let mut q = conn.prepare(&format!(
        "SELECT b.nama, SUM(bk.jumlah) FROM baris_keluar bk JOIN transaksi_keluar t ON t.id=bk.transaksi_id JOIN kiriman k ON k.id=bk.kiriman_id
         JOIN barang b ON b.id=k.barang_id WHERE t.batal=0 AND t.tanggal BETWEEN ?1 AND ?2 GROUP BY b.id ORDER BY 2 DESC, b.nama LIMIT 5"
    ))?;
    r.terlaris = q.query_map(params![dari, sampai], |x| Ok((x.get(0)?, x.get(1)?)))?.collect::<Result<_, _>>()?;
    r.stok_menipis = barang::daftar(conn, s, None, false)?.into_iter().filter(|b| b.stok_minimum > 0 && b.stok < b.stok_minimum).collect();
    let batas = (hari + Duration::days(30)).to_string();
    let mut q = conn.prepare(&format!(
        "{PILIH_KIRIMAN} WHERE k.batal=0 AND k.sisa>0 AND k.kedaluwarsa IS NOT NULL AND k.kedaluwarsa <= ?1 ORDER BY k.kedaluwarsa, k.id"
    ))?;
    r.hampir_kedaluwarsa = q.query_map([batas], |x| kiriman_dari_baris(x, harga, hari))?.collect::<Result<_, _>>()?;
    let mut q = conn.prepare(&format!("{PILIH_KIRIMAN} WHERE k.batal=0 AND k.sisa>0 ORDER BY {URUT_KIRIMAN} LIMIT 10"))?;
    r.stok_lama = q.query_map([], |x| kiriman_dari_baris(x, harga, hari))?.collect::<Result<_, _>>()?;
    Ok(r)
}
