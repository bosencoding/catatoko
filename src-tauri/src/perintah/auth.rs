use super::{dengan_sesi, periksa_sesi};
use crate::akses::Sesi;
use crate::galat::Hasil;
use crate::pengaturan::{self, Pengaturan};
use crate::state::{kunci, AppState, Mulai};
use crate::{auth, log};
use std::time::Instant;
use tauri::State;

/// Pesan galat penyiapan folder data, atau `None` bila aplikasi siap dipakai.
#[tauri::command(rename_all = "snake_case")]
pub fn status_awal(m: State<Mulai>) -> Option<String> {
    m.galat.clone()
}

#[tauri::command(rename_all = "snake_case")]
pub fn perlu_setup(st: State<AppState>) -> Hasil<bool> {
    auth::perlu_setup(&kunci(&st.db))
}

#[tauri::command(rename_all = "snake_case")]
pub fn setup_awal(st: State<AppState>, nama_toko: String, nama_pengguna: String, nama: String, sandi: String) -> Hasil<String> {
    auth::setup_awal(&mut kunci(&st.db), &nama_toko, &nama_pengguna, &nama, &sandi)
}

#[tauri::command(rename_all = "snake_case")]
pub fn login(st: State<AppState>, nama_pengguna: String, sandi: String) -> Hasil<Sesi> {
    let db = kunci(&st.db);
    let sesi = auth::login(&db, &nama_pengguna, &sandi, chrono::Local::now())?;
    *kunci(&st.sesi) = Some(sesi.clone());
    *kunci(&st.terakhir_aktif) = Instant::now();
    Ok(sesi)
}

#[tauri::command(rename_all = "snake_case")]
pub fn logout(st: State<AppState>) -> Hasil<()> {
    let db = kunci(&st.db);
    if let Some(s) = kunci(&st.sesi).take() {
        log::catat(&db, Some(s.pengguna_id), "auth.logout", "Keluar")?;
    }
    Ok(())
}

/// Sesi saat ini tanpa memperpanjang waktu aktif (dipakai untuk pemeriksaan berkala).
#[tauri::command(rename_all = "snake_case")]
pub fn sesi_sekarang(st: State<AppState>) -> Hasil<Option<Sesi>> {
    let db = kunci(&st.db);
    match periksa_sesi(&st, &db, false) {
        Ok(s) => Ok(Some(s)),
        Err(e) if e.kode == "BELUM_LOGIN" || e.kode == "TERKUNCI" => Ok(None),
        Err(e) => Err(e),
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn sentuh(st: State<AppState>) -> Hasil<()> {
    dengan_sesi(&st, |_, _| Ok(()))
}

#[tauri::command(rename_all = "snake_case")]
pub fn pulihkan_sandi(st: State<AppState>, kode: String, nama_pengguna: String, sandi_baru: String) -> Hasil<String> {
    auth::pulihkan_sandi(&mut kunci(&st.db), &kode, &nama_pengguna, &sandi_baru)
}

#[tauri::command(rename_all = "snake_case")]
pub fn ganti_sandi_sendiri(st: State<AppState>, lama: String, baru: String) -> Hasil<()> {
    dengan_sesi(&st, |c, s| auth::ganti_sandi_sendiri(c, s, &lama, &baru))
}

#[tauri::command(rename_all = "snake_case")]
pub fn pengaturan_ambil(st: State<AppState>) -> Hasil<Pengaturan> {
    pengaturan::ambil(&kunci(&st.db))
}

#[tauri::command(rename_all = "snake_case")]
pub fn pengaturan_simpan(st: State<AppState>, kunci: String, nilai: String) -> Hasil<()> {
    dengan_sesi(&st, |c, s| pengaturan::simpan(c, s, &kunci, &nilai))
}

#[tauri::command(rename_all = "snake_case")]
pub fn log_daftar(st: State<AppState>, dari: String, sampai: String) -> Hasil<Vec<log::BarisLog>> {
    dengan_sesi(&st, |c, s| log::daftar(c, s, &dari, &sampai))
}
