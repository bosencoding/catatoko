//! Lokasi folder data. Selalu di sebelah program; tidak pernah pindah diam-diam.

use crate::galat::{Galat, Hasil};
use std::path::{Path, PathBuf};

/// Lokasi folder data: override pengembangan (`CATAT_DATA_DIR`) atau `data/` di sebelah program.
/// Sengaja tidak ada sumber lain (mis. variabel lingkungan milik aplikasi lain), supaya data
/// tidak pernah pindah diam-diam.
pub fn pilih_folder(override_dev: Option<std::ffi::OsString>, program: &Path) -> Option<PathBuf> {
    match override_dev {
        Some(d) => Some(PathBuf::from(d)),
        None => program.parent().map(|p| p.join("data")),
    }
}

/// Menentukan dan menyiapkan folder `data/`.
pub fn folder_data() -> Hasil<PathBuf> {
    let tak_terbaca = |e: &dyn std::fmt::Display| Galat::baru("FOLDER_DATA", format!("Lokasi program tidak bisa dibaca: {e}"));
    let program = std::env::current_exe().map_err(|e| tak_terbaca(&e))?;
    let dir = pilih_folder(std::env::var_os("CATAT_DATA_DIR"), &program).ok_or_else(|| tak_terbaca(&"tanpa folder induk"))?;
    siapkan(&dir)?;
    Ok(dir)
}

/// Memegang kunci berkas selama aplikasi terbuka, supaya folder data yang sama tidak dipakai
/// dua jendela sekaligus. Kunci lepas sendiri saat `File` yang dikembalikan ditutup.
pub fn kunci_tunggal(dir: &Path) -> Hasil<std::fs::File> {
    let berkas = std::fs::OpenOptions::new().create(true).truncate(false).write(true).open(dir.join(".catat.kunci"))?;
    berkas.try_lock().map_err(|_| {
        Galat::baru("SUDAH_TERBUKA", "Catat sudah terbuka di jendela lain dengan folder data yang sama. Pakai jendela itu, atau tutup dulu sebelum membuka lagi.")
    })?;
    Ok(berkas)
}

/// Apakah `jalur` berada di dalam `dir` (setelah `.` dan `..` diselesaikan)?
pub fn di_dalam(dir: &Path, jalur: &Path) -> bool {
    use std::path::Component;
    let rapikan = |p: &Path| {
        let mut hasil = PathBuf::new();
        for k in p.components() {
            match k {
                Component::ParentDir => {
                    hasil.pop();
                }
                Component::CurDir => {}
                lain => hasil.push(lain),
            }
        }
        hasil
    };
    let mutlak = |p: &Path| if p.is_absolute() { p.to_path_buf() } else { std::env::current_dir().unwrap_or_default().join(p) };
    rapikan(&mutlak(jalur)).starts_with(rapikan(&mutlak(dir)))
}

/// Membuat `dir`, `dir/foto`, `dir/cadangan` dan memastikan bisa ditulis.
pub fn siapkan(dir: &Path) -> Hasil<()> {
    let gagal = |e: std::io::Error| {
        Galat::baru(
            "FOLDER_DATA",
            format!(
                "Folder data tidak bisa ditulis: {}\n({e})\nPindahkan folder Catat ke tempat yang bisa ditulis, misalnya folder Dokumen atau flashdisk yang tidak dikunci.",
                dir.display()
            ),
        )
    };
    for d in [dir.to_path_buf(), dir.join("foto"), dir.join("cadangan")] {
        std::fs::create_dir_all(&d).map_err(gagal)?;
    }
    let uji = dir.join(".uji-tulis");
    std::fs::write(&uji, b"ok").map_err(gagal)?;
    std::fs::remove_file(&uji).map_err(gagal)?;
    Ok(())
}
