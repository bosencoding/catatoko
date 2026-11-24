use super::{barang as uji_barang, hitung, jual, jual_tiga, kasir, masukan_masuk, pemilik, sisa, skenario, staf, Skenario, HARI_INI};
use crate::akses::{Modul, Tingkat};
use crate::stok::{self, Alokasi};
use crate::{barang, kas};

fn pasangan(v: &[Alokasi]) -> Vec<(i64, i64)> {
    v.iter().map(|a| (a.kiriman_id, a.jumlah)).collect()
}

fn potret(s: &Skenario) -> (i64, i64, i64, i64, i64, i64) {
    (
        sisa(&s.c, s.sep),
        sisa(&s.c, s.okt),
        sisa(&s.c, s.nov),
        hitung(&s.c, "SELECT count(*) FROM transaksi_keluar"),
        hitung(&s.c, "SELECT count(*) FROM baris_keluar"),
        hitung(&s.c, "SELECT count(*) FROM kas"),
    )
}

#[test]
fn usul_ambil_terlama() {
    let s = skenario();
    let u = stok::usul_alokasi(&s.c, &pemilik(), s.barang, 1_000, HARI_INI).unwrap();
    assert_eq!(pasangan(&u), vec![(s.sep, 1_000)]);
}

#[test]
fn usul_pecah_antar_kiriman() {
    let s = skenario();
    let u = stok::usul_alokasi(&s.c, &pemilik(), s.barang, 15_000, HARI_INI).unwrap();
    assert_eq!(pasangan(&u), vec![(s.sep, 2_000), (s.okt, 10_000), (s.nov, 3_000)]);
    let semua = stok::usul_alokasi(&s.c, &pemilik(), s.barang, 32_000, HARI_INI).unwrap();
    assert_eq!(semua.iter().map(|a| a.jumlah).sum::<i64>(), 32_000);
}

#[test]
fn usul_lewati_kedaluwarsa() {
    let s = skenario();
    s.c.execute("UPDATE kiriman SET kedaluwarsa='2026-11-23' WHERE id=?1", [s.sep]).unwrap();
    let u = stok::usul_alokasi(&s.c, &pemilik(), s.barang, 1_000, HARI_INI).unwrap();
    assert_eq!(pasangan(&u), vec![(s.okt, 1_000)]);
    let e = stok::usul_alokasi(&s.c, &pemilik(), s.barang, 31_000, HARI_INI).unwrap_err();
    assert_eq!(e.kode, "STOK_KURANG");
    assert!(e.pesan.contains("30") && e.pesan.contains("kedaluwarsa"), "{}", e.pesan);
}

#[test]
fn usul_tanggal_masuk_mengalahkan_kedaluwarsa() {
    let s = skenario();
    s.c.execute("UPDATE kiriman SET kedaluwarsa='2027-05-31' WHERE id=?1", [s.sep]).unwrap();
    s.c.execute("UPDATE kiriman SET kedaluwarsa='2027-01-31' WHERE id=?1", [s.okt]).unwrap();
    let u = stok::usul_alokasi(&s.c, &pemilik(), s.barang, 3_000, HARI_INI).unwrap();
    assert_eq!(pasangan(&u), vec![(s.sep, 2_000), (s.okt, 1_000)]);
}

#[test]
fn usul_kurang() {
    let s = skenario();
    let e = stok::usul_alokasi(&s.c, &pemilik(), s.barang, 33_000, HARI_INI).unwrap_err();
    assert_eq!(e.kode, "STOK_KURANG");
    assert!(e.pesan.contains("Pakan Apung 781-2") && e.pesan.contains("32"), "{}", e.pesan);
    assert_eq!(stok::usul_alokasi(&s.c, &pemilik(), s.barang, 0, HARI_INI).unwrap_err().kode, "VALIDASI");
    assert_eq!(stok::usul_alokasi(&s.c, &pemilik(), 99, 1_000, HARI_INI).unwrap_err().kode, "VALIDASI");
}

#[test]
fn usul_melewati_kiriman_batal_dan_habis() {
    let s = skenario();
    s.c.execute("UPDATE kiriman SET batal=1 WHERE id=?1", [s.sep]).unwrap();
    s.c.execute("UPDATE kiriman SET sisa=0 WHERE id=?1", [s.okt]).unwrap();
    let u = stok::usul_alokasi(&s.c, &pemilik(), s.barang, 1_000, HARI_INI).unwrap();
    assert_eq!(pasangan(&u), vec![(s.nov, 1_000)]);
}

#[test]
fn keluar_mengurangi_sisa_dan_membuat_kas() {
    let s = skenario();
    let id = jual_tiga(&s);
    assert_eq!((sisa(&s.c, s.sep), sisa(&s.c, s.okt), sisa(&s.c, s.nov)), (0, 9_000, 20_000));
    let (arah, jumlah, kategori, ref_id): (String, i64, String, i64) = s
        .c
        .query_row(
            "SELECT k.arah, k.jumlah, g.nama, k.ref_id FROM kas k JOIN kategori_kas g ON g.id=k.kategori_id WHERE k.ref_jenis='transaksi_keluar'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .unwrap();
    assert_eq!((arah.as_str(), jumlah, kategori.as_str(), ref_id), ("masuk", 1_350_000, kas::KAT_PENJUALAN, id));
    let t = &stok::daftar_keluar(&s.c, &pemilik(), HARI_INI, HARI_INI).unwrap()[0];
    assert_eq!((t.id, t.total, t.laba, t.batal), (id, 1_350_000, Some(188_000), false));
    assert_eq!(t.baris.len(), 2);
    assert_eq!((t.baris[0].kiriman_id, t.baris[0].jumlah, t.baris[0].subtotal, t.baris[0].harga_beli), (s.sep, 2_000, 900_000, Some(385_000)));
    assert_eq!(t.baris[0].barang_nama, "Pakan Apung 781-2");
    assert_eq!(t.dibuat_oleh_nama.as_deref(), Some("Pemilik Uji"));
    assert_eq!(barang::ambil(&s.c, &pemilik(), s.barang).unwrap().stok, 29_000);
    assert_eq!(hitung(&s.c, "SELECT count(*) FROM log_aktivitas WHERE tindakan='keluar.tambah'"), 1);
    // kiriman yang sudah keluar ditandai
    let k = stok::kiriman_barang(&s.c, &pemilik(), s.barang, false, HARI_INI).unwrap();
    assert!(k[0].pernah_keluar && k[1].pernah_keluar && !k[2].pernah_keluar);
}

#[test]
fn keluar_pilih_manual() {
    let s = skenario();
    stok::keluar(&s.c, &pemilik(), jual(s.barang, 450_000, &[(s.nov, 1_500)]), HARI_INI).unwrap();
    assert_eq!((sisa(&s.c, s.sep), sisa(&s.c, s.nov)), (2_000, 18_500));
    assert_eq!(hitung(&s.c, "SELECT jumlah FROM kas"), 675_000);
}

#[test]
fn keluar_beberapa_barang() {
    let s = skenario();
    let jaring = barang::tambah(&s.c, s.d.path(), &pemilik(), uji_barang::masukan("Jaring")).unwrap();
    let kj = stok::masuk(&s.c, s.d.path(), &pemilik(), super::masukan_masuk(jaring, "2026-11-01", 5_000, 100_000)).unwrap();
    let mut m = jual(s.barang, 450_000, &[(s.sep, 1_000)]);
    m.baris.push(stok::BarisKeluarMasukan { barang_id: jaring, harga_jual: 150_000, alokasi: vec![Alokasi { kiriman_id: kj, jumlah: 2_000 }] });
    stok::keluar(&s.c, &pemilik(), m, HARI_INI).unwrap();
    let t = &stok::daftar_keluar(&s.c, &pemilik(), HARI_INI, HARI_INI).unwrap()[0];
    assert_eq!(t.total, 450_000 + 300_000);
    assert_eq!(t.laba, Some((450_000 - 385_000) + (300_000 - 200_000)));
}

#[test]
fn keluar_atomik() {
    let s = skenario();
    let sebelum = potret(&s);
    let mut m = jual(s.barang, 450_000, &[(s.sep, 2_000)]);
    m.baris.push(stok::BarisKeluarMasukan { barang_id: s.barang, harga_jual: 450_000, alokasi: vec![Alokasi { kiriman_id: s.okt, jumlah: 10_001 }] });
    let e = stok::keluar(&s.c, &pemilik(), m, HARI_INI).unwrap_err();
    assert_eq!(e.kode, "STOK_KURANG");
    assert!(e.pesan.contains("Pakan Apung 781-2") && e.pesan.contains("10"), "{}", e.pesan);
    assert_eq!(potret(&s), sebelum);
}

#[test]
fn kiriman_sama_dua_kali_dijumlahkan() {
    let s = skenario();
    let sebelum = potret(&s);
    let e = stok::keluar(&s.c, &pemilik(), jual(s.barang, 1, &[(s.sep, 1_500), (s.sep, 1_000)]), HARI_INI).unwrap_err();
    assert_eq!(e.kode, "STOK_KURANG");
    assert_eq!(potret(&s), sebelum);
}

#[test]
fn keluar_kedaluwarsa_perlu_izin() {
    let s = skenario();
    s.c.execute("UPDATE kiriman SET kedaluwarsa='2026-11-23' WHERE id=?1", [s.sep]).unwrap();
    let sebelum = potret(&s);
    let mut m = jual(s.barang, 100_000, &[(s.sep, 1_000)]);
    let e = stok::keluar(&s.c, &pemilik(), m.clone(), HARI_INI).unwrap_err();
    assert_eq!(e.kode, "VALIDASI");
    assert!(e.pesan.contains("kedaluwarsa"), "{}", e.pesan);
    assert_eq!(potret(&s), sebelum);
    m.izinkan_kedaluwarsa = true;
    stok::keluar(&s.c, &pemilik(), m, HARI_INI).unwrap();
    assert_eq!(sisa(&s.c, s.sep), 1_000);
}

#[test]
fn keluar_tak_sah_ditolak() {
    let s = skenario();
    let p = pemilik();
    let sebelum = potret(&s);
    let jaring = barang::tambah(&s.c, s.d.path(), &p, uji_barang::masukan("Jaring")).unwrap();
    let mut kasus = vec![
        jual(jaring, 1, &[(s.sep, 1_000)]), // kiriman milik barang lain
        jual(s.barang, 1, &[]),            // tanpa alokasi
        jual(s.barang, 1, &[(s.sep, 0)]),
        jual(s.barang, 1, &[(s.sep, -1_000)]),
        jual(s.barang, -1, &[(s.sep, 1_000)]),
        jual(s.barang, 1, &[(999, 1_000)]),
        jual(99, 1, &[(s.sep, 1_000)]),
    ];
    let mut m = jual(s.barang, 1, &[(s.sep, 1_000)]);
    m.baris.clear();
    kasus.push(m);
    let mut m = jual(s.barang, 1, &[(s.sep, 1_000)]);
    m.akun_kas_id = 99;
    kasus.push(m);
    let mut m = jual(s.barang, 1, &[(s.sep, 1_000)]);
    m.tanggal = "kemarin".into();
    kasus.push(m);
    let mut m = jual(s.barang, 1, &[(s.nov, 1_000)]);
    m.tanggal = "2026-11-19".into(); // sebelum kiriman Nov masuk
    kasus.push(m);
    for (i, m) in kasus.into_iter().enumerate() {
        assert_eq!(stok::keluar(&s.c, &p, m, HARI_INI).unwrap_err().kode, "VALIDASI", "kasus {i}");
    }
    s.c.execute("UPDATE kiriman SET batal=1 WHERE id=?1", [s.okt]).unwrap();
    assert_eq!(stok::keluar(&s.c, &p, jual(s.barang, 1, &[(s.okt, 1_000)]), HARI_INI).unwrap_err().kode, "VALIDASI");
    assert_eq!(potret(&s), sebelum);
}

#[test]
fn harga_nol_tanpa_kas() {
    let s = skenario();
    stok::keluar(&s.c, &pemilik(), jual(s.barang, 0, &[(s.sep, 1_000)]), HARI_INI).unwrap();
    assert_eq!(hitung(&s.c, "SELECT count(*) FROM kas"), 0);
    assert_eq!(sisa(&s.c, s.sep), 1_000);
}

#[test]
fn kasir_bisa_jual_tanpa_hak_kas() {
    let s = skenario();
    let k = kasir();
    let u = stok::usul_alokasi(&s.c, &k, s.barang, 3_000, HARI_INI).unwrap();
    stok::keluar(&s.c, &k, jual(s.barang, 450_000, &pasangan(&u)), HARI_INI).unwrap();
    assert_eq!(hitung(&s.c, "SELECT jumlah FROM kas WHERE sumber='otomatis'"), 1_350_000);
    assert_eq!(hitung(&s.c, "SELECT dibuat_oleh FROM kas"), 2);
    let t = &stok::daftar_keluar(&s.c, &k, HARI_INI, HARI_INI).unwrap()[0];
    assert_eq!(t.laba, None);
    assert!(t.baris.iter().all(|b| b.harga_beli.is_none()));
    assert_eq!(t.total, 1_350_000);
}

#[test]
fn hak_keluar() {
    let s = skenario();
    let lihat = staf(&[(Modul::BarangKeluar, Tingkat::Lihat)], false);
    assert_eq!(stok::keluar(&s.c, &lihat, jual(s.barang, 1, &[(s.sep, 1_000)]), HARI_INI).unwrap_err().kode, "TIDAK_BERHAK");
    assert!(stok::daftar_keluar(&s.c, &lihat, HARI_INI, HARI_INI).is_ok());
    let tanpa = staf(&[(Modul::Barang, Tingkat::Ubah), (Modul::BarangMasuk, Tingkat::Ubah)], true);
    assert_eq!(stok::keluar(&s.c, &tanpa, jual(s.barang, 1, &[(s.sep, 1_000)]), HARI_INI).unwrap_err().kode, "TIDAK_BERHAK");
    assert_eq!(stok::daftar_keluar(&s.c, &tanpa, HARI_INI, HARI_INI).unwrap_err().kode, "TIDAK_BERHAK");
    assert_eq!(stok::usul_alokasi(&s.c, &tanpa, s.barang, 1_000, HARI_INI).unwrap_err().kode, "TIDAK_BERHAK");
    let _ = masukan_masuk;
}
