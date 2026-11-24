pub mod akses;
pub mod auth;
pub mod db;
pub mod galat;
pub mod jalur;
pub mod log;
pub mod pengaturan;
pub mod pengguna;
pub mod perintah;
pub mod state;
#[cfg(test)]
mod uji;
pub mod waktu;

use tauri::Manager;
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

fn siapkan_state() -> galat::Hasil<state::AppState> {
    let data = jalur::folder_data()?;
    let conn = db::buka(&data.join("catat.db"))?;
    Ok(state::AppState::baru(conn, data))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use perintah as p;
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            match siapkan_state() {
                Ok(st) => {
                    app.manage(st);
                }
                Err(e) => {
                    app.dialog().message(e.pesan.clone()).title("Catat tidak bisa dibuka").kind(MessageDialogKind::Error).blocking_show();
                    eprintln!("{}", e.pesan);
                    std::process::exit(1);
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            p::auth::perlu_setup,
            p::auth::setup_awal,
            p::auth::login,
            p::auth::logout,
            p::auth::sesi_sekarang,
            p::auth::sentuh,
            p::auth::pulihkan_sandi,
            p::auth::ganti_sandi_sendiri,
            p::auth::pengaturan_ambil,
            p::auth::pengaturan_simpan,
            p::auth::log_daftar,
            p::pengguna::pengguna_daftar,
            p::pengguna::pengguna_tambah,
            p::pengguna::pengguna_ubah,
            p::pengguna::pengguna_atur_ulang_sandi,
            p::pengguna::hak_bawaan_staf,
        ])
        .run(tauri::generate_context!())
        .expect("gagal menjalankan aplikasi");
}
