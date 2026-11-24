//! Waktu lokal sebagai teks, dan validasi tanggal `YYYY-MM-DD`.

use crate::galat::{Galat, Hasil};
use chrono::{Local, NaiveDate};

pub fn kini() -> String {
    Local::now().format("%Y-%m-%dT%H:%M:%S").to_string()
}

pub fn hari_ini() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

pub fn tanggal(teks: &str) -> Hasil<NaiveDate> {
    if teks.len() != 10 {
        return Err(Galat::validasi(format!("Tanggal tidak sah: {teks}")));
    }
    NaiveDate::parse_from_str(teks, "%Y-%m-%d").map_err(|_| Galat::validasi(format!("Tanggal tidak sah: {teks}")))
}

/// Memastikan `dari <= sampai` dan keduanya tanggal sah.
pub fn rentang(dari: &str, sampai: &str) -> Hasil<(NaiveDate, NaiveDate)> {
    let (a, b) = (tanggal(dari)?, tanggal(sampai)?);
    if a > b {
        return Err(Galat::validasi("Tanggal awal tidak boleh setelah tanggal akhir."));
    }
    Ok((a, b))
}
