//! Cadangkan dan pulihkan: satu zip berisi `manifest.json`, `catat.db`, dan `foto/*`.

use crate::akses::{wajib_pemilik, Sesi};
use crate::galat::{Galat, Hasil};
use crate::{db, log};
use rusqlite::{Connection, OpenFlags};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

const UKURAN_MAKS: u64 = 8 * 1024 * 1024 * 1024;

fn bukan_cadangan(alasan: &str) -> Galat {
    Galat::validasi(format!("File ini bukan cadangan Catat yang bisa dipakai ({alasan}). Data Anda tidak diubah."))
}

fn galat_zip(e: impl std::fmt::Display) -> Galat {
    Galat::internal(format!("gagal menulis cadangan: {e}"))
}

/// Jalur zip yang belum dipakai di `dir`, berawalan `awalan` dan bercap waktu.
fn nama_baru(dir: &Path, awalan: &str) -> PathBuf {
    let cap = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let mut calon = dir.join(format!("{awalan}-{cap}.zip"));
    let mut n = 2;
    while calon.exists() {
        calon = dir.join(format!("{awalan}-{cap}-{n}.zip"));
        n += 1;
    }
    calon
}

fn tulis_zip(conn: &Connection, data: &Path, tujuan: &Path) -> Hasil<()> {
    // Salinan konsisten dari database yang sedang terbuka.
    let sementara = data.join("cadangan").join(format!(".tmp-{}.db", uuid::Uuid::new_v4().simple()));
    let hasil = (|| {
        conn.execute("VACUUM INTO ?1", [sementara.to_string_lossy().as_ref()])?;
        let file = File::create(tujuan).map_err(|e| Galat::validasi(format!("Cadangan tidak bisa disimpan di lokasi itu. Pilih folder lain. ({e})")))?;
        let mut zip = ZipWriter::new(file);
        let padat = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated).large_file(true);
        let apa_adanya = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        let manifest = serde_json::json!({ "aplikasi": "catat", "versi_skema": db::versi_skema_terbaru(), "dibuat": crate::waktu::kini() });
        zip.start_file("manifest.json", padat).map_err(galat_zip)?;
        zip.write_all(manifest.to_string().as_bytes())?;
        zip.start_file("catat.db", padat).map_err(galat_zip)?;
        std::io::copy(&mut File::open(&sementara)?, &mut zip)?;
        for entri in std::fs::read_dir(data.join("foto"))? {
            let entri = entri?;
            if entri.file_type()?.is_file() {
                zip.start_file(format!("foto/{}", entri.file_name().to_string_lossy()), apa_adanya).map_err(galat_zip)?;
                std::io::copy(&mut File::open(entri.path())?, &mut zip)?;
            }
        }
        zip.finish().map_err(galat_zip)?;
        Ok(())
    })();
    let _ = std::fs::remove_file(&sementara);
    if hasil.is_err() {
        let _ = std::fs::remove_file(tujuan);
    }
    hasil
}

/// Membuat cadangan. Tanpa `tujuan`, disimpan di `data/cadangan/catat-YYYYMMDD-HHMMSS.zip`.
pub fn cadangkan(conn: &Connection, data: &Path, s: &Sesi, tujuan: Option<&Path>) -> Hasil<PathBuf> {
    wajib_pemilik(s)?;
    let tujuan = tujuan.map(Path::to_path_buf).unwrap_or_else(|| nama_baru(&data.join("cadangan"), "catat"));
    tulis_zip(conn, data, &tujuan)?;
    log::catat(conn, Some(s.pengguna_id), "cadangan.buat", &format!("Cadangan dibuat: {}", tujuan.display()))?;
    Ok(tujuan)
}

enum Entri {
    Manifest,
    Db,
    Foto(String),
    Folder,
}

fn kenali(nama: &str, folder: bool) -> Option<Entri> {
    match nama {
        "manifest.json" => Some(Entri::Manifest),
        "catat.db" => Some(Entri::Db),
        "foto/" | "foto" if folder => Some(Entri::Folder),
        _ => {
            let file = nama.strip_prefix("foto/")?;
            let aman = !file.is_empty() && !file.starts_with('.') && file.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'));
            aman.then(|| Entri::Foto(file.to_string()))
        }
    }
}

/// Mengekstrak ke folder sementara setelah memastikan isinya memang cadangan Catat.
fn ekstrak(sumber: &Path, tujuan: &Path) -> Hasil<()> {
    let file = File::open(sumber).map_err(|_| bukan_cadangan("file tidak bisa dibuka"))?;
    let mut arsip = ZipArchive::new(file).map_err(|_| bukan_cadangan("bukan file zip"))?;
    let (mut ada_manifest, mut ada_db, mut total) = (false, false, 0u64);
    // Periksa semua nama dulu sebelum menulis apa pun.
    for i in 0..arsip.len() {
        let e = arsip.by_index(i).map_err(|_| bukan_cadangan("zip rusak"))?;
        total += e.size();
        match kenali(e.name(), e.is_dir()) {
            Some(Entri::Manifest) => ada_manifest = true,
            Some(Entri::Db) => ada_db = true,
            Some(_) => {}
            None => return Err(bukan_cadangan("berisi file yang tidak dikenal")),
        }
    }
    if !ada_manifest || !ada_db {
        return Err(bukan_cadangan("isi tidak lengkap"));
    }
    if total > UKURAN_MAKS {
        return Err(bukan_cadangan("ukuran terlalu besar"));
    }
    let mut manifest = String::new();
    arsip.by_name("manifest.json").and_then(|mut e| Ok(e.read_to_string(&mut manifest)?)).map_err(|_| bukan_cadangan("manifest rusak"))?;
    let manifest: serde_json::Value = serde_json::from_str(&manifest).map_err(|_| bukan_cadangan("manifest rusak"))?;
    if manifest["aplikasi"] != "catat" {
        return Err(bukan_cadangan("bukan dari aplikasi Catat"));
    }
    let versi = manifest["versi_skema"].as_i64().ok_or_else(|| bukan_cadangan("manifest rusak"))?;
    if versi > db::versi_skema_terbaru() {
        return Err(bukan_cadangan("dibuat oleh versi Catat yang lebih baru; perbarui aplikasinya dulu"));
    }
    std::fs::create_dir_all(tujuan.join("foto"))?;
    for i in 0..arsip.len() {
        let mut e = arsip.by_index(i).map_err(|_| bukan_cadangan("zip rusak"))?;
        let ke = match kenali(e.name(), e.is_dir()) {
            Some(Entri::Db) => tujuan.join("catat.db"),
            Some(Entri::Foto(nama)) => tujuan.join("foto").join(nama),
            _ => continue,
        };
        std::io::copy(&mut e, &mut File::create(ke)?).map_err(|_| bukan_cadangan("zip rusak"))?;
    }
    Ok(())
}

/// Memastikan database hasil ekstrak sehat dan bisa dipakai masuk.
fn periksa_db(path: &Path) -> Hasil<()> {
    let rusak = || bukan_cadangan("databasenya rusak");
    let c = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(|_| rusak())?;
    let sehat: String = c.query_row("PRAGMA integrity_check", [], |r| r.get(0)).map_err(|_| rusak())?;
    if sehat != "ok" {
        return Err(rusak());
    }
    let versi: i64 = c.query_row("PRAGMA user_version", [], |r| r.get(0)).map_err(|_| rusak())?;
    if versi < 1 || versi > db::versi_skema_terbaru() {
        return Err(bukan_cadangan("versi datanya tidak cocok dengan aplikasi ini"));
    }
    let pemilik: i64 = c.query_row("SELECT count(*) FROM pengguna WHERE peran='pemilik' AND aktif=1", [], |r| r.get(0)).map_err(|_| rusak())?;
    if pemilik == 0 {
        return Err(bukan_cadangan("tidak ada akun pemilik di dalamnya"));
    }
    Ok(())
}

/// Mengganti data sekarang dengan isi cadangan. Data sekarang dicadangkan otomatis lebih dulu.
/// `conn` diganti dengan koneksi ke database yang dipulihkan; pemanggil wajib menghapus sesi.
pub fn pulihkan(conn: &mut Connection, data: &Path, s: &Sesi, sumber: &Path) -> Hasil<()> {
    wajib_pemilik(s)?;
    let sementara = data.join(format!(".pulih-{}", uuid::Uuid::new_v4().simple()));
    let hasil = (|| {
        ekstrak(sumber, &sementara)?;
        periksa_db(&sementara.join("catat.db"))?;
        let otomatis = nama_baru(&data.join("cadangan"), "catat-sebelum-pulih");
        tulis_zip(conn, data, &otomatis)?;
        tukar(conn, data, &sementara)?;
        log::catat(
            conn,
            None,
            "cadangan.pulihkan",
            &format!("Data dipulihkan dari {} oleh {}. Data sebelumnya disimpan di {}", sumber.display(), s.nama, otomatis.display()),
        )
    })();
    let _ = std::fs::remove_dir_all(&sementara);
    hasil
}

fn tukar(conn: &mut Connection, data: &Path, sementara: &Path) -> Hasil<()> {
    let (db_kini, db_lama) = (data.join("catat.db"), data.join("catat.db.lama"));
    let (foto_kini, foto_lama) = (data.join("foto"), data.join("foto.lama"));
    // Lepas file database yang sedang dipakai.
    drop(std::mem::replace(conn, Connection::open_in_memory()?));
    let pindah = (|| -> std::io::Result<()> {
        std::fs::rename(&db_kini, &db_lama)?;
        std::fs::rename(&foto_kini, &foto_lama)?;
        std::fs::rename(sementara.join("catat.db"), &db_kini)?;
        std::fs::rename(sementara.join("foto"), &foto_kini)
    })();
    let dibuka = pindah.map_err(Galat::from).and_then(|_| db::buka(&db_kini));
    match dibuka {
        Ok(baru) => {
            *conn = baru;
            let _ = std::fs::remove_file(&db_lama);
            let _ = std::fs::remove_dir_all(&foto_lama);
            Ok(())
        }
        Err(e) => {
            // Kembalikan keadaan semula.
            if db_lama.exists() {
                let _ = std::fs::remove_file(&db_kini);
                let _ = std::fs::rename(&db_lama, &db_kini);
            }
            if foto_lama.exists() {
                let _ = std::fs::remove_dir_all(&foto_kini);
                let _ = std::fs::rename(&foto_lama, &foto_kini);
            }
            *conn = db::buka(&db_kini)?;
            Err(Galat::internal(format!("pemulihan gagal dan data lama dikembalikan: {}", e.pesan)))
        }
    }
}
