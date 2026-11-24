//! Alat bantu uji antarmuka. HANYA ada di build debug; build rilis tidak memuatnya.
//!
//! `CATAT_UJI_JS=/jalur/skrip.js` → skrip dijalankan di webview setelah halaman dimuat.
//! `CATAT_UJI_OUT=/folder` → perintah `uji_tulis` menulis hasil skrip ke folder itu.

use tauri::webview::{PageLoadEvent, PageLoadPayload};
use tauri::Webview;

pub fn saat_halaman_dimuat(webview: &Webview, muatan: &PageLoadPayload<'_>) {
    if muatan.event() != PageLoadEvent::Finished {
        return;
    }
    let Some(jalur) = std::env::var_os("CATAT_UJI_JS") else { return };
    match std::fs::read_to_string(&jalur) {
        Ok(skrip) => {
            let _ = webview.eval(&skrip);
        }
        Err(e) => eprintln!("CATAT_UJI_JS tidak terbaca: {e}"),
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn uji_tulis(nama: String, isi: String) -> Result<(), String> {
    let folder = std::env::var_os("CATAT_UJI_OUT").ok_or("CATAT_UJI_OUT belum diatur")?;
    if nama.is_empty() || !nama.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-')) || nama.starts_with('.') {
        return Err("nama file tidak sah".into());
    }
    std::fs::write(std::path::Path::new(&folder).join(nama), isi).map_err(|e| e.to_string())
}
