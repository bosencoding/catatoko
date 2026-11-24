use crate::db::{buka, buka_memori, migrasi};
use crate::jalur::siapkan;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

const TABEL: [&str; 13] = [
    "pengguna", "hak_akses", "pengaturan", "kategori_barang", "barang", "akun_kas",
    "kategori_kas", "kiriman", "transaksi_keluar", "baris_keluar", "penyesuaian_stok",
    "kas", "log_aktivitas",
];

fn hitung(c: &rusqlite::Connection, sql: &str) -> i64 {
    c.query_row(sql, [], |r| r.get(0)).unwrap()
}

#[test]
fn migrasi_membuat_semua_tabel() {
    let c = buka_memori();
    for t in TABEL {
        let n: i64 = c
            .query_row("SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?1", [t], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 1, "tabel {t} tidak ada");
    }
    assert_eq!(hitung(&c, "PRAGMA user_version"), 1);
    assert_eq!(hitung(&c, "PRAGMA foreign_keys"), 1);
}

#[test]
fn migrasi_dua_kali_aman() {
    let mut c = buka_memori();
    migrasi(&mut c).unwrap();
    assert_eq!(hitung(&c, "SELECT count(*) FROM akun_kas"), 1);
}

#[test]
fn benih_bawaan() {
    let c = buka_memori();
    let nama: String = c.query_row("SELECT nama FROM akun_kas", [], |r| r.get(0)).unwrap();
    assert_eq!(nama, "Tunai");
    assert_eq!(hitung(&c, "SELECT count(*) FROM kategori_kas"), 7);
    assert_eq!(hitung(&c, "SELECT count(*) FROM kategori_kas WHERE bawaan=1"), 3);
    let menit: String = c
        .query_row("SELECT nilai FROM pengaturan WHERE kunci='kunci_otomatis_menit'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(menit, "10");
}

#[test]
fn log_tidak_bisa_diubah() {
    let c = buka_memori();
    c.execute("INSERT INTO log_aktivitas(waktu,tindakan,ringkasan) VALUES('2026-11-24T10:00:00','x','y')", [])
        .unwrap();
    assert!(c.execute("UPDATE log_aktivitas SET ringkasan='z'", []).is_err());
    assert!(c.execute("DELETE FROM log_aktivitas", []).is_err());
    assert_eq!(hitung(&c, "SELECT count(*) FROM log_aktivitas"), 1);
}

#[test]
fn sisa_tidak_boleh_negatif() {
    let c = buka_memori();
    c.execute("INSERT INTO barang(kode,nama,satuan,dibuat_pada) VALUES('B1','x','kg','t')", []).unwrap();
    c.execute(
        "INSERT INTO kiriman(barang_id,tanggal_masuk,jumlah_awal,sisa,harga_beli,dibuat_pada) VALUES(1,'2026-01-01',1000,1000,1,'t')",
        [],
    )
    .unwrap();
    assert!(c.execute("UPDATE kiriman SET sisa=-1", []).is_err());
}

#[test]
fn buka_file_mode_delete() {
    let d = tempfile::tempdir().unwrap();
    let c = buka(&d.path().join("catat.db")).unwrap();
    let mode: String = c.query_row("PRAGMA journal_mode", [], |r| r.get(0)).unwrap();
    assert_eq!(mode, "delete");
    assert_eq!(hitung(&c, "PRAGMA foreign_keys"), 1);
}

#[test]
fn siapkan_membuat_subfolder() {
    let d = tempfile::tempdir().unwrap();
    let data = d.path().join("data");
    siapkan(&data).unwrap();
    assert!(data.join("foto").is_dir());
    assert!(data.join("cadangan").is_dir());
}

#[cfg(unix)]
#[test]
fn folder_tak_bisa_ditulis() {
    let d = tempfile::tempdir().unwrap();
    std::fs::set_permissions(d.path(), std::fs::Permissions::from_mode(0o555)).unwrap();
    let data = d.path().join("data");
    let e = siapkan(&data).unwrap_err();
    std::fs::set_permissions(d.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(e.kode, "FOLDER_DATA");
    assert!(e.pesan.contains(&data.display().to_string()), "{}", e.pesan);
}

#[cfg(unix)]
#[test]
fn folder_ada_tapi_terkunci() {
    let d = tempfile::tempdir().unwrap();
    let data = d.path().join("data");
    siapkan(&data).unwrap();
    std::fs::set_permissions(&data, std::fs::Permissions::from_mode(0o555)).unwrap();
    let e = siapkan(&data).unwrap_err();
    std::fs::set_permissions(&data, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(e.kode, "FOLDER_DATA");
}

#[test]
fn folder_data_hanya_dari_override_atau_lokasi_program() {
    use crate::jalur::pilih_folder;
    use std::path::{Path, PathBuf};
    assert_eq!(pilih_folder(None, Path::new("/media/usb/Catat/catat")), Some(PathBuf::from("/media/usb/Catat/data")));
    assert_eq!(pilih_folder(Some("/x/dev".into()), Path::new("/media/usb/Catat/catat")), Some(PathBuf::from("/x/dev")));
    assert_eq!(pilih_folder(None, Path::new("")), None);
}

#[test]
fn hanya_satu_jendela_boleh_memegang_data() {
    let d = tempfile::tempdir().unwrap();
    siapkan(d.path()).unwrap();
    let pertama = crate::jalur::kunci_tunggal(d.path()).unwrap();
    let e = crate::jalur::kunci_tunggal(d.path()).unwrap_err();
    assert_eq!(e.kode, "SUDAH_TERBUKA");
    assert!(e.pesan.contains("sudah terbuka"), "{}", e.pesan);
    drop(pertama);
    assert!(crate::jalur::kunci_tunggal(d.path()).is_ok(), "kunci lepas saat jendela pertama ditutup");
}
