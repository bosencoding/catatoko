//! Bentuk galat tunggal yang dikirim ke tampilan: `{ kode, pesan }`.

#[derive(Debug, Clone, thiserror::Error, serde::Serialize)]
#[error("{pesan}")]
pub struct Galat {
    pub kode: &'static str,
    pub pesan: String,
}

pub type Hasil<T> = Result<T, Galat>;

impl Galat {
    pub fn baru(kode: &'static str, pesan: impl Into<String>) -> Self {
        Galat { kode, pesan: pesan.into() }
    }
    pub fn validasi(pesan: impl Into<String>) -> Self {
        Self::baru("VALIDASI", pesan)
    }
    pub fn tidak_ditemukan(apa: &str) -> Self {
        Self::baru("TIDAK_DITEMUKAN", format!("{apa} tidak ditemukan."))
    }
    pub fn internal(pesan: impl std::fmt::Display) -> Self {
        Self::baru("INTERNAL", format!("Terjadi kesalahan pada aplikasi: {pesan}"))
    }
}

impl From<rusqlite::Error> for Galat {
    fn from(e: rusqlite::Error) -> Self {
        match e {
            rusqlite::Error::QueryReturnedNoRows => Galat::tidak_ditemukan("Data"),
            e => Galat::internal(e),
        }
    }
}

impl From<std::io::Error> for Galat {
    fn from(e: std::io::Error) -> Self {
        Galat::internal(e)
    }
}
