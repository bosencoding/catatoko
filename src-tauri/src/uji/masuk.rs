use super::{barang as uji_barang, db_uji, folder_data, hitung, kasir, masukan_masuk, pemilik, skenario, staf, HARI_INI};
use crate::akses::{Modul, Tingkat};
use crate::{barang, kas, stok};
use rusqlite::Connection;

fn dengan_barang() -> (Connection, tempfile::TempDir, i64) {
    let (c, d) = (db_uji(), folder_data());
    let id = barang::tambah(&c, d.path(), &pemilik(), uji_barang::masukan("Pakan Apung 781-2")).unwrap();
    (c, d, id)
}

#[test]
fn masuk_membuat_kiriman_dan_kas() {
    let (c, d, b) = dengan_barang();
    let id = stok::masuk(&c, d.path(), &pemilik(), masukan_masuk(b, "2026-10-15", 10_000, 392_000)).unwrap();
    let k = &stok::kiriman_barang(&c, &pemilik(), b, true, HARI_INI).unwrap()[0];
    assert_eq!((k.id, k.jumlah_awal, k.sisa, k.harga_beli), (id, 10_000, 10_000, Some(392_000)));
    assert_eq!((k.barang_nama.as_str(), k.satuan.as_str()), ("Pakan Apung 781-2", "karung"));
    assert!(!k.pernah_keluar && !k.batal);
    let (arah, jumlah, kategori, sumber, ref_jenis, ref_id): (String, i64, String, String, String, i64) = c
        .query_row(
            "SELECT k.arah, k.jumlah, g.nama, k.sumber, k.ref_jenis, k.ref_id FROM kas k JOIN kategori_kas g ON g.id=k.kategori_id",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)),
        )
        .unwrap();
    assert_eq!((arah.as_str(), jumlah, kategori.as_str(), sumber.as_str(), ref_jenis.as_str(), ref_id), ("keluar", 3_920_000, kas::KAT_PEMBELIAN, "otomatis", "kiriman", id));
    assert_eq!(barang::ambil(&c, &pemilik(), b).unwrap().stok, 10_000);
    assert_eq!(hitung(&c, "SELECT count(*) FROM log_aktivitas WHERE tindakan='masuk.tambah'"), 1);
}

#[test]
fn jumlah_desimal() {
    let (c, d, b) = dengan_barang();
    stok::masuk(&c, d.path(), &pemilik(), masukan_masuk(b, "2026-10-15", 500, 15_000)).unwrap();
    assert_eq!(hitung(&c, "SELECT jumlah FROM kas"), 7_500);
}

#[test]
fn stok_awal_tanpa_kas() {
    let s = skenario();
    assert_eq!(hitung(&s.c, "SELECT count(*) FROM kas"), 0);
    assert_eq!(barang::ambil(&s.c, &pemilik(), s.barang).unwrap().stok, 32_000);
}

#[test]
fn harga_nol_tanpa_kas() {
    let (c, d, b) = dengan_barang();
    stok::masuk(&c, d.path(), &pemilik(), masukan_masuk(b, "2026-10-15", 1_000, 0)).unwrap();
    assert_eq!(hitung(&c, "SELECT count(*) FROM kas"), 0);
}

#[test]
fn urutan_kiriman() {
    let (c, d, b) = dengan_barang();
    let p = pemilik();
    let okt = stok::masuk(&c, d.path(), &p, masukan_masuk(b, "2026-10-15", 1_000, 1)).unwrap();
    let sep = stok::masuk(&c, d.path(), &p, masukan_masuk(b, "2026-09-03", 1_000, 1)).unwrap();
    let nov = stok::masuk(&c, d.path(), &p, masukan_masuk(b, "2026-11-20", 1_000, 1)).unwrap();
    // tanggal sama: tanpa kedaluwarsa terakhir, kedaluwarsa terdekat dulu
    let nov_tanpa = stok::masuk(&c, d.path(), &p, masukan_masuk(b, "2026-11-20", 1_000, 1)).unwrap();
    let mut m = masukan_masuk(b, "2026-11-20", 1_000, 1);
    m.kedaluwarsa = Some("2027-06-30".into());
    let nov_jauh = stok::masuk(&c, d.path(), &p, m.clone()).unwrap();
    m.kedaluwarsa = Some("2027-02-28".into());
    let nov_dekat = stok::masuk(&c, d.path(), &p, m).unwrap();
    let urut: Vec<i64> = stok::kiriman_barang(&c, &p, b, true, HARI_INI).unwrap().iter().map(|k| k.id).collect();
    assert_eq!(urut, vec![sep, okt, nov_dekat, nov_jauh, nov, nov_tanpa]);
}

#[test]
fn tanda_kedaluwarsa() {
    let (c, d, b) = dengan_barang();
    let mut m = masukan_masuk(b, "2026-09-01", 1_000, 1);
    m.kedaluwarsa = Some("2026-11-20".into());
    stok::masuk(&c, d.path(), &pemilik(), m.clone()).unwrap();
    m.tanggal_masuk = "2026-09-02".into();
    m.kedaluwarsa = Some("2026-12-10".into());
    stok::masuk(&c, d.path(), &pemilik(), m.clone()).unwrap();
    m.tanggal_masuk = "2026-09-03".into();
    m.kedaluwarsa = Some(HARI_INI.into());
    stok::masuk(&c, d.path(), &pemilik(), m).unwrap();
    let v = stok::kiriman_barang(&c, &pemilik(), b, true, HARI_INI).unwrap();
    assert!(v[0].sudah_kedaluwarsa);
    assert_eq!(v[0].hari_ke_kedaluwarsa, Some(-4));
    assert!(!v[1].sudah_kedaluwarsa);
    assert_eq!(v[1].hari_ke_kedaluwarsa, Some(16));
    assert!(!v[2].sudah_kedaluwarsa, "masih boleh dijual pada hari kedaluwarsanya");
    assert_eq!(v[2].hari_ke_kedaluwarsa, Some(0));
}

#[test]
fn harga_beli_tersembunyi() {
    let s = skenario();
    let v = stok::kiriman_barang(&s.c, &kasir(), s.barang, true, HARI_INI).unwrap();
    assert_eq!(v.len(), 3);
    assert!(v.iter().all(|k| k.harga_beli.is_none()));
    assert_eq!(v[0].spek.as_deref(), Some("protein 30%"));
    assert!(!serde_json::to_string(&v).unwrap().contains("385000"));
}

#[test]
fn hak_masuk() {
    let s = skenario();
    let m = masukan_masuk(s.barang, "2026-11-21", 1_000, 1);
    assert_eq!(stok::masuk(&s.c, s.d.path(), &kasir(), m.clone()).unwrap_err().kode, "TIDAK_BERHAK");
    assert_eq!(stok::daftar_masuk(&s.c, &kasir(), "2026-01-01", "2026-12-31").unwrap_err().kode, "TIDAK_BERHAK");
    let lihat = staf(&[(Modul::BarangMasuk, Tingkat::Lihat)], true);
    assert_eq!(stok::masuk(&s.c, s.d.path(), &lihat, m.clone()).unwrap_err().kode, "TIDAK_BERHAK");
    assert_eq!(stok::daftar_masuk(&s.c, &lihat, "2026-01-01", "2026-12-31").unwrap().len(), 3);
    assert!(stok::masuk(&s.c, s.d.path(), &staf(&[(Modul::BarangMasuk, Tingkat::Tambah)], true), m).is_ok());
    assert_eq!(stok::kiriman_barang(&s.c, &staf(&[(Modul::Kas, Tingkat::Ubah)], true), s.barang, true, HARI_INI).unwrap_err().kode, "TIDAK_BERHAK");
}

#[test]
fn daftar_masuk_terbaru_dulu() {
    let s = skenario();
    let v = stok::daftar_masuk(&s.c, &pemilik(), "2026-10-01", "2026-12-31").unwrap();
    assert_eq!(v.iter().map(|k| k.id).collect::<Vec<_>>(), vec![s.nov, s.okt]);
    assert_eq!(stok::daftar_masuk(&s.c, &pemilik(), "2026-12-01", "2026-10-01").unwrap_err().kode, "VALIDASI");
}

#[test]
fn validasi_masuk() {
    let (c, d, b) = dengan_barang();
    let p = pemilik();
    let dasar = || masukan_masuk(b, "2026-10-15", 1_000, 1_000);
    let mut kasus = Vec::new();
    let mut m = dasar();
    m.jumlah = 0;
    kasus.push(m);
    let mut m = dasar();
    m.jumlah = -1_000;
    kasus.push(m);
    let mut m = dasar();
    m.harga_beli = -1;
    kasus.push(m);
    let mut m = dasar();
    m.barang_id = 99;
    kasus.push(m);
    let mut m = dasar();
    m.tanggal_masuk = "15-10-2026".into();
    kasus.push(m);
    let mut m = dasar();
    m.kedaluwarsa = Some("2026-10-14".into());
    kasus.push(m);
    let mut m = dasar();
    m.kedaluwarsa = Some("besok".into());
    kasus.push(m);
    let mut m = dasar();
    m.akun_kas_id = None;
    kasus.push(m);
    let mut m = dasar();
    m.akun_kas_id = Some(99);
    kasus.push(m);
    for (i, m) in kasus.into_iter().enumerate() {
        assert_eq!(stok::masuk(&c, d.path(), &p, m).unwrap_err().kode, "VALIDASI", "kasus {i}");
    }
    // barang nonaktif
    let mut nonaktif = uji_barang::masukan("Pakan Apung 781-2");
    nonaktif.kode = "B0001".into();
    nonaktif.aktif = false;
    barang::ubah(&c, d.path(), &p, b, nonaktif).unwrap();
    assert_eq!(stok::masuk(&c, d.path(), &p, dasar()).unwrap_err().kode, "VALIDASI");
    assert_eq!(hitung(&c, "SELECT count(*) FROM kiriman"), 0);
    assert_eq!(hitung(&c, "SELECT count(*) FROM kas"), 0);
}

#[test]
fn kedaluwarsa_kosong_dianggap_tidak_ada() {
    let (c, d, b) = dengan_barang();
    let mut m = masukan_masuk(b, "2026-10-15", 1_000, 1);
    m.kedaluwarsa = Some("  ".into());
    m.spek = Some(" ".into());
    stok::masuk(&c, d.path(), &pemilik(), m).unwrap();
    let k = &stok::kiriman_barang(&c, &pemilik(), b, true, HARI_INI).unwrap()[0];
    assert_eq!(k.kedaluwarsa, None);
    assert_eq!(k.spek, None);
    assert_eq!(k.hari_ke_kedaluwarsa, None);
}

#[test]
fn gagal_foto_tak_menyisakan_data() {
    let (c, d, b) = dengan_barang();
    let palsu = d.path().join("palsu.jpg");
    std::fs::write(&palsu, b"bukan gambar").unwrap();
    let mut m = masukan_masuk(b, "2026-10-15", 1_000, 1_000);
    m.foto_sumber = Some(palsu);
    assert_eq!(stok::masuk(&c, d.path(), &pemilik(), m).unwrap_err().kode, "VALIDASI");
    assert_eq!(hitung(&c, "SELECT count(*) FROM kiriman"), 0);
    assert_eq!(hitung(&c, "SELECT count(*) FROM kas"), 0);
}
