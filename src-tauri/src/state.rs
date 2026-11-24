//! State bersama aplikasi: koneksi DB, sesi login, waktu aktivitas terakhir.

use crate::akses::Sesi;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use std::time::Instant;

pub struct AppState {
    pub db: Mutex<Connection>,
    pub sesi: Mutex<Option<Sesi>>,
    pub terakhir_aktif: Mutex<Instant>,
    pub data: PathBuf,
}

impl AppState {
    pub fn baru(conn: Connection, data: PathBuf) -> Self {
        AppState { db: Mutex::new(conn), sesi: Mutex::new(None), terakhir_aktif: Mutex::new(Instant::now()), data }
    }
}

/// Mengunci mutex tanpa ikut panik bila pemegang sebelumnya panik.
pub fn kunci<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}
