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

#[tauri::command(rename_all = "snake_case")]
pub fn usul_alokasi(st: State<AppState>, barang_id: i64, jumlah: i64, terpakai: Option<Vec<stok::Alokasi>>) -> Hasil<Vec<stok::Alokasi>> {
    dengan_sesi(&st, |c, s| stok::usul_alokasi_sisa(c, s, barang_id, jumlah, &waktu::hari_ini(), terpakai.as_deref().unwrap_or(&[])))
}

#[tauri::command(rename_all = "snake_case")]
pub fn keluar_tambah(st: State<AppState>, masukan: stok::KeluarMasukan) -> Hasil<i64> {
    dengan_sesi(&st, |c, s| stok::keluar(c, s, masukan, &waktu::hari_ini()))
}

#[tauri::command(rename_all = "snake_case")]
pub fn keluar_daftar(st: State<AppState>, dari: String, sampai: String) -> Hasil<Vec<stok::TransaksiKeluar>> {
    dengan_sesi(&st, |c, s| stok::daftar_keluar(c, s, &dari, &sampai))
}

#[tauri::command(rename_all = "snake_case")]
pub fn keluar_batal(st: State<AppState>, id: i64, alasan: String) -> Hasil<()> {
    dengan_sesi(&st, |c, s| stok::batal_keluar(c, s, id, &alasan))
}

#[tauri::command(rename_all = "snake_case")]
pub fn masuk_batal(st: State<AppState>, kiriman_id: i64, alasan: String) -> Hasil<()> {
    dengan_sesi(&st, |c, s| stok::batal_masuk(c, s, kiriman_id, &alasan))
}

#[tauri::command(rename_all = "snake_case")]
pub fn kiriman_perbaiki(st: State<AppState>, id: i64, perbaikan: stok::PerbaikanKiriman) -> Hasil<()> {
    dengan_sesi(&st, |c, s| stok::perbaiki_kiriman(c, &st.data, s, id, perbaikan))
}

#[tauri::command(rename_all = "snake_case")]
pub fn stok_sesuaikan(st: State<AppState>, kiriman_id: i64, tanggal: String, selisih: i64, alasan: String, catatan: Option<String>) -> Hasil<i64> {
    dengan_sesi(&st, |c, s| stok::sesuaikan(c, s, kiriman_id, &tanggal, selisih, &alasan, catatan.as_deref()))
}

#[tauri::command(rename_all = "snake_case")]
pub fn penyesuaian_batal(st: State<AppState>, id: i64, alasan: String) -> Hasil<()> {
    dengan_sesi(&st, |c, s| stok::batal_penyesuaian(c, s, id, &alasan))
}

#[tauri::command(rename_all = "snake_case")]
pub fn penyesuaian_daftar(st: State<AppState>, dari: String, sampai: String) -> Hasil<Vec<stok::Penyesuaian>> {
    dengan_sesi(&st, |c, s| stok::daftar_penyesuaian(c, s, &dari, &sampai))
}

#[tauri::command(rename_all = "snake_case")]
pub fn masuk_terbaru(st: State<AppState>, batas: u32) -> Hasil<Vec<Kiriman>> {
    dengan_sesi(&st, |c, s| stok::masuk_terbaru(c, s, batas))
}

#[tauri::command(rename_all = "snake_case")]
pub fn keluar_terbaru(st: State<AppState>, batas: u32) -> Hasil<Vec<stok::TransaksiKeluar>> {
    dengan_sesi(&st, |c, s| stok::keluar_terbaru(c, s, batas))
}
