//! Foto barang/kiriman: disalin ke `data/foto/`, diperkecil, disimpan sebagai JPEG.

use crate::galat::{Galat, Hasil};
use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, ImageReader};
use std::path::Path;

const SISI_MAKS: u32 = 1280;
const SISI_KECIL: u32 = 256;
const UKURAN_MAKS: u64 = 25 * 1024 * 1024;

/// Nama file versi kecil: `foto/x.jpg` → `foto/x_k.jpg`.
pub fn kecil(relatif: &str) -> String {
    match relatif.strip_suffix(".jpg") {
        Some(dasar) => format!("{dasar}_k.jpg"),
        None => relatif.to_string(),
    }
}

fn tulis_jpeg(gambar: &DynamicImage, sisi: u32, tujuan: &Path) -> Hasil<()> {
    let hasil = if gambar.width() > sisi || gambar.height() > sisi { gambar.thumbnail(sisi, sisi) } else { gambar.clone() };
    let mut file = std::io::BufWriter::new(std::fs::File::create(tujuan)?);
    JpegEncoder::new_with_quality(&mut file, 85).encode_image(&hasil.to_rgb8()).map_err(Galat::internal)
}

/// Menyimpan foto dan versi kecilnya. Mengembalikan jalur relatif terhadap folder data.
pub fn simpan(data: &Path, sumber: &Path) -> Hasil<String> {
    let bukan_gambar = || Galat::validasi("File foto tidak bisa dibaca. Pilih file gambar JPG, PNG, atau WebP.");
    let meta = std::fs::metadata(sumber).map_err(|_| bukan_gambar())?;
    if meta.len() > UKURAN_MAKS {
        return Err(Galat::validasi("Ukuran foto terlalu besar. Maksimal 25 MB."));
    }
    let gambar = ImageReader::open(sumber)
        .and_then(|r| r.with_guessed_format())
        .map_err(|_| bukan_gambar())?
        .decode()
        .map_err(|_| bukan_gambar())?;
    let relatif = format!("foto/{}.jpg", uuid::Uuid::new_v4().simple());
    let hasil = tulis_jpeg(&gambar, SISI_MAKS, &data.join(&relatif)).and_then(|_| tulis_jpeg(&gambar, SISI_KECIL, &data.join(kecil(&relatif))));
    if let Err(e) = hasil {
        hapus(data, &relatif);
        return Err(e);
    }
    Ok(relatif)
}

/// Menghapus foto dan versi kecilnya. Hanya menyentuh file di dalam `foto/`.
pub fn hapus(data: &Path, relatif: &str) {
    if !relatif.starts_with("foto/") || relatif.contains("..") {
        return;
    }
    let _ = std::fs::remove_file(data.join(relatif));
    let _ = std::fs::remove_file(data.join(kecil(relatif)));
}
