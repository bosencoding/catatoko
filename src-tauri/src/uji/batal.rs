use super::{barang as uji_barang, db_uji, folder_data, hitung, jual, jual_tiga, kasir, masukan_masuk, pemilik, sisa, skenario, staf, HARI_INI};
use crate::akses::{Modul, Tingkat};
use crate::stok::{self, PerbaikanKiriman};
use crate::{barang, kas};

fn perbaikan(harga: i64) -> PerbaikanKiriman {
    PerbaikanKiriman { harga_beli: harga, spek: None, kedaluwarsa: None, pemasok: None, foto_sumber: None }
}

fn log_terakhir(c: &rusqlite::Connection, tindakan: &str) -> String {
    c.query_row("SELECT ringkasan FROM log_aktivitas WHERE tindakan=?1 ORDER BY id DESC LIMIT 1", [tindakan], |r| r.get(0)).unwrap()
}

#[test]
fn batal_keluar_mengembalikan_stok_dan_kas() {
    let s = skenario();
    let id = jual_tiga(&s);
    assert_eq!(stok::batal_keluar(&s.c, &pemilik(), id, " ").unwrap_err().kode, "VALIDASI");
    stok::batal_keluar(&s.c, &pemilik(), id, "salah harga").unwrap();
    assert_eq!((sisa(&s.c, s.sep), sisa(&s.c, s.okt), sisa(&s.c, s.nov)), (2_000, 10_000, 20_000));
    assert_eq!(hitung(&s.c, "SELECT batal FROM kas WHERE ref_jenis='transaksi_keluar'"), 1);
    assert_eq!(kas::saldo(&s.c, &pemilik()).unwrap().total, 0);
    let t = &stok::daftar_keluar(&s.c, &pemilik(), HARI_INI, HARI_INI).unwrap()[0];
    assert!(t.batal);
    assert_eq!(t.alasan_batal.as_deref(), Some("salah harga"));
    assert_eq!(t.baris.len(), 2, "rincian tetap terlihat");
    assert!(log_terakhir(&s.c, "keluar.batal").contains("salah harga"));
    // kiriman tidak lagi dianggap pernah keluar
    assert!(stok::kiriman_barang(&s.c, &pemilik(), s.barang, true, HARI_INI).unwrap().iter().all(|k| !k.pernah_keluar));
}

#[test]
fn batal_dua_kali_ditolak() {
    let s = skenario();
    let id = jual_tiga(&s);
    stok::batal_keluar(&s.c, &pemilik(), id, "salah harga").unwrap();
    assert_eq!(stok::batal_keluar(&s.c, &pemilik(), id, "salah harga").unwrap_err().kode, "VALIDASI");
    assert_eq!(sisa(&s.c, s.sep), 2_000);
    assert_eq!(stok::batal_keluar(&s.c, &pemilik(), 999, "salah harga").unwrap_err().kode, "TIDAK_DITEMUKAN");
}

#[test]
fn batal_masuk_belum_keluar() {
    let (c, d) = (db_uji(), folder_data());
    let b = barang::tambah(&c, d.path(), &pemilik(), uji_barang::masukan("Pakan")).unwrap();
    let k = stok::masuk(&c, d.path(), &pemilik(), masukan_masuk(b, "2026-10-15", 10_000, 392_000)).unwrap();
    assert_eq!(kas::saldo(&c, &pemilik()).unwrap().total, -3_920_000);
    stok::batal_masuk(&c, &pemilik(), k, "salah barang").unwrap();
    assert_eq!(sisa(&c, k), 0);
    assert_eq!(hitung(&c, "SELECT batal FROM kiriman"), 1);
    assert_eq!(kas::saldo(&c, &pemilik()).unwrap().total, 0);
    assert_eq!(barang::ambil(&c, &pemilik(), b).unwrap().stok, 0);
    assert!(stok::kiriman_barang(&c, &pemilik(), b, true, HARI_INI).unwrap().is_empty());
    assert_eq!(stok::batal_masuk(&c, &pemilik(), k, "salah barang").unwrap_err().kode, "VALIDASI");
    assert_eq!(stok::keluar(&c, &pemilik(), jual(b, 1, &[(k, 1_000)]), HARI_INI).unwrap_err().kode, "VALIDASI");
}

#[test]
fn batal_masuk_sudah_keluar_ditolak() {
    let s = skenario();
    let id = jual_tiga(&s);
    let e = stok::batal_masuk(&s.c, &pemilik(), s.sep, "salah barang").unwrap_err();
    assert_eq!(e.kode, "VALIDASI");
    assert_eq!(hitung(&s.c, "SELECT batal FROM kiriman WHERE id=1"), 0);
    stok::batal_keluar(&s.c, &pemilik(), id, "salah harga").unwrap();
    stok::batal_masuk(&s.c, &pemilik(), s.sep, "salah barang").unwrap();
    assert_eq!(sisa(&s.c, s.sep), 0);
}

#[test]
fn perbaiki_harga_menyesuaikan_kas() {
    let (c, d) = (db_uji(), folder_data());
    let p = pemilik();
    let b = barang::tambah(&c, d.path(), &p, uji_barang::masukan("Pakan")).unwrap();
    let k = stok::masuk(&c, d.path(), &p, masukan_masuk(b, "2026-10-15", 10_000, 392_000)).unwrap();
    stok::keluar(&c, &p, jual(b, 450_000, &[(k, 1_000)]), HARI_INI).unwrap();
    assert_eq!(stok::daftar_keluar(&c, &p, HARI_INI, HARI_INI).unwrap()[0].laba, Some(58_000));
    let mut pb = perbaikan(390_000);
    pb.spek = Some("protein 31%".into());
    pb.kedaluwarsa = Some("2027-03-31".into());
    stok::perbaiki_kiriman(&c, d.path(), &p, k, pb).unwrap();
    assert_eq!(hitung(&c, "SELECT jumlah FROM kas WHERE ref_jenis='kiriman' AND batal=0"), 3_900_000);
    assert_eq!(hitung(&c, "SELECT count(*) FROM kas WHERE ref_jenis='kiriman'"), 1);
    assert_eq!(stok::daftar_keluar(&c, &p, HARI_INI, HARI_INI).unwrap()[0].laba, Some(60_000));
    let kr = &stok::kiriman_barang(&c, &p, b, true, HARI_INI).unwrap()[0];
    assert_eq!((kr.harga_beli, kr.spek.as_deref(), kr.kedaluwarsa.as_deref(), kr.sisa), (Some(390_000), Some("protein 31%"), Some("2027-03-31"), 9_000));
    let log = log_terakhir(&c, "kiriman.perbaiki");
    assert!(log.contains("392000") && log.contains("390000") && log.contains("protein 31%"), "{log}");
}

#[test]
fn perbaiki_stok_awal_tanpa_kas() {
    let s = skenario();
    stok::perbaiki_kiriman(&s.c, s.d.path(), &pemilik(), s.sep, perbaikan(380_000)).unwrap();
    assert_eq!(hitung(&s.c, "SELECT count(*) FROM kas"), 0);
    assert_eq!(hitung(&s.c, "SELECT harga_beli FROM kiriman WHERE id=1"), 380_000);
}

#[test]
fn perbaiki_dari_harga_nol_membuat_kas() {
    let (c, d) = (db_uji(), folder_data());
    let p = pemilik();
    let b = barang::tambah(&c, d.path(), &p, uji_barang::masukan("Pakan")).unwrap();
    let k = stok::masuk(&c, d.path(), &p, masukan_masuk(b, "2026-10-15", 2_000, 0)).unwrap();
    assert_eq!(hitung(&c, "SELECT count(*) FROM kas"), 0);
    stok::perbaiki_kiriman(&c, d.path(), &p, k, perbaikan(100_000)).unwrap();
    assert_eq!(hitung(&c, "SELECT jumlah FROM kas WHERE ref_jenis='kiriman' AND batal=0"), 200_000);
    stok::perbaiki_kiriman(&c, d.path(), &p, k, perbaikan(0)).unwrap();
    assert_eq!(hitung(&c, "SELECT count(*) FROM kas WHERE batal=0"), 0);
}

#[test]
fn perbaiki_tak_sah_ditolak() {
    let s = skenario();
    let p = pemilik();
    assert_eq!(stok::perbaiki_kiriman(&s.c, s.d.path(), &p, s.sep, perbaikan(-1)).unwrap_err().kode, "VALIDASI");
    let mut pb = perbaikan(1);
    pb.kedaluwarsa = Some("2026-09-02".into());
    assert_eq!(stok::perbaiki_kiriman(&s.c, s.d.path(), &p, s.sep, pb).unwrap_err().kode, "VALIDASI");
    assert_eq!(stok::perbaiki_kiriman(&s.c, s.d.path(), &p, 999, perbaikan(1)).unwrap_err().kode, "TIDAK_DITEMUKAN");
    s.c.execute("UPDATE kiriman SET batal=1 WHERE id=?1", [s.nov]).unwrap();
    assert_eq!(stok::perbaiki_kiriman(&s.c, s.d.path(), &p, s.nov, perbaikan(1)).unwrap_err().kode, "VALIDASI");
    assert_eq!(hitung(&s.c, "SELECT harga_beli FROM kiriman WHERE id=1"), 385_000);
}

#[test]
fn sesuaikan_kurang_dan_tambah() {
    let s = skenario();
    let p = pemilik();
    let a = stok::sesuaikan(&s.c, &p, s.okt, HARI_INI, -1_000, "rusak", Some("karung sobek")).unwrap();
    assert_eq!(sisa(&s.c, s.okt), 9_000);
    stok::sesuaikan(&s.c, &p, s.okt, HARI_INI, 500, "koreksi", None).unwrap();
    assert_eq!(sisa(&s.c, s.okt), 9_500);
    assert_eq!(hitung(&s.c, "SELECT count(*) FROM kas"), 0);
    let v = stok::daftar_penyesuaian(&s.c, &p, HARI_INI, HARI_INI).unwrap();
    assert_eq!(v.len(), 2);
    let rusak = v.iter().find(|x| x.id == a).unwrap();
    assert_eq!((rusak.selisih, rusak.alasan.as_str(), rusak.nilai, rusak.barang_nama.as_str()), (-1_000, "rusak", Some(-392_000), "Pakan Apung 781-2"));
    assert!(stok::kiriman_barang(&s.c, &p, s.barang, true, HARI_INI).unwrap()[1].pernah_keluar);
    assert_eq!(stok::batal_masuk(&s.c, &p, s.okt, "coba batal").unwrap_err().kode, "VALIDASI");
}

#[test]
fn sesuaikan_tak_sah_ditolak() {
    let s = skenario();
    let p = pemilik();
    let e = stok::sesuaikan(&s.c, &p, s.sep, HARI_INI, -2_001, "hilang", None).unwrap_err();
    assert_eq!(e.kode, "STOK_KURANG");
    assert_eq!(stok::sesuaikan(&s.c, &p, s.sep, HARI_INI, -1_000, "dicuri", None).unwrap_err().kode, "VALIDASI");
    assert_eq!(stok::sesuaikan(&s.c, &p, s.sep, HARI_INI, 0, "koreksi", None).unwrap_err().kode, "VALIDASI");
    assert_eq!(stok::sesuaikan(&s.c, &p, s.sep, "hari ini", -1_000, "rusak", None).unwrap_err().kode, "VALIDASI");
    assert_eq!(stok::sesuaikan(&s.c, &p, 999, HARI_INI, -1_000, "rusak", None).unwrap_err().kode, "TIDAK_DITEMUKAN");
    assert_eq!(sisa(&s.c, s.sep), 2_000);
    assert_eq!(hitung(&s.c, "SELECT count(*) FROM penyesuaian_stok"), 0);
}

#[test]
fn batal_penyesuaian_membalik() {
    let s = skenario();
    let p = pemilik();
    let kurang = stok::sesuaikan(&s.c, &p, s.sep, HARI_INI, -1_000, "rusak", None).unwrap();
    stok::batal_penyesuaian(&s.c, &p, kurang, "ternyata utuh").unwrap();
    assert_eq!(sisa(&s.c, s.sep), 2_000);
    assert_eq!(stok::batal_penyesuaian(&s.c, &p, kurang, "ternyata utuh").unwrap_err().kode, "VALIDASI");
    // pembalikan yang membuat sisa negatif ditolak
    let tambah = stok::sesuaikan(&s.c, &p, s.sep, HARI_INI, 1_000, "koreksi", None).unwrap();
    stok::keluar(&s.c, &p, jual(s.barang, 1, &[(s.sep, 3_000)]), HARI_INI).unwrap();
    assert_eq!(stok::batal_penyesuaian(&s.c, &p, tambah, "salah hitung").unwrap_err().kode, "STOK_KURANG");
    assert_eq!(sisa(&s.c, s.sep), 0);
}

#[test]
fn hak_batal_dan_perbaikan() {
    let s = skenario();
    let id = jual_tiga(&s);
    let k = kasir();
    assert_eq!(stok::batal_keluar(&s.c, &k, id, "coba batal").unwrap_err().kode, "TIDAK_BERHAK");
    assert_eq!(stok::batal_masuk(&s.c, &k, s.nov, "coba batal").unwrap_err().kode, "TIDAK_BERHAK");
    assert_eq!(stok::perbaiki_kiriman(&s.c, s.d.path(), &k, s.nov, perbaikan(1)).unwrap_err().kode, "TIDAK_BERHAK");
    assert_eq!(stok::sesuaikan(&s.c, &k, s.nov, HARI_INI, -1_000, "rusak", None).unwrap_err().kode, "TIDAK_BERHAK");
    assert_eq!(stok::daftar_penyesuaian(&s.c, &k, HARI_INI, HARI_INI).unwrap_err().kode, "TIDAK_BERHAK");
    let tambah = staf(&[(Modul::BarangMasuk, Tingkat::Tambah), (Modul::BarangKeluar, Tingkat::Tambah)], true);
    assert_eq!(stok::batal_keluar(&s.c, &tambah, id, "coba batal").unwrap_err().kode, "TIDAK_BERHAK");
    assert_eq!(stok::sesuaikan(&s.c, &tambah, s.nov, HARI_INI, -1_000, "rusak", None).unwrap_err().kode, "TIDAK_BERHAK");
    assert!(stok::batal_keluar(&s.c, &staf(&[(Modul::BarangKeluar, Tingkat::Ubah)], false), id, "boleh batal").is_ok());
}

#[test]
fn semua_tercatat_log() {
    let s = skenario();
    let p = pemilik();
    let id = jual_tiga(&s);
    stok::batal_keluar(&s.c, &p, id, "salah harga").unwrap();
    stok::perbaiki_kiriman(&s.c, s.d.path(), &p, s.nov, perbaikan(399_000)).unwrap();
    let a = stok::sesuaikan(&s.c, &p, s.nov, HARI_INI, -1_000, "rusak", None).unwrap();
    stok::batal_penyesuaian(&s.c, &p, a, "ternyata utuh").unwrap();
    stok::batal_masuk(&s.c, &p, s.nov, "salah barang").unwrap();
    for t in ["keluar.batal", "kiriman.perbaiki", "stok.sesuaikan", "stok.sesuaikan_batal", "masuk.batal"] {
        assert_eq!(hitung(&s.c, &format!("SELECT count(*) FROM log_aktivitas WHERE tindakan='{t}'")), 1, "{t}");
    }
}
