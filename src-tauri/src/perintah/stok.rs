use super::dengan_sesi;
use crate::galat::Hasil;
use crate::state::AppState;
use crate::stok::{self, Kiriman, MasukMasukan};
use crate::waktu;
use tauri::State;

#[tauri::command(rename_all = "snake_case")]
pub fn masuk_tambah(st: State<AppState>, masukan: MasukMasukan) -> Hasil<i64> {
    dengan_sesi(&st, |c, s| stok::masuk(c, &st.data, s, masukan))
}

#[tauri::command(rename_all = "snake_case")]
pub fn masuk_daftar(st: State<AppState>, dari: String, sampai: String) -> Hasil<Vec<Kiriman>> {
    dengan_sesi(&st, |c, s| stok::daftar_masuk(c, s, &dari, &sampai))
}

#[tauri::command(rename_all = "snake_case")]
pub fn kiriman_barang(st: State<AppState>, barang_id: i64, hanya_bersisa: bool) -> Hasil<Vec<Kiriman>> {
    dengan_sesi(&st, |c, s| stok::kiriman_barang(c, s, barang_id, hanya_bersisa, &waktu::hari_ini()))
}
