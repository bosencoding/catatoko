pub mod akses;
pub mod auth;
pub mod barang;
pub mod db;
pub mod foto;
pub mod galat;
pub mod jalur;
pub mod kas;
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
                    // Tampilan hanya boleh membaca file di dalam data/foto.
                    let _ = app.asset_protocol_scope().allow_directory(st.data.join("foto"), true);
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
            p::barang::barang_daftar,
            p::barang::barang_ambil,
            p::barang::barang_tambah,
            p::barang::barang_ubah,
            p::barang::kategori_barang_daftar,
            p::barang::kategori_barang_tambah,
            p::barang::folder_data,
            p::kas::kas_daftar,
            p::kas::kas_saldo,
            p::kas::kas_tambah,
            p::kas::kas_ubah,
            p::kas::kas_batal,
            p::kas::kas_pindah,
            p::kas::akun_kas_daftar,
            p::kas::akun_kas_tambah,
            p::kas::akun_kas_ubah,
            p::kas::kategori_kas_daftar,
            p::kas::kategori_kas_tambah,
        ])
        .run(tauri::generate_context!())
        .expect("gagal menjalankan aplikasi");
}
