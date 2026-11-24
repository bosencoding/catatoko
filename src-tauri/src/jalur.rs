//! Lokasi folder data. Selalu di sebelah program; tidak pernah pindah diam-diam.

use crate::galat::{Galat, Hasil};
use std::path::{Path, PathBuf};

/// Menentukan dan menyiapkan folder `data/`.
pub fn folder_data() -> Hasil<PathBuf> {
    let dir = if let Some(d) = std::env::var_os("CATAT_DATA_DIR") {
        PathBuf::from(d)
    } else if let Some(p) = std::env::var_os("APPIMAGE").map(PathBuf::from).and_then(|p| p.parent().map(Path::to_path_buf)) {
        p.join("data")
    } else {
        let exe = std::env::current_exe().map_err(|e| Galat::baru("FOLDER_DATA", format!("Lokasi program tidak bisa dibaca: {e}")))?;
        exe.parent().map(|p| p.join("data")).ok_or_else(|| Galat::baru("FOLDER_DATA", "Lokasi program tidak bisa dibaca."))?
    };
    siapkan(&dir)?;
    Ok(dir)
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
