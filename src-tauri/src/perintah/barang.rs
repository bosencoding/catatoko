use super::dengan_sesi;
use crate::barang::{self, Barang, BarangMasukan, Kategori};
use crate::galat::Hasil;
use crate::state::AppState;
use tauri::State;

#[tauri::command(rename_all = "snake_case")]
pub fn barang_daftar(st: State<AppState>, cari: Option<String>, termasuk_nonaktif: bool) -> Hasil<Vec<Barang>> {
    dengan_sesi(&st, |c, s| barang::daftar(c, s, cari.as_deref(), termasuk_nonaktif))
}

#[tauri::command(rename_all = "snake_case")]
pub fn barang_ambil(st: State<AppState>, id: i64) -> Hasil<Barang> {
    dengan_sesi(&st, |c, s| barang::ambil(c, s, id))
}

#[tauri::command(rename_all = "snake_case")]
pub fn barang_tambah(st: State<AppState>, masukan: BarangMasukan) -> Hasil<i64> {
    dengan_sesi(&st, |c, s| barang::tambah(c, &st.data, s, masukan))
}

#[tauri::command(rename_all = "snake_case")]
pub fn barang_ubah(st: State<AppState>, id: i64, masukan: BarangMasukan) -> Hasil<()> {
    dengan_sesi(&st, |c, s| barang::ubah(c, &st.data, s, id, masukan))
}

#[tauri::command(rename_all = "snake_case")]
pub fn kategori_barang_daftar(st: State<AppState>) -> Hasil<Vec<Kategori>> {
    dengan_sesi(&st, |c, s| barang::kategori_daftar(c, s))
}

#[tauri::command(rename_all = "snake_case")]
pub fn kategori_barang_tambah(st: State<AppState>, nama: String) -> Hasil<i64> {
    dengan_sesi(&st, |c, s| barang::kategori_tambah(c, s, &nama))
}

/// Lokasi folder data, dipakai tampilan untuk membentuk alamat foto.
#[tauri::command(rename_all = "snake_case")]
pub fn folder_data(st: State<AppState>) -> String {
    st.data.to_string_lossy().into_owned()
}
