//! Pembungkus tipis `#[tauri::command]`. Semua aturan ada di modul inti.

use crate::akses::Sesi;
use crate::galat::{Galat, Hasil};
use crate::pengaturan;
use crate::state::{kunci, AppState};
use rusqlite::Connection;
use std::time::{Duration, Instant};

pub mod auth;
pub mod barang;
pub mod kas;
pub mod pengguna;

/// Menjalankan `f` dengan sesi yang masih berlaku. Sesi yang menganggur terlalu lama dikunci.
pub fn dengan_sesi<T>(st: &AppState, f: impl FnOnce(&mut Connection, &Sesi) -> Hasil<T>) -> Hasil<T> {
    let mut db = kunci(&st.db);
    let sesi = periksa_sesi(st, &db, true)?;
    f(&mut db, &sesi)
}

/// Mengembalikan sesi bila masih berlaku. `sentuh` memperbarui waktu aktivitas terakhir.
pub(crate) fn periksa_sesi(st: &AppState, db: &Connection, sentuh: bool) -> Hasil<Sesi> {
    let mut sesi = kunci(&st.sesi);
    let Some(s) = sesi.clone() else {
        return Err(Galat::baru("BELUM_LOGIN", "Silakan masuk dulu."));
    };
    let batas = Duration::from_secs(pengaturan::menit_kunci(db) * 60);
    let mut terakhir = kunci(&st.terakhir_aktif);
    if terakhir.elapsed() > batas {
        *sesi = None;
        return Err(Galat::baru("TERKUNCI", "Aplikasi terkunci karena lama tidak dipakai. Silakan masuk lagi."));
    }
    if sentuh {
        *terakhir = Instant::now();
    }
    Ok(s)
}
