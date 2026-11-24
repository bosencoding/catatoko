//! Sesi login dan pemeriksaan hak. Setiap fungsi inti memanggil `wajib*` lebih dulu.

use crate::galat::{Galat, Hasil};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Modul {
    Barang,
    BarangMasuk,
    BarangKeluar,
    Kas,
    Laporan,
}

impl Modul {
    pub const SEMUA: [Modul; 5] = [Modul::Barang, Modul::BarangMasuk, Modul::BarangKeluar, Modul::Kas, Modul::Laporan];

    pub fn kunci(self) -> &'static str {
        match self {
            Modul::Barang => "barang",
            Modul::BarangMasuk => "barang_masuk",
            Modul::BarangKeluar => "barang_keluar",
            Modul::Kas => "kas",
            Modul::Laporan => "laporan",
        }
    }

    pub fn dari_kunci(k: &str) -> Option<Modul> {
        Self::SEMUA.into_iter().find(|m| m.kunci() == k)
    }

    fn label(self) -> &'static str {
        match self {
            Modul::Barang => "Barang",
            Modul::BarangMasuk => "Barang Masuk",
            Modul::BarangKeluar => "Barang Keluar",
            Modul::Kas => "Kas",
            Modul::Laporan => "Laporan",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum Tingkat {
    TidakAda = 0,
    Lihat = 1,
    Tambah = 2,
    Ubah = 3,
}

impl Tingkat {
    pub fn dari_angka(n: i64) -> Tingkat {
        match n {
            1 => Tingkat::Lihat,
            2 => Tingkat::Tambah,
            3 => Tingkat::Ubah,
            _ => Tingkat::TidakAda,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Sesi {
    pub pengguna_id: i64,
    pub nama_pengguna: String,
    pub nama: String,
    pub pemilik: bool,
    pub lihat_harga_beli: bool,
    pub hak: HashMap<Modul, Tingkat>,
}

impl Sesi {
    pub fn boleh(&self, m: Modul, t: Tingkat) -> bool {
        self.pemilik || self.hak.get(&m).copied().unwrap_or(Tingkat::TidakAda) >= t
    }

    pub fn boleh_harga_beli(&self) -> bool {
        self.pemilik || self.lihat_harga_beli
    }

    #[cfg(test)]
    pub fn pemilik_uji() -> Sesi {
        Sesi { pengguna_id: 1, nama_pengguna: "pemilik".into(), nama: "Pemilik Uji".into(), pemilik: true, lihat_harga_beli: true, hak: HashMap::new() }
    }

    #[cfg(test)]
    pub fn staf_uji(hak: &[(Modul, Tingkat)], harga: bool) -> Sesi {
        Sesi { pengguna_id: 2, nama_pengguna: "staf".into(), nama: "Staf Uji".into(), pemilik: false, lihat_harga_beli: harga, hak: hak.iter().copied().collect() }
    }
}

fn tolak(apa: &str) -> Galat {
    Galat::baru("TIDAK_BERHAK", format!("Akun Anda tidak punya izin untuk {apa}. Minta pemilik toko mengatur hak akses."))
}

pub fn wajib(s: &Sesi, m: Modul, t: Tingkat) -> Hasil<()> {
    if s.boleh(m, t) {
        return Ok(());
    }
    let kerja = match t {
        Tingkat::Ubah => "mengubah atau membatalkan data",
        Tingkat::Tambah => "menambah data",
        _ => "melihat",
    };
    Err(tolak(&format!("{kerja} {}", m.label())))
}

pub fn wajib_pemilik(s: &Sesi) -> Hasil<()> {
    if s.pemilik { Ok(()) } else { Err(tolak("membuka bagian khusus pemilik")) }
}

pub fn wajib_harga_beli(s: &Sesi) -> Hasil<()> {
    if s.boleh_harga_beli() { Ok(()) } else { Err(tolak("melihat harga beli dan laba")) }
}
