use super::{barang as uji_barang, folder_data, hitung, kasir, pemilik};
use crate::{barang, cadangan, db};
use rusqlite::Connection;
use std::io::Write;
use std::path::{Path, PathBuf};

/// DB berbasis file di folder data sementara, dengan pemilik (id 1) dan satu barang berfoto.
fn toko() -> (Connection, tempfile::TempDir) {
    let d = folder_data();
    let c = db::buka(&d.path().join("catat.db")).unwrap();
    c.execute_batch(
        "INSERT INTO pengguna(id,nama_pengguna,nama,hash_sandi,peran,lihat_harga_beli,dibuat_pada) VALUES(1,'pemilik','Pemilik Uji','x','pemilik',1,'t');
         INSERT INTO pengguna(id,nama_pengguna,nama,hash_sandi,peran,dibuat_pada) VALUES(2,'staf','Staf Uji','x','staf','t');",
    )
    .unwrap();
    let sumber = d.path().join("sumber.png");
    image::RgbImage::from_pixel(64, 64, image::Rgb([1, 2, 3])).save(&sumber).unwrap();
    let mut m = uji_barang::masukan("Barang A");
    m.foto_sumber = Some(sumber);
    barang::tambah(&c, d.path(), &pemilik(), m).unwrap();
    (c, d)
}

fn nama_barang(c: &Connection) -> Vec<String> {
    let mut q = c.prepare("SELECT nama FROM barang ORDER BY id").unwrap();
    let v = q.query_map([], |r| r.get(0)).unwrap().map(|x| x.unwrap()).collect();
    v
}

fn zip_di(d: &Path) -> usize {
    std::fs::read_dir(d.join("cadangan")).unwrap().filter(|e| e.as_ref().unwrap().file_name().to_string_lossy().ends_with(".zip")).count()
}

fn buat_zip(path: &Path, isi: &[(&str, &[u8])]) {
    let mut z = zip::ZipWriter::new(std::fs::File::create(path).unwrap());
    for (nama, data) in isi {
        z.start_file(*nama, zip::write::SimpleFileOptions::default()).unwrap();
        z.write_all(data).unwrap();
    }
    z.finish().unwrap();
}

fn potret(d: &Path) -> (Vec<u8>, Vec<String>) {
    let mut foto: Vec<String> = std::fs::read_dir(d.join("foto")).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().into_owned()).collect();
    foto.sort();
    (std::fs::read(d.join("catat.db")).unwrap(), foto)
}

#[test]
fn cadangkan_lalu_pulihkan() {
    let (mut c, d) = toko();
    let foto_a = barang::daftar(&c, &pemilik(), None, false).unwrap()[0].foto.clone().unwrap();
    let zip = cadangan::cadangkan(&c, d.path(), &pemilik(), None).unwrap();
    assert!(zip.starts_with(d.path().join("cadangan")));
    assert!(zip.file_name().unwrap().to_string_lossy().starts_with("catat-"));
    barang::tambah(&c, d.path(), &pemilik(), uji_barang::masukan("Barang B")).unwrap();
    std::fs::remove_file(d.path().join(&foto_a)).unwrap();
    assert_eq!(nama_barang(&c), vec!["Barang A", "Barang B"]);
    cadangan::pulihkan(&mut c, d.path(), &pemilik(), &zip).unwrap();
    assert_eq!(nama_barang(&c), vec!["Barang A"]);
    assert!(d.path().join(&foto_a).exists(), "foto ikut pulih");
    // koneksi baru tetap berfungsi dan berpengaturan benar
    assert_eq!(hitung(&c, "PRAGMA foreign_keys"), 1);
    barang::tambah(&c, d.path(), &pemilik(), uji_barang::masukan("Barang C")).unwrap();
    assert_eq!(hitung(&c, "SELECT count(*) FROM log_aktivitas WHERE tindakan='cadangan.pulihkan'"), 1);
    // tidak ada sisa folder sementara
    let sisa: Vec<String> = std::fs::read_dir(d.path()).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().into_owned()).filter(|n| n.starts_with('.') || n.ends_with(".lama")).collect();
    assert!(sisa.is_empty(), "{sisa:?}");
}

#[test]
fn cadangkan_ke_lokasi_pilihan() {
    let (c, d) = toko();
    let luar = tempfile::tempdir().unwrap();
    let tujuan = luar.path().join("punyaku.zip");
    assert_eq!(cadangan::cadangkan(&c, d.path(), &pemilik(), Some(&tujuan)).unwrap(), tujuan);
    let mut arsip = zip::ZipArchive::new(std::fs::File::open(&tujuan).unwrap()).unwrap();
    let nama: Vec<String> = (0..arsip.len()).map(|i| arsip.by_index(i).unwrap().name().to_string()).collect();
    assert!(nama.contains(&"manifest.json".to_string()) && nama.contains(&"catat.db".to_string()));
    assert_eq!(nama.iter().filter(|n| n.starts_with("foto/")).count(), 2);
    assert_eq!(hitung(&c, "SELECT count(*) FROM log_aktivitas WHERE tindakan='cadangan.buat'"), 1);
    assert_eq!(cadangan::cadangkan(&c, d.path(), &pemilik(), Some(&luar.path().join("tidak/ada/x.zip"))).unwrap_err().kode, "VALIDASI");
}

#[test]
fn pulihkan_membuat_cadangan_otomatis() {
    let (mut c, d) = toko();
    let zip = cadangan::cadangkan(&c, d.path(), &pemilik(), None).unwrap();
    assert_eq!(zip_di(d.path()), 1);
    cadangan::pulihkan(&mut c, d.path(), &pemilik(), &zip).unwrap();
    assert_eq!(zip_di(d.path()), 2);
}

#[test]
fn zip_rusak_ditolak() {
    let (mut c, d) = toko();
    let sebelum = potret(d.path());
    let rusak = d.path().join("rusak.zip");
    std::fs::write(&rusak, b"ini bukan zip sama sekali, hanya teks acak").unwrap();
    assert_eq!(cadangan::pulihkan(&mut c, d.path(), &pemilik(), &rusak).unwrap_err().kode, "VALIDASI");
    assert_eq!(cadangan::pulihkan(&mut c, d.path(), &pemilik(), &d.path().join("tidak-ada.zip")).unwrap_err().kode, "VALIDASI");
    assert_eq!(potret(d.path()), sebelum);
    assert_eq!(nama_barang(&c), vec!["Barang A"]);
    assert_eq!(zip_di(d.path()), 0, "tidak ada cadangan otomatis untuk file yang ditolak");
}

#[test]
fn bukan_cadangan_catat_ditolak() {
    let (mut c, d) = toko();
    let sebelum = potret(d.path());
    let f = d.path().join("lain.zip");
    buat_zip(&f, &[("catatan.txt", b"halo")]);
    assert_eq!(cadangan::pulihkan(&mut c, d.path(), &pemilik(), &f).unwrap_err().kode, "VALIDASI");
    buat_zip(&f, &[("manifest.json", br#"{"aplikasi":"lain","versi_skema":1,"dibuat":"x"}"#), ("catat.db", b"x")]);
    assert_eq!(cadangan::pulihkan(&mut c, d.path(), &pemilik(), &f).unwrap_err().kode, "VALIDASI");
    buat_zip(&f, &[("manifest.json", br#"{"aplikasi":"catat","versi_skema":1,"dibuat":"x"}"#)]);
    assert_eq!(cadangan::pulihkan(&mut c, d.path(), &pemilik(), &f).unwrap_err().kode, "VALIDASI", "tanpa catat.db");
    buat_zip(&f, &[("manifest.json", br#"{"aplikasi":"catat","versi_skema":1,"dibuat":"x"}"#), ("catat.db", b"bukan sqlite")]);
    assert_eq!(cadangan::pulihkan(&mut c, d.path(), &pemilik(), &f).unwrap_err().kode, "VALIDASI", "db rusak");
    assert_eq!(potret(d.path()), sebelum);
    assert_eq!(nama_barang(&c), vec!["Barang A"]);
}

fn db_sah(dir: &Path, dengan_pemilik: bool) -> PathBuf {
    let p = dir.join("sumber.db");
    let _ = std::fs::remove_file(&p);
    let c = db::buka(&p).unwrap();
    if dengan_pemilik {
        c.execute("INSERT INTO pengguna(nama_pengguna,nama,hash_sandi,peran,dibuat_pada) VALUES('p','P','x','pemilik','t')", []).unwrap();
    }
    drop(c);
    p
}

#[test]
fn jalur_keluar_folder_ditolak() {
    let (mut c, d) = toko();
    let luar = tempfile::tempdir().unwrap();
    let dbs = std::fs::read(db_sah(luar.path(), true)).unwrap();
    let manifest = format!(r#"{{"aplikasi":"catat","versi_skema":{},"dibuat":"x"}}"#, db::versi_skema_terbaru());
    let sebelum = potret(d.path());
    for jahat in ["../x.jpg", "foto/../../x.jpg", "/tmp/x.jpg", "foto/sub/x.jpg"] {
        let f = luar.path().join("jahat.zip");
        buat_zip(&f, &[("manifest.json", manifest.as_bytes()), ("catat.db", &dbs), (jahat, b"x")]);
        assert_eq!(cadangan::pulihkan(&mut c, d.path(), &pemilik(), &f).unwrap_err().kode, "VALIDASI", "{jahat}");
    }
    assert_eq!(potret(d.path()), sebelum);
    assert!(!d.path().parent().unwrap().join("x.jpg").exists());
}

#[test]
fn skema_lebih_baru_ditolak() {
    let (mut c, d) = toko();
    let luar = tempfile::tempdir().unwrap();
    let dbs = std::fs::read(db_sah(luar.path(), true)).unwrap();
    let f = luar.path().join("baru.zip");
    let manifest = format!(r#"{{"aplikasi":"catat","versi_skema":{},"dibuat":"x"}}"#, db::versi_skema_terbaru() + 1);
    buat_zip(&f, &[("manifest.json", manifest.as_bytes()), ("catat.db", &dbs)]);
    let e = cadangan::pulihkan(&mut c, d.path(), &pemilik(), &f).unwrap_err();
    assert_eq!(e.kode, "VALIDASI");
    assert!(e.pesan.contains("versi"), "{}", e.pesan);
    assert_eq!(nama_barang(&c), vec!["Barang A"]);
}

#[test]
fn cadangan_tanpa_pemilik_ditolak() {
    let (mut c, d) = toko();
    let luar = tempfile::tempdir().unwrap();
    let dbs = std::fs::read(db_sah(luar.path(), false)).unwrap();
    let f = luar.path().join("kosong.zip");
    let manifest = format!(r#"{{"aplikasi":"catat","versi_skema":{},"dibuat":"x"}}"#, db::versi_skema_terbaru());
    buat_zip(&f, &[("manifest.json", manifest.as_bytes()), ("catat.db", &dbs)]);
    assert_eq!(cadangan::pulihkan(&mut c, d.path(), &pemilik(), &f).unwrap_err().kode, "VALIDASI");
    assert_eq!(nama_barang(&c), vec!["Barang A"]);
}

#[test]
fn staf_tak_bisa() {
    let (mut c, d) = toko();
    assert_eq!(cadangan::cadangkan(&c, d.path(), &kasir(), None).unwrap_err().kode, "TIDAK_BERHAK");
    let zip = cadangan::cadangkan(&c, d.path(), &pemilik(), None).unwrap();
    assert_eq!(cadangan::pulihkan(&mut c, d.path(), &kasir(), &zip).unwrap_err().kode, "TIDAK_BERHAK");
}

#[test]
fn cadangan_tidak_boleh_menimpa_data() {
    let (c, d) = toko();
    let sebelum = potret(d.path());
    for jahat in ["catat.db", "foto/x.zip", "apa.zip"] {
        let e = cadangan::cadangkan(&c, d.path(), &pemilik(), Some(&d.path().join(jahat))).unwrap_err();
        assert_eq!(e.kode, "VALIDASI", "{jahat}");
    }
    assert_eq!(potret(d.path()), sebelum);
    assert!(cadangan::cadangkan(&c, d.path(), &pemilik(), Some(&d.path().join("cadangan/manual.zip"))).is_ok());
    assert!(crate::jalur::di_dalam(d.path(), &d.path().join("foto/../catat.db")));
    assert!(!crate::jalur::di_dalam(d.path(), &d.path().parent().unwrap().join("laporan.pdf")));
}

#[test]
fn tukar_gagal_mengembalikan_data_lama() {
    let (mut c, d) = toko();
    // Folder sementara tanpa foto/: langkah terakhir penukaran gagal.
    let sementara = d.path().join(".pulih-uji");
    std::fs::create_dir_all(&sementara).unwrap();
    let luar = tempfile::tempdir().unwrap();
    std::fs::copy(db_sah(luar.path(), true), sementara.join("catat.db")).unwrap();
    assert!(cadangan::tukar(&mut c, d.path(), &sementara).is_err());
    assert_eq!(nama_barang(&c), vec!["Barang A"], "koneksi kembali ke data lama");
    assert!(d.path().join("foto").is_dir() && !d.path().join("foto.lama").exists() && !d.path().join("catat.db.lama").exists());
    barang::tambah(&c, d.path(), &pemilik(), uji_barang::masukan("Barang D")).unwrap();
}

#[test]
fn sisa_pemulihan_yang_terputus_dibereskan_saat_mulai() {
    let (c, d) = toko();
    drop(c);
    let p = d.path();
    // Keadaan setelah mati listrik di tengah penukaran: data lama sudah diganti nama, yang baru setengah jadi.
    std::fs::rename(p.join("catat.db"), p.join("catat.db.lama")).unwrap();
    std::fs::rename(p.join("foto"), p.join("foto.lama")).unwrap();
    std::fs::write(p.join("catat.db"), b"setengah jadi").unwrap();
    std::fs::create_dir_all(p.join(".pulih-abc/foto")).unwrap();
    std::fs::write(p.join("cadangan/.tmp-1.db"), b"x").unwrap();
    cadangan::bereskan_sisa(p);
    let c = db::buka(&p.join("catat.db")).unwrap();
    assert_eq!(nama_barang(&c), vec!["Barang A"]);
    assert_eq!(std::fs::read_dir(p.join("foto")).unwrap().count(), 2);
    for sisa in ["catat.db.lama", "foto.lama", ".pulih-abc", "cadangan/.tmp-1.db"] {
        assert!(!p.join(sisa).exists(), "{sisa}");
    }
    // tanpa sisa: tidak mengubah apa pun
    cadangan::bereskan_sisa(p);
    assert_eq!(nama_barang(&c), vec!["Barang A"]);
}
