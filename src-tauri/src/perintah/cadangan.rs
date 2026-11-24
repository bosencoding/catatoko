use super::dengan_sesi;
use crate::cadangan;
use crate::galat::Hasil;
use crate::state::{kunci, AppState};
use std::path::PathBuf;
use tauri::State;

#[tauri::command(rename_all = "snake_case")]
pub fn cadangan_buat(st: State<AppState>, tujuan: Option<String>) -> Hasil<String> {
    dengan_sesi(&st, |c, s| {
        let tujuan = tujuan.map(PathBuf::from);
        Ok(cadangan::cadangkan(c, &st.data, s, tujuan.as_deref())?.to_string_lossy().into_owned())
    })
}

/// Setelah berhasil, sesi dihapus: pengguna masuk lagi dengan akun dari data yang dipulihkan.
#[tauri::command(rename_all = "snake_case")]
pub fn cadangan_pulihkan(st: State<AppState>, sumber: String) -> Hasil<()> {
    dengan_sesi(&st, |c, s| cadangan::pulihkan(c, &st.data, s, &PathBuf::from(sumber)))?;
    *kunci(&st.sesi) = None;
    Ok(())
}
