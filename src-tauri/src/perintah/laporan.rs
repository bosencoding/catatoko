use super::dengan_sesi;
use crate::galat::{Galat, Hasil};
use crate::laporan::{self, Jenis, Laporan};
use crate::ringkasan::{self, Butir, Ringkasan};
use crate::state::AppState;
use crate::{ekspor, log, waktu};
use std::path::PathBuf;
use tauri::State;

#[tauri::command(rename_all = "snake_case")]
pub fn ringkasan(st: State<AppState>, dari: String, sampai: String, butir: Butir) -> Hasil<Ringkasan> {
    dengan_sesi(&st, |c, s| ringkasan::ringkasan(c, s, &dari, &sampai, butir, &waktu::hari_ini()))
}

#[tauri::command(rename_all = "snake_case")]
pub fn laporan_buat(st: State<AppState>, jenis: Jenis, dari: String, sampai: String) -> Hasil<Laporan> {
    dengan_sesi(&st, |c, s| laporan::buat(c, s, jenis, &dari, &sampai))
}

/// Menulis laporan ke `tujuan` (jalur yang dipilih pengguna lewat dialog simpan).
#[tauri::command(rename_all = "snake_case")]
pub fn laporan_ekspor(st: State<AppState>, jenis: Jenis, dari: String, sampai: String, format: String, tujuan: String) -> Hasil<String> {
    dengan_sesi(&st, |c, s| {
        let l = laporan::buat(c, s, jenis, &dari, &sampai)?;
        let mut jalur = PathBuf::from(&tujuan);
        if crate::jalur::di_dalam(&st.data, &jalur) {
            return Err(Galat::validasi("Pilih lokasi di luar folder data aplikasi, misalnya folder Dokumen."));
        }
        if !matches!(format.as_str(), "xlsx" | "pdf") {
            return Err(Galat::validasi("Format ekspor harus xlsx atau pdf."));
        }
        if jalur.extension().and_then(|e| e.to_str()).map(str::to_lowercase).as_deref() != Some(format.as_str()) {
            jalur.set_extension(&format);
        }
        if format == "xlsx" { ekspor::xlsx(&l, &jalur)? } else { ekspor::pdf(&l, &jalur)? }
        log::catat(c, Some(s.pengguna_id), "laporan.ekspor", &format!("{} ({}) diekspor ke {}", l.judul, l.periode, format.to_uppercase()))?;
        Ok(jalur.to_string_lossy().into_owned())
    })
}
