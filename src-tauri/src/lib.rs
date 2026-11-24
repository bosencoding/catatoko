pub mod akses;
pub mod auth;
pub mod barang;
pub mod cadangan;
pub mod db;
pub mod ekspor;
pub mod foto;
pub mod galat;
pub mod jalur;
pub mod kas;
pub mod laporan;
pub mod log;
pub mod pengaturan;
pub mod pengguna;
pub mod perintah;
pub mod ringkasan;
pub mod state;
pub mod stok;
#[cfg(test)]
mod uji;
#[cfg(debug_assertions)]
mod uji_ui;
pub mod waktu;

use tauri::Manager;

fn siapkan_state() -> galat::Hasil<state::AppState> {
    let data = jalur::folder_data()?;
    let conn = db::buka(&data.join("catat.db"))?;
    Ok(state::AppState::baru(conn, data))
}

/// Daftar perintah Tauri. Tambahan (hanya untuk build debug) bisa diselipkan lewat argumen.
macro_rules! daftar_perintah {
    ($($tambahan:path),*) => {
        tauri::generate_handler![
            perintah::auth::status_awal,
            perintah::auth::perlu_setup,
            perintah::auth::setup_awal,
            perintah::auth::login,
            perintah::auth::logout,
            perintah::auth::sesi_sekarang,
            perintah::auth::sentuh,
            perintah::auth::pulihkan_sandi,
            perintah::auth::ganti_sandi_sendiri,
            perintah::auth::pengaturan_ambil,
            perintah::auth::pengaturan_simpan,
            perintah::auth::log_daftar,
            perintah::pengguna::pengguna_daftar,
            perintah::pengguna::pengguna_tambah,
            perintah::pengguna::pengguna_ubah,
            perintah::pengguna::pengguna_atur_ulang_sandi,
            perintah::pengguna::hak_bawaan_staf,
            perintah::barang::barang_daftar,
            perintah::barang::barang_ambil,
            perintah::barang::barang_tambah,
            perintah::barang::barang_ubah,
            perintah::barang::kategori_barang_daftar,
            perintah::barang::kategori_barang_tambah,
            perintah::barang::folder_data,
            perintah::kas::kas_daftar,
            perintah::kas::kas_saldo,
            perintah::kas::kas_tambah,
            perintah::kas::kas_ubah,
            perintah::kas::kas_batal,
            perintah::kas::kas_pindah,
            perintah::kas::akun_kas_daftar,
            perintah::kas::akun_kas_tambah,
            perintah::kas::akun_kas_ubah,
            perintah::kas::kategori_kas_daftar,
            perintah::kas::kategori_kas_tambah,
            perintah::stok::masuk_tambah,
            perintah::stok::masuk_daftar,
            perintah::stok::kiriman_barang,
            perintah::stok::usul_alokasi,
            perintah::stok::keluar_tambah,
            perintah::stok::keluar_daftar,
            perintah::stok::keluar_batal,
            perintah::stok::masuk_batal,
            perintah::stok::kiriman_perbaiki,
            perintah::stok::stok_sesuaikan,
            perintah::stok::penyesuaian_batal,
            perintah::stok::penyesuaian_daftar,
            perintah::stok::masuk_terbaru,
            perintah::stok::keluar_terbaru,
            perintah::laporan::ringkasan,
            perintah::laporan::laporan_buat,
            perintah::laporan::laporan_ekspor,
            perintah::cadangan::cadangan_buat,
            perintah::cadangan::cadangan_pulihkan,
            $($tambahan),*
        ]
    };
}

#[cfg(not(debug_assertions))]
fn penangan() -> impl Fn(tauri::ipc::Invoke) -> bool + Send + Sync + 'static {
    daftar_perintah!()
}

#[cfg(debug_assertions)]
fn penangan() -> impl Fn(tauri::ipc::Invoke) -> bool + Send + Sync + 'static {
    daftar_perintah!(uji_ui::uji_tulis)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .on_page_load(|_webview, _muatan| {
            #[cfg(debug_assertions)]
            uji_ui::saat_halaman_dimuat(_webview, _muatan);
        })
        .setup(|app| {
            // Gagal menyiapkan folder data bukan alasan untuk mati diam-diam:
            // jendela tetap dibuka dan tampilan menampilkan pesannya.
            match siapkan_state() {
                Ok(st) => {
                    // Tampilan hanya boleh membaca file di dalam data/foto.
                    let _ = app.asset_protocol_scope().allow_directory(st.data.join("foto"), true);
                    app.manage(st);
                    app.manage(state::Mulai { galat: None });
                }
                Err(e) => {
                    eprintln!("{}", e.pesan);
                    app.manage(state::Mulai { galat: Some(e.pesan) });
                }
            }
            Ok(())
        })
        .invoke_handler(penangan())
        .run(tauri::generate_context!())
        .expect("gagal menjalankan aplikasi");
}
