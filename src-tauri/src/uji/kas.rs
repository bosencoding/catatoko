use super::{db_uji, hitung, kasir, pemilik, staf};
use crate::akses::{Modul, Tingkat};
use crate::kas::{self, KasMasukan, SaringKas};
use rusqlite::Connection;

fn kat(c: &Connection, nama: &str) -> i64 {
    c.query_row("SELECT id FROM kategori_kas WHERE nama=?1", [nama], |r| r.get(0)).unwrap()
}

pub fn masukan(c: &Connection, tanggal: &str, arah: &str, kategori: &str, jumlah: i64) -> KasMasukan {
    KasMasukan { tanggal: tanggal.into(), arah: arah.into(), akun_kas_id: 1, kategori_id: kat(c, kategori), jumlah, keterangan: None }
}

fn semua() -> SaringKas {
    SaringKas { dari: "2026-01-01".into(), sampai: "2026-12-31".into(), akun_kas_id: None, kategori_id: None, arah: None }
}

/// Modal 1.000.000 (1 Nov) lalu Listrik 250.000 (5 Nov).
fn dasar() -> (Connection, i64, i64) {
    let c = db_uji();
    let a = kas::tambah_manual(&c, &pemilik(), masukan(&c, "2026-11-01", "masuk", "Modal", 1_000_000)).unwrap();
    let b = kas::tambah_manual(&c, &pemilik(), masukan(&c, "2026-11-05", "keluar", "Listrik/Air", 250_000)).unwrap();
    (c, a, b)
}

#[test]
fn saldo_berjalan() {
    let (c, _, _) = dasar();
    let v = kas::daftar(&c, &pemilik(), &semua()).unwrap();
    assert_eq!(v.len(), 2);
    assert_eq!((v[0].jumlah, v[0].saldo), (1_000_000, 1_000_000));
    assert_eq!((v[1].jumlah, v[1].saldo), (250_000, 750_000));
    assert_eq!(v[1].kategori, "Listrik/Air");
    assert_eq!(v[1].akun, "Tunai");
    assert_eq!(v[1].dibuat_oleh_nama.as_deref(), Some("Pemilik Uji"));
    let s = kas::saldo(&c, &pemilik()).unwrap();
    assert_eq!(s.total, 750_000);
    assert_eq!(s.per_akun.len(), 1);
    assert_eq!(s.per_akun[0].saldo, 750_000);
}

#[test]
fn saldo_berjalan_membawa_saldo_sebelum_periode() {
    let (c, _, _) = dasar();
    let mut f = semua();
    f.dari = "2026-11-03".into();
    let v = kas::daftar(&c, &pemilik(), &f).unwrap();
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].saldo, 750_000);
}

#[test]
fn batal_tak_dihitung() {
    let (c, _, b) = dasar();
    assert_eq!(kas::batal_manual(&c, &pemilik(), b, "x").unwrap_err().kode, "VALIDASI", "alasan terlalu pendek");
    kas::batal_manual(&c, &pemilik(), b, "salah input").unwrap();
    assert_eq!(kas::saldo(&c, &pemilik()).unwrap().total, 1_000_000);
    let v = kas::daftar(&c, &pemilik(), &semua()).unwrap();
    assert_eq!(v.len(), 2);
    assert!(v[1].batal);
    assert_eq!(v[1].saldo, 1_000_000);
    assert_eq!(kas::batal_manual(&c, &pemilik(), b, "lagi lagi").unwrap_err().kode, "VALIDASI");
}

#[test]
fn ubah_manual_mengubah_saldo() {
    let (c, _, b) = dasar();
    kas::ubah_manual(&c, &pemilik(), b, masukan(&c, "2026-11-06", "keluar", "Gaji", 300_000)).unwrap();
    let v = kas::daftar(&c, &pemilik(), &semua()).unwrap();
    assert_eq!((v[1].tanggal.as_str(), v[1].kategori.as_str(), v[1].saldo), ("2026-11-06", "Gaji", 700_000));
    let log: String = c.query_row("SELECT ringkasan FROM log_aktivitas WHERE tindakan='kas.ubah'", [], |r| r.get(0)).unwrap();
    assert!(log.contains("250000") && log.contains("300000"), "{log}");
}

#[test]
fn pindah_tak_mengubah_total() {
    let (c, _, _) = dasar();
    let bank = kas::akun_tambah(&c, &pemilik(), "Bank").unwrap();
    kas::pindah(&c, &pemilik(), "2026-11-10", 1, bank, 400_000, Some("setor")).unwrap();
    let s = kas::saldo(&c, &pemilik()).unwrap();
    assert_eq!(s.total, 750_000);
    let per: Vec<(String, i64)> = s.per_akun.iter().map(|a| (a.nama.clone(), a.saldo)).collect();
    assert!(per.contains(&("Tunai".into(), 350_000)) && per.contains(&("Bank".into(), 400_000)), "{per:?}");
    let v = kas::daftar(&c, &pemilik(), &semua()).unwrap();
    assert_eq!(v.len(), 4);
    assert_eq!(v[3].saldo, 750_000);
    assert_eq!(v[3].ref_jenis.as_deref(), Some("pindah"));
    // saring per akun: saldo berjalan akun itu
    let mut f = semua();
    f.akun_kas_id = Some(bank);
    let b = kas::daftar(&c, &pemilik(), &f).unwrap();
    assert_eq!(b.len(), 1);
    assert_eq!((b[0].arah.as_str(), b[0].saldo), ("masuk", 400_000));
}

#[test]
fn batal_pindah_membatalkan_kedua_sisi() {
    let (c, _, _) = dasar();
    let bank = kas::akun_tambah(&c, &pemilik(), "Bank").unwrap();
    let id = kas::pindah(&c, &pemilik(), "2026-11-10", 1, bank, 400_000, None).unwrap();
    assert_eq!(kas::ubah_manual(&c, &pemilik(), id, masukan(&c, "2026-11-10", "keluar", "Gaji", 1)).unwrap_err().kode, "VALIDASI");
    kas::batal_manual(&c, &pemilik(), id, "salah akun").unwrap();
    let s = kas::saldo(&c, &pemilik()).unwrap();
    assert_eq!(s.total, 750_000);
    assert!(s.per_akun.iter().all(|a| a.nama != "Bank" || a.saldo == 0));
    assert_eq!(hitung(&c, "SELECT count(*) FROM kas WHERE ref_jenis='pindah' AND batal=1"), 2);
}

#[test]
fn pindah_tak_sah_ditolak() {
    let (c, _, _) = dasar();
    let bank = kas::akun_tambah(&c, &pemilik(), "Bank").unwrap();
    let e = kas::pindah(&c, &pemilik(), "2026-11-10", 1, bank, 750_001, None).unwrap_err();
    assert_eq!(e.kode, "VALIDASI");
    assert!(e.pesan.contains("Tunai"), "{}", e.pesan);
    assert_eq!(kas::pindah(&c, &pemilik(), "2026-11-10", 1, 1, 1000, None).unwrap_err().kode, "VALIDASI");
    assert_eq!(kas::pindah(&c, &pemilik(), "2026-11-10", 1, bank, 0, None).unwrap_err().kode, "VALIDASI");
    assert_eq!(kas::pindah(&c, &pemilik(), "2026-11-10", 1, 99, 1000, None).unwrap_err().kode, "VALIDASI");
    assert_eq!(hitung(&c, "SELECT count(*) FROM kas"), 2);
}

#[test]
fn isian_tak_sah_ditolak() {
    let c = db_uji();
    let p = pemilik();
    for jumlah in [0, -5] {
        assert_eq!(kas::tambah_manual(&c, &p, masukan(&c, "2026-11-01", "masuk", "Modal", jumlah)).unwrap_err().kode, "VALIDASI");
    }
    for tgl in ["2026-13-40", "24/11/2026", ""] {
        assert_eq!(kas::tambah_manual(&c, &p, masukan(&c, tgl, "masuk", "Modal", 1000)).unwrap_err().kode, "VALIDASI");
    }
    assert_eq!(kas::tambah_manual(&c, &p, masukan(&c, "2026-11-01", "samping", "Modal", 1000)).unwrap_err().kode, "VALIDASI");
    let mut m = masukan(&c, "2026-11-01", "masuk", "Modal", 1000);
    m.akun_kas_id = 99;
    assert_eq!(kas::tambah_manual(&c, &p, m).unwrap_err().kode, "VALIDASI");
    let mut m = masukan(&c, "2026-11-01", "masuk", "Modal", 1000);
    m.kategori_id = 99;
    assert_eq!(kas::tambah_manual(&c, &p, m).unwrap_err().kode, "VALIDASI");
    assert_eq!(hitung(&c, "SELECT count(*) FROM kas"), 0);
}

#[test]
fn kategori_bawaan_tak_bisa_manual() {
    let c = db_uji();
    for k in ["Penjualan", "Pembelian barang", "Pindah akun"] {
        assert_eq!(kas::tambah_manual(&c, &pemilik(), masukan(&c, "2026-11-01", "masuk", k, 1000)).unwrap_err().kode, "VALIDASI");
    }
}

#[test]
fn otomatis_tak_bisa_diubah_manual() {
    let (c, _, _) = dasar();
    let id = kas::catat_otomatis(&c, "2026-11-07", "masuk", 1, kas::KAT_PENJUALAN, 450_000, "Penjualan #1", "transaksi_keluar", 1, 1).unwrap();
    assert_eq!(kas::ubah_manual(&c, &pemilik(), id, masukan(&c, "2026-11-07", "masuk", "Modal", 1)).unwrap_err().kode, "VALIDASI");
    assert_eq!(kas::batal_manual(&c, &pemilik(), id, "coba batal").unwrap_err().kode, "VALIDASI");
    assert_eq!(kas::saldo(&c, &pemilik()).unwrap().total, 1_200_000);
    assert!(kas::sesuaikan_otomatis(&c, "transaksi_keluar", 1, 400_000).unwrap());
    assert_eq!(kas::saldo(&c, &pemilik()).unwrap().total, 1_150_000);
    kas::batal_otomatis(&c, "transaksi_keluar", 1).unwrap();
    assert_eq!(kas::saldo(&c, &pemilik()).unwrap().total, 750_000);
    assert!(!kas::sesuaikan_otomatis(&c, "kiriman", 77, 5).unwrap());
}

#[test]
fn saring_kategori_dan_arah() {
    let (c, _, _) = dasar();
    let mut f = semua();
    f.arah = Some("keluar".into());
    let v = kas::daftar(&c, &pemilik(), &f).unwrap();
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].saldo, 750_000, "saldo berjalan tetap dihitung dari semua baris");
    let mut f = semua();
    f.kategori_id = Some(kat(&c, "Modal"));
    assert_eq!(kas::daftar(&c, &pemilik(), &f).unwrap().len(), 1);
}

#[test]
fn rentang_terbalik_ditolak() {
    let c = db_uji();
    let mut f = semua();
    f.dari = "2026-12-01".into();
    f.sampai = "2026-11-01".into();
    assert_eq!(kas::daftar(&c, &pemilik(), &f).unwrap_err().kode, "VALIDASI");
}

#[test]
fn hak_kas() {
    let (c, a, _) = dasar();
    assert_eq!(kas::daftar(&c, &kasir(), &semua()).unwrap_err().kode, "TIDAK_BERHAK");
    assert_eq!(kas::saldo(&c, &kasir()).unwrap_err().kode, "TIDAK_BERHAK");
    let lihat = staf(&[(Modul::Kas, Tingkat::Lihat)], false);
    assert!(kas::daftar(&c, &lihat, &semua()).is_ok());
    assert_eq!(kas::tambah_manual(&c, &lihat, masukan(&c, "2026-11-01", "masuk", "Modal", 1)).unwrap_err().kode, "TIDAK_BERHAK");
    let tambah = staf(&[(Modul::Kas, Tingkat::Tambah)], false);
    assert!(kas::tambah_manual(&c, &tambah, masukan(&c, "2026-11-01", "masuk", "Modal", 1)).is_ok());
    assert_eq!(kas::batal_manual(&c, &tambah, a, "coba batal").unwrap_err().kode, "TIDAK_BERHAK");
    assert_eq!(kas::pindah(&c, &lihat, "2026-11-10", 1, 1, 1, None).unwrap_err().kode, "TIDAK_BERHAK");
    assert_eq!(kas::akun_tambah(&c, &tambah, "Bank").unwrap_err().kode, "TIDAK_BERHAK");
    assert_eq!(kas::kategori_tambah(&c, &tambah, "Sewa").unwrap_err().kode, "TIDAK_BERHAK");
}

#[test]
fn akun_dan_kategori() {
    let c = db_uji();
    let p = pemilik();
    assert_eq!(kas::akun_tambah(&c, &p, "tunai").unwrap_err().kode, "VALIDASI");
    let bank = kas::akun_tambah(&c, &p, "Bank").unwrap();
    kas::akun_ubah(&c, &p, bank, "Bank BRI", false).unwrap();
    assert_eq!(kas::akun_daftar(&c).unwrap().iter().filter(|a| a.aktif).count(), 1);
    assert_eq!(kas::akun_ubah(&c, &p, 1, "Tunai", false).unwrap_err().kode, "VALIDASI", "akun aktif terakhir");
    assert_eq!(kas::tambah_manual(&c, &p, KasMasukan { akun_kas_id: bank, ..masukan(&c, "2026-11-01", "masuk", "Modal", 1) }).unwrap_err().kode, "VALIDASI");
    let sewa = kas::kategori_tambah(&c, &p, "Sewa").unwrap();
    assert_eq!(kas::kategori_tambah(&c, &p, "sewa").unwrap_err().kode, "VALIDASI");
    assert!(kas::kategori_daftar(&c).unwrap().iter().any(|k| k.id == sewa && !k.bawaan));
}
