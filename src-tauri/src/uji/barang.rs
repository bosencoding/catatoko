use super::{db_uji, hitung, kasir, pemilik, staf};
use crate::akses::{Modul, Tingkat};
use crate::barang::{self, BarangMasukan};
use crate::foto;
use std::path::{Path, PathBuf};

pub fn masukan(nama: &str) -> BarangMasukan {
    BarangMasukan {
        kode: String::new(),
        nama: nama.into(),
        kategori_id: None,
        satuan: "karung".into(),
        harga_jual: 450_000,
        stok_minimum: 5_000,
        foto_sumber: None,
        aktif: true,
    }
}

fn data() -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    crate::jalur::siapkan(d.path()).unwrap();
    d
}

fn png(dir: &Path, lebar: u32, tinggi: u32) -> PathBuf {
    let p = dir.join("sumber.png");
    image::RgbImage::from_pixel(lebar, tinggi, image::Rgb([20, 120, 200])).save(&p).unwrap();
    p
}

fn isi_foto(d: &Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(d.join("foto")).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().into_owned()).collect();
    v.sort();
    v
}

#[test]
fn tambah_dan_daftar() {
    let (c, d) = (db_uji(), data());
    let id = barang::tambah(&c, d.path(), &pemilik(), masukan("Pakan Apung 781-2")).unwrap();
    barang::tambah(&c, d.path(), &pemilik(), masukan("Jaring Kolam")).unwrap();
    let semua = barang::daftar(&c, &pemilik(), None, false).unwrap();
    assert_eq!(semua.len(), 2);
    let cari = barang::daftar(&c, &pemilik(), Some("apung"), false).unwrap();
    assert_eq!(cari.len(), 1);
    assert_eq!(cari[0].id, id);
    assert_eq!(cari[0].stok, 0);
    assert_eq!(cari[0].nilai_stok, Some(0));
    assert_eq!(cari[0].satuan, "karung");
    assert_eq!(barang::ambil(&c, &pemilik(), id).unwrap().nama, "Pakan Apung 781-2");
    assert_eq!(barang::ambil(&c, &pemilik(), 999).unwrap_err().kode, "TIDAK_DITEMUKAN");
}

#[test]
fn stok_dan_nilai_dari_kiriman() {
    let (c, d) = (db_uji(), data());
    let id = barang::tambah(&c, d.path(), &pemilik(), masukan("Pakan")).unwrap();
    c.execute_batch(&format!(
        "INSERT INTO kiriman(barang_id,tanggal_masuk,jumlah_awal,sisa,harga_beli,dibuat_pada) VALUES({id},'2026-09-03',5000,2000,385000,'t');
         INSERT INTO kiriman(barang_id,tanggal_masuk,jumlah_awal,sisa,harga_beli,dibuat_pada) VALUES({id},'2026-10-15',10000,10000,392000,'t');
         INSERT INTO kiriman(barang_id,tanggal_masuk,jumlah_awal,sisa,harga_beli,batal,dibuat_pada) VALUES({id},'2026-10-16',10000,0,1,1,'t');"
    ))
    .unwrap();
    let b = barang::ambil(&c, &pemilik(), id).unwrap();
    assert_eq!(b.stok, 12_000);
    assert_eq!(b.nilai_stok, Some(2 * 385_000 + 10 * 392_000));
}

#[test]
fn kode_otomatis_berurut() {
    let (c, d) = (db_uji(), data());
    let a = barang::tambah(&c, d.path(), &pemilik(), masukan("A")).unwrap();
    let b = barang::tambah(&c, d.path(), &pemilik(), masukan("B")).unwrap();
    assert_eq!(barang::ambil(&c, &pemilik(), a).unwrap().kode, "B0001");
    assert_eq!(barang::ambil(&c, &pemilik(), b).unwrap().kode, "B0002");
    let mut m = masukan("C");
    m.kode = "B0003".into();
    barang::tambah(&c, d.path(), &pemilik(), m).unwrap();
    let e = barang::tambah(&c, d.path(), &pemilik(), masukan("D")).unwrap();
    assert_eq!(barang::ambil(&c, &pemilik(), e).unwrap().kode, "B0004");
}

#[test]
fn kode_ganda_ditolak() {
    let (c, d) = (db_uji(), data());
    let mut m = masukan("A");
    m.kode = "PKN-1".into();
    barang::tambah(&c, d.path(), &pemilik(), m.clone()).unwrap();
    m.kode = "pkn-1".into();
    let e = barang::tambah(&c, d.path(), &pemilik(), m).unwrap_err();
    assert_eq!(e.kode, "VALIDASI");
    assert!(e.pesan.contains("pkn-1"), "{}", e.pesan);
}

#[test]
fn isian_tak_sah_ditolak() {
    let (c, d) = (db_uji(), data());
    let p = pemilik();
    let mut m = masukan("  ");
    assert_eq!(barang::tambah(&c, d.path(), &p, m.clone()).unwrap_err().kode, "VALIDASI");
    m = masukan("A");
    m.satuan = "".into();
    assert_eq!(barang::tambah(&c, d.path(), &p, m.clone()).unwrap_err().kode, "VALIDASI");
    m = masukan("A");
    m.harga_jual = -1;
    assert_eq!(barang::tambah(&c, d.path(), &p, m.clone()).unwrap_err().kode, "VALIDASI");
    m = masukan("A");
    m.kategori_id = Some(99);
    assert_eq!(barang::tambah(&c, d.path(), &p, m).unwrap_err().kode, "VALIDASI");
    assert_eq!(hitung(&c, "SELECT count(*) FROM barang"), 0);
}

#[test]
fn hak_barang() {
    let (c, d) = (db_uji(), data());
    let id = barang::tambah(&c, d.path(), &pemilik(), masukan("A")).unwrap();
    let lihat = staf(&[(Modul::Barang, Tingkat::Lihat)], false);
    assert_eq!(barang::tambah(&c, d.path(), &lihat, masukan("B")).unwrap_err().kode, "TIDAK_BERHAK");
    assert_eq!(barang::ubah(&c, d.path(), &lihat, id, masukan("B")).unwrap_err().kode, "TIDAK_BERHAK");
    let tambah = staf(&[(Modul::Barang, Tingkat::Tambah)], false);
    assert!(barang::tambah(&c, d.path(), &tambah, masukan("B")).is_ok());
    assert_eq!(barang::ubah(&c, d.path(), &tambah, id, masukan("B")).unwrap_err().kode, "TIDAK_BERHAK");
    assert_eq!(barang::daftar(&c, &staf(&[(Modul::Kas, Tingkat::Ubah)], true), None, false).unwrap_err().kode, "TIDAK_BERHAK");
    // yang hanya boleh menjual tetap perlu memilih barang
    assert!(barang::daftar(&c, &staf(&[(Modul::BarangKeluar, Tingkat::Tambah)], false), None, false).is_ok());
}

#[test]
fn nilai_stok_tersembunyi() {
    let (c, d) = (db_uji(), data());
    barang::tambah(&c, d.path(), &pemilik(), masukan("A")).unwrap();
    let v = barang::daftar(&c, &kasir(), None, false).unwrap();
    assert_eq!(v[0].nilai_stok, None);
    assert_eq!(v[0].stok, 0);
}

#[test]
fn ubah_dan_nonaktif() {
    let (c, d) = (db_uji(), data());
    let id = barang::tambah(&c, d.path(), &pemilik(), masukan("A")).unwrap();
    let kat = barang::kategori_tambah(&c, &pemilik(), "Pakan").unwrap();
    assert_eq!(barang::kategori_tambah(&c, &pemilik(), "pakan").unwrap_err().kode, "VALIDASI");
    let mut m = masukan("A2");
    m.kode = "B0001".into();
    m.kategori_id = Some(kat);
    m.aktif = false;
    barang::ubah(&c, d.path(), &pemilik(), id, m).unwrap();
    assert!(barang::daftar(&c, &pemilik(), None, false).unwrap().is_empty());
    let semua = barang::daftar(&c, &pemilik(), None, true).unwrap();
    assert_eq!(semua[0].nama, "A2");
    assert_eq!(semua[0].kategori.as_deref(), Some("Pakan"));
    assert_eq!(barang::kategori_daftar(&c, &pemilik()).unwrap().len(), 1);
    assert_eq!(hitung(&c, "SELECT count(*) FROM log_aktivitas WHERE tindakan IN('barang.tambah','barang.ubah')"), 2);
}

#[test]
fn foto_diperkecil() {
    let d = data();
    let sumber = png(d.path(), 3000, 2000);
    let rel = foto::simpan(d.path(), &sumber).unwrap();
    assert!(rel.starts_with("foto/") && rel.ends_with(".jpg"), "{rel}");
    let besar = image::ImageReader::open(d.path().join(&rel)).unwrap().with_guessed_format().unwrap();
    assert_eq!(besar.format(), Some(image::ImageFormat::Jpeg));
    assert_eq!(besar.into_dimensions().unwrap(), (1280, 853));
    let kecil = image::ImageReader::open(d.path().join(foto::kecil(&rel))).unwrap().with_guessed_format().unwrap();
    assert_eq!(kecil.format(), Some(image::ImageFormat::Jpeg));
    assert_eq!(kecil.into_dimensions().unwrap().0, 256);
}

#[test]
fn foto_kecil_tidak_diperbesar() {
    let d = data();
    let rel = foto::simpan(d.path(), &png(d.path(), 400, 300)).unwrap();
    assert_eq!(image::image_dimensions(d.path().join(&rel)).unwrap(), (400, 300));
}

#[test]
fn foto_bukan_gambar_ditolak() {
    let (c, d) = (db_uji(), data());
    let palsu = d.path().join("palsu.jpg");
    std::fs::write(&palsu, b"ini bukan gambar").unwrap();
    assert_eq!(foto::simpan(d.path(), &palsu).unwrap_err().kode, "VALIDASI");
    assert_eq!(foto::simpan(d.path(), &d.path().join("tidak-ada.jpg")).unwrap_err().kode, "VALIDASI");
    let mut m = masukan("A");
    m.foto_sumber = Some(palsu);
    assert_eq!(barang::tambah(&c, d.path(), &pemilik(), m).unwrap_err().kode, "VALIDASI");
    assert!(isi_foto(d.path()).is_empty());
    assert_eq!(hitung(&c, "SELECT count(*) FROM barang"), 0);
}

#[test]
fn foto_rusak_ditolak_tanpa_sisa() {
    let d = data();
    let sumber = png(d.path(), 200, 200);
    let mut isi = std::fs::read(&sumber).unwrap();
    isi.truncate(isi.len() / 2);
    let rusak = d.path().join("rusak.png");
    std::fs::write(&rusak, isi).unwrap();
    assert_eq!(foto::simpan(d.path(), &rusak).unwrap_err().kode, "VALIDASI");
    assert!(isi_foto(d.path()).is_empty());
}

#[test]
fn foto_terlalu_besar_ditolak() {
    let d = data();
    let besar = d.path().join("besar.jpg");
    let f = std::fs::File::create(&besar).unwrap();
    f.set_len(26 * 1024 * 1024).unwrap();
    let e = foto::simpan(d.path(), &besar).unwrap_err();
    assert_eq!(e.kode, "VALIDASI");
    assert!(e.pesan.contains("25"), "{}", e.pesan);
}

#[test]
fn gagal_simpan_barang_menghapus_foto() {
    let (c, d) = (db_uji(), data());
    let mut m = masukan("A");
    m.kode = "X1".into();
    barang::tambah(&c, d.path(), &pemilik(), m.clone()).unwrap();
    m.foto_sumber = Some(png(d.path(), 300, 300));
    assert_eq!(barang::tambah(&c, d.path(), &pemilik(), m).unwrap_err().kode, "VALIDASI");
    assert!(isi_foto(d.path()).is_empty());
}

#[test]
fn ganti_foto_menghapus_lama() {
    let (c, d) = (db_uji(), data());
    let mut m = masukan("A");
    m.foto_sumber = Some(png(d.path(), 300, 300));
    let id = barang::tambah(&c, d.path(), &pemilik(), m.clone()).unwrap();
    let lama = barang::ambil(&c, &pemilik(), id).unwrap().foto.unwrap();
    assert_eq!(isi_foto(d.path()).len(), 2);
    m.kode = "B0001".into();
    barang::ubah(&c, d.path(), &pemilik(), id, m.clone()).unwrap();
    let baru = barang::ambil(&c, &pemilik(), id).unwrap().foto.unwrap();
    assert_ne!(lama, baru);
    assert!(!d.path().join(&lama).exists());
    assert!(!d.path().join(foto::kecil(&lama)).exists());
    assert_eq!(isi_foto(d.path()).len(), 2);
    // tanpa foto baru, foto lama dipertahankan
    m.foto_sumber = None;
    barang::ubah(&c, d.path(), &pemilik(), id, m).unwrap();
    assert_eq!(barang::ambil(&c, &pemilik(), id).unwrap().foto.unwrap(), baru);
}
