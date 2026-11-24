use super::{db_uji, hitung, pemilik, staf};
use crate::akses::{wajib, Modul, Tingkat};
use crate::auth::{ganti_sandi_sendiri, login, perlu_setup, pulihkan_sandi, setup_awal};
use crate::pengguna::{self, hak_bawaan_staf, PenggunaMasukan};
use crate::state::AppState;
use chrono::{DateTime, Duration, Local, TimeZone};
use rusqlite::Connection;
use std::collections::HashMap;
use std::time::Instant;

const SANDI: &str = "rahasia123";

fn t0() -> DateTime<Local> {
    Local.with_ymd_and_hms(2026, 11, 24, 10, 0, 0).unwrap()
}

fn db_setup() -> (Connection, String) {
    let mut c = crate::db::buka_memori();
    let kode = setup_awal(&mut c, "Toko Mina", "budi", "Pak Budi", SANDI).unwrap();
    (c, kode)
}

fn masukan_staf(nama: &str) -> PenggunaMasukan {
    PenggunaMasukan {
        nama_pengguna: nama.into(),
        nama: "Staf".into(),
        peran: "staf".into(),
        lihat_harga_beli: false,
        hak: hak_bawaan_staf(),
        aktif: true,
    }
}

#[test]
fn setup_membuat_pemilik_dan_kode() {
    let mut c = crate::db::buka_memori();
    assert!(perlu_setup(&c).unwrap());
    let kode = setup_awal(&mut c, "Toko Mina", "budi", "Pak Budi", SANDI).unwrap();
    assert!(!perlu_setup(&c).unwrap());
    let bagian: Vec<&str> = kode.split('-').collect();
    assert_eq!(bagian.len(), 4);
    for b in bagian {
        assert_eq!(b.len(), 5);
        assert!(b.chars().all(|ch| "ABCDEFGHJKLMNPQRSTUVWXYZ23456789".contains(ch)), "{kode}");
    }
    let toko: String = c.query_row("SELECT nilai FROM pengaturan WHERE kunci='nama_toko'", [], |r| r.get(0)).unwrap();
    assert_eq!(toko, "Toko Mina");
    assert!(setup_awal(&mut c, "Lain", "ani", "Ani", SANDI).is_err());
    assert_eq!(hitung(&c, "SELECT count(*) FROM pengguna"), 1);
}

#[test]
fn sandi_pendek_ditolak() {
    let mut c = crate::db::buka_memori();
    let e = setup_awal(&mut c, "Toko", "budi", "Budi", "1234567").unwrap_err();
    assert_eq!(e.kode, "VALIDASI");
    assert!(perlu_setup(&c).unwrap());
}

#[test]
fn sandi_tidak_disimpan_polos() {
    let (c, kode) = db_setup();
    let hash: String = c.query_row("SELECT hash_sandi FROM pengguna", [], |r| r.get(0)).unwrap();
    assert!(hash.starts_with("$argon2id$"), "{hash}");
    let hk: String = c.query_row("SELECT nilai FROM pengaturan WHERE kunci='hash_kode_pemulihan'", [], |r| r.get(0)).unwrap();
    assert!(hk.starts_with("$argon2id$"));
    assert!(!hk.contains(&kode));
}

#[test]
fn login_benar_dan_salah() {
    let (c, _) = db_setup();
    let s = login(&c, "budi", SANDI, t0()).unwrap();
    assert!(s.pemilik);
    assert_eq!(s.nama, "Pak Budi");
    assert!(login(&c, "BUDI", SANDI, t0()).is_ok(), "nama pengguna tak peka huruf besar");
    let a = login(&c, "budi", "salahsalah", t0()).unwrap_err();
    let b = login(&c, "tidakada", "salahsalah", t0()).unwrap_err();
    assert_eq!(a.kode, "SANDI_SALAH");
    assert_eq!(b.kode, "SANDI_SALAH");
    assert_eq!(a.pesan, b.pesan);
}

#[test]
fn penahanan_bertingkat() {
    let (c, _) = db_setup();
    for _ in 0..4 {
        assert_eq!(login(&c, "budi", "salahsalah", t0()).unwrap_err().kode, "SANDI_SALAH");
    }
    // gagal ke-5 → ditahan 30 detik
    login(&c, "budi", "salahsalah", t0()).unwrap_err();
    let e = login(&c, "budi", SANDI, t0() + Duration::seconds(29)).unwrap_err();
    assert_eq!(e.kode, "DITAHAN");
    assert!(e.pesan.contains("detik"), "{}", e.pesan);
    // gagal ke-6 setelah lepas → ditahan 60 detik
    let t1 = t0() + Duration::seconds(31);
    login(&c, "budi", "salahsalah", t1).unwrap_err();
    assert_eq!(login(&c, "budi", SANDI, t1 + Duration::seconds(59)).unwrap_err().kode, "DITAHAN");
    let s = login(&c, "budi", SANDI, t1 + Duration::seconds(61)).unwrap();
    assert!(s.pemilik);
    assert_eq!(hitung(&c, "SELECT gagal_login FROM pengguna WHERE nama_pengguna='budi'"), 0);
}

#[test]
fn penahanan_maksimum_15_menit() {
    let (c, _) = db_setup();
    c.execute("UPDATE pengguna SET gagal_login=40", []).unwrap();
    login(&c, "budi", "salahsalah", t0()).unwrap_err();
    assert_eq!(login(&c, "budi", SANDI, t0() + Duration::seconds(899)).unwrap_err().kode, "DITAHAN");
    assert!(login(&c, "budi", SANDI, t0() + Duration::seconds(901)).is_ok());
}

#[test]
fn pengguna_nonaktif_tak_bisa_login() {
    let (c, _) = db_setup();
    let s = login(&c, "budi", SANDI, t0()).unwrap();
    let mut m = masukan_staf("ani");
    let id = pengguna::tambah(&c, &s, m.clone(), SANDI).unwrap();
    assert!(login(&c, "ani", SANDI, t0()).is_ok());
    m.aktif = false;
    pengguna::ubah(&c, &s, id, m).unwrap();
    assert_eq!(login(&c, "ani", SANDI, t0()).unwrap_err().kode, "SANDI_SALAH");
}

#[test]
fn pulihkan_dengan_kode() {
    let (mut c, kode) = db_setup();
    assert_eq!(pulihkan_sandi(&mut c, "AAAAA-AAAAA-AAAAA-AAAAA", "budi", "sandibaru1").unwrap_err().kode, "SANDI_SALAH");
    assert!(login(&c, "budi", SANDI, t0()).is_ok());
    let baru = pulihkan_sandi(&mut c, &kode.to_lowercase().replace('-', " "), "budi", "sandibaru1").unwrap();
    assert_ne!(baru, kode);
    assert!(login(&c, "budi", "sandibaru1", t0()).is_ok());
    assert_eq!(login(&c, "budi", SANDI, t0()).unwrap_err().kode, "SANDI_SALAH");
    assert_eq!(pulihkan_sandi(&mut c, &kode, "budi", "sandibaru2").unwrap_err().kode, "SANDI_SALAH");
    // target bukan pemilik
    let s = login(&c, "budi", "sandibaru1", t0()).unwrap();
    pengguna::tambah(&c, &s, masukan_staf("ani"), SANDI).unwrap();
    assert_eq!(pulihkan_sandi(&mut c, &baru, "ani", "sandibaru3").unwrap_err().kode, "VALIDASI");
    // kode masih berlaku karena pemulihan di atas gagal
    assert!(pulihkan_sandi(&mut c, &baru, "budi", "sandibaru3").is_ok());
}

#[test]
fn pulihkan_melepas_penahanan() {
    let (mut c, kode) = db_setup();
    for _ in 0..6 {
        let _ = login(&c, "budi", "salahsalah", t0());
    }
    pulihkan_sandi(&mut c, &kode, "budi", "sandibaru1").unwrap();
    assert!(login(&c, "budi", "sandibaru1", t0()).is_ok());
}

#[test]
fn ganti_sandi_sendiri_butuh_sandi_lama() {
    let (c, _) = db_setup();
    let s = login(&c, "budi", SANDI, t0()).unwrap();
    assert_eq!(ganti_sandi_sendiri(&c, &s, "salahsalah", "sandibaru1").unwrap_err().kode, "SANDI_SALAH");
    assert_eq!(ganti_sandi_sendiri(&c, &s, SANDI, "pendek").unwrap_err().kode, "VALIDASI");
    ganti_sandi_sendiri(&c, &s, SANDI, "sandibaru1").unwrap();
    assert!(login(&c, "budi", "sandibaru1", t0()).is_ok());
}

#[test]
fn pemilik_boleh_semua() {
    let s = pemilik();
    assert!(s.boleh(Modul::Kas, Tingkat::Ubah));
    assert!(s.boleh_harga_beli());
    assert!(wajib(&s, Modul::Laporan, Tingkat::Ubah).is_ok());
}

#[test]
fn staf_sesuai_hak() {
    let s = staf(&[(Modul::Kas, Tingkat::Lihat)], false);
    assert!(wajib(&s, Modul::Kas, Tingkat::Lihat).is_ok());
    assert_eq!(wajib(&s, Modul::Kas, Tingkat::Tambah).unwrap_err().kode, "TIDAK_BERHAK");
    assert_eq!(wajib(&s, Modul::Barang, Tingkat::Lihat).unwrap_err().kode, "TIDAK_BERHAK");
    assert!(!s.boleh_harga_beli());
    assert_eq!(crate::akses::wajib_pemilik(&s).unwrap_err().kode, "TIDAK_BERHAK");
    assert_eq!(crate::akses::wajib_harga_beli(&s).unwrap_err().kode, "TIDAK_BERHAK");
    let t = staf(&[(Modul::Kas, Tingkat::Ubah)], true);
    assert!(wajib(&t, Modul::Kas, Tingkat::Lihat).is_ok(), "tingkat tinggi mencakup yang rendah");
}

#[test]
fn staf_tak_bisa_kelola_pengguna() {
    let c = db_uji();
    let s = staf(&[(Modul::Kas, Tingkat::Ubah)], true);
    assert_eq!(pengguna::daftar(&c, &s).unwrap_err().kode, "TIDAK_BERHAK");
    assert_eq!(pengguna::tambah(&c, &s, masukan_staf("x"), SANDI).unwrap_err().kode, "TIDAK_BERHAK");
    assert_eq!(pengguna::atur_ulang_sandi(&c, &s, 1, "sandibaru1").unwrap_err().kode, "TIDAK_BERHAK");
    assert_eq!(crate::log::daftar(&c, &s, "2000-01-01", "2100-01-01").unwrap_err().kode, "TIDAK_BERHAK");
}

#[test]
fn staf_baru_bawaan_kasir() {
    let (c, _) = db_setup();
    let s = login(&c, "budi", SANDI, t0()).unwrap();
    pengguna::tambah(&c, &s, masukan_staf("ani"), SANDI).unwrap();
    let a = login(&c, "ani", SANDI, t0()).unwrap();
    assert!(!a.pemilik);
    assert!(!a.boleh_harga_beli());
    assert!(a.boleh(Modul::Barang, Tingkat::Lihat) && !a.boleh(Modul::Barang, Tingkat::Tambah));
    assert!(a.boleh(Modul::BarangKeluar, Tingkat::Tambah) && !a.boleh(Modul::BarangKeluar, Tingkat::Ubah));
    for m in [Modul::BarangMasuk, Modul::Kas, Modul::Laporan] {
        assert!(!a.boleh(m, Tingkat::Lihat));
    }
}

#[test]
fn barang_masuk_dan_laporan_butuh_harga_beli() {
    let c = db_uji();
    let s = pemilik();
    for modul in [Modul::BarangMasuk, Modul::Laporan] {
        let mut m = masukan_staf("ani");
        m.hak.insert(modul, Tingkat::Lihat);
        assert_eq!(pengguna::tambah(&c, &s, m.clone(), SANDI).unwrap_err().kode, "VALIDASI");
        assert_eq!(pengguna::ubah(&c, &s, 2, m.clone()).unwrap_err().kode, "VALIDASI");
        m.lihat_harga_beli = true;
        m.nama_pengguna = "staf".into();
        assert!(pengguna::ubah(&c, &s, 2, m).is_ok());
    }
}

#[test]
fn pemilik_terakhir_dilindungi() {
    let c = db_uji();
    let s = pemilik();
    let mut m = PenggunaMasukan {
        nama_pengguna: "pemilik".into(),
        nama: "Pemilik".into(),
        peran: "pemilik".into(),
        lihat_harga_beli: true,
        hak: HashMap::new(),
        aktif: false,
    };
    assert_eq!(pengguna::ubah(&c, &s, 1, m.clone()).unwrap_err().kode, "VALIDASI");
    m.aktif = true;
    m.peran = "staf".into();
    assert_eq!(pengguna::ubah(&c, &s, 1, m.clone()).unwrap_err().kode, "VALIDASI");
    // dengan pemilik kedua, boleh
    let mut kedua = masukan_staf("staf");
    kedua.peran = "pemilik".into();
    pengguna::ubah(&c, &s, 2, kedua).unwrap();
    assert!(pengguna::ubah(&c, &s, 1, m).is_ok());
}

#[test]
fn nama_pengguna_ganda_ditolak() {
    let c = db_uji();
    let e = pengguna::tambah(&c, &pemilik(), masukan_staf("STAF"), SANDI).unwrap_err();
    assert_eq!(e.kode, "VALIDASI");
    assert_eq!(pengguna::tambah(&c, &pemilik(), masukan_staf("  "), SANDI).unwrap_err().kode, "VALIDASI");
    let mut m = masukan_staf("x");
    m.peran = "admin".into();
    assert_eq!(pengguna::tambah(&c, &pemilik(), m, SANDI).unwrap_err().kode, "VALIDASI");
}

#[test]
fn daftar_pengguna_tanpa_hash() {
    let c = db_uji();
    let d = pengguna::daftar(&c, &pemilik()).unwrap();
    assert_eq!(d.len(), 2);
    let j = serde_json::to_string(&d).unwrap();
    assert!(!j.contains("hash"), "{j}");
}

#[test]
fn aksi_tercatat_di_log() {
    let (c, _) = db_setup();
    let s = login(&c, "budi", SANDI, t0()).unwrap();
    let _ = login(&c, "budi", "salahsalah", t0());
    pengguna::tambah(&c, &s, masukan_staf("ani"), SANDI).unwrap();
    let ada = |t: &str| hitung(&c, &format!("SELECT count(*) FROM log_aktivitas WHERE tindakan='{t}'"));
    assert_eq!(ada("auth.setup"), 1);
    assert_eq!(ada("auth.login"), 1);
    assert_eq!(ada("auth.gagal"), 1);
    assert_eq!(ada("pengguna.tambah"), 1);
    let log = crate::log::daftar(&c, &s, "2000-01-01", "2100-01-01").unwrap();
    assert_eq!(log.len(), 4);
    assert!(!serde_json::to_string(&log).unwrap().contains(SANDI));
}

fn state_dengan(c: Connection, menit_lalu: u64) -> AppState {
    let st = AppState::baru(c, std::env::temp_dir());
    *st.sesi.lock().unwrap() = Some(pemilik());
    *st.terakhir_aktif.lock().unwrap() = Instant::now().checked_sub(std::time::Duration::from_secs(menit_lalu * 60)).unwrap();
    st
}

#[test]
fn sesi_menganggur_terkunci() {
    let st = state_dengan(db_uji(), 11);
    let e = crate::perintah::dengan_sesi(&st, |_, _| Ok(1)).unwrap_err();
    assert_eq!(e.kode, "TERKUNCI");
    assert!(st.sesi.lock().unwrap().is_none());
    assert_eq!(crate::perintah::dengan_sesi(&st, |_, _| Ok(1)).unwrap_err().kode, "BELUM_LOGIN");
}

#[test]
fn sesi_aktif_diperpanjang() {
    let st = state_dengan(db_uji(), 9);
    assert_eq!(crate::perintah::dengan_sesi(&st, |_, s| Ok(s.pengguna_id)).unwrap(), 1);
    assert!(st.terakhir_aktif.lock().unwrap().elapsed().as_secs() < 5);
}

#[test]
fn batas_kunci_mengikuti_pengaturan() {
    let c = db_uji();
    crate::pengaturan::simpan(&c, &pemilik(), "kunci_otomatis_menit", "20").unwrap();
    let st = state_dengan(c, 11);
    assert!(crate::perintah::dengan_sesi(&st, |_, _| Ok(1)).is_ok());
}

#[test]
fn pengaturan_dijaga() {
    let c = db_uji();
    let s = staf(&[], false);
    assert_eq!(crate::pengaturan::simpan(&c, &s, "nama_toko", "X").unwrap_err().kode, "TIDAK_BERHAK");
    let p = pemilik();
    assert_eq!(crate::pengaturan::simpan(&c, &p, "kunci_otomatis_menit", "0").unwrap_err().kode, "VALIDASI");
    assert_eq!(crate::pengaturan::simpan(&c, &p, "kunci_otomatis_menit", "121").unwrap_err().kode, "VALIDASI");
    assert_eq!(crate::pengaturan::simpan(&c, &p, "hash_kode_pemulihan", "x").unwrap_err().kode, "VALIDASI");
    assert_eq!(crate::pengaturan::simpan(&c, &p, "nama_toko", " ").unwrap_err().kode, "VALIDASI");
    crate::pengaturan::simpan(&c, &p, "nama_toko", "Toko Baru").unwrap();
    assert_eq!(crate::pengaturan::ambil(&c).unwrap().nama_toko, "Toko Baru");
}
