use super::dengan_sesi;
use crate::akses::{Modul, Tingkat};
use crate::galat::Hasil;
use crate::pengguna::{self, Pengguna, PenggunaMasukan};
use crate::state::AppState;
use std::collections::HashMap;
use tauri::State;

#[tauri::command(rename_all = "snake_case")]
pub fn pengguna_daftar(st: State<AppState>) -> Hasil<Vec<Pengguna>> {
    dengan_sesi(&st, |c, s| pengguna::daftar(c, s))
}

#[tauri::command(rename_all = "snake_case")]
pub fn pengguna_tambah(st: State<AppState>, masukan: PenggunaMasukan, sandi: String) -> Hasil<i64> {
    dengan_sesi(&st, |c, s| pengguna::tambah(c, s, masukan, &sandi))
}

#[tauri::command(rename_all = "snake_case")]
pub fn pengguna_ubah(st: State<AppState>, id: i64, masukan: PenggunaMasukan) -> Hasil<()> {
    dengan_sesi(&st, |c, s| pengguna::ubah(c, s, id, masukan))
}

#[tauri::command(rename_all = "snake_case")]
pub fn pengguna_atur_ulang_sandi(st: State<AppState>, id: i64, sandi_baru: String) -> Hasil<()> {
    dengan_sesi(&st, |c, s| pengguna::atur_ulang_sandi(c, s, id, &sandi_baru))
}

#[tauri::command(rename_all = "snake_case")]
pub fn hak_bawaan_staf() -> HashMap<Modul, Tingkat> {
    pengguna::hak_bawaan_staf()
}
