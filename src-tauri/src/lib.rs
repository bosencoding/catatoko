pub mod db;
pub mod galat;
pub mod jalur;
#[cfg(test)]
mod uji;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .run(tauri::generate_context!())
        .expect("gagal menjalankan aplikasi");
}
