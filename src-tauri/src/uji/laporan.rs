use super::{hitung, jual, jual_tiga, kas as uji_kas, kasir, pemilik, skenario, staf, Skenario, HARI_INI};
use crate::akses::{Modul, Tingkat};
use crate::laporan::{self, Jenis, Laporan, Sel};
use crate::ringkasan::{self, Butir};
use crate::{ekspor, kas, stok};

const DARI: &str = "2026-09-01";
const SAMPAI: &str = "2026-11-30";

/// Skenario + jual 3 karung + listrik 250.000 + 1 karung rusak dari kiriman Okt
/// + satu penjualan yang dibatalkan + satu pindah akun (keduanya tak boleh muncul di laporan).
fn benih() -> Skenario {
    let s = skenario();
    let p = pemilik();
    jual_tiga(&s);
    kas::tambah_manual(&s.c, &p, uji_kas::masukan(&s.c, "2026-11-10", "keluar", "Listrik/Air", 250_000)).unwrap();
    stok::sesuaikan(&s.c, &p, s.okt, HARI_INI, -1_000, "rusak", None).unwrap();
    let batal = stok::keluar(&s.c, &p, jual(s.barang, 500_000, &[(s.nov, 1_000)]), HARI_INI).unwrap();
    stok::batal_keluar(&s.c, &p, batal, "salah input").unwrap();
    let bank = kas::akun_tambah(&s.c, &p, "Bank").unwrap();
    kas::pindah(&s.c, &p, HARI_INI, 1, bank, 100_000, None).unwrap();
    s
}

fn ringkas(l: &Laporan, label: &str) -> Sel {
    l.ringkasan.iter().find(|(k, _)| k == label).unwrap_or_else(|| panic!("ringkasan '{label}' tidak ada: {:?}", l.ringkasan)).1.clone()
}

fn kolom(l: &Laporan, nama: &str) -> usize {
    l.kolom.iter().position(|k| k == nama).unwrap_or_else(|| panic!("kolom '{nama}' tidak ada: {:?}", l.kolom))
}

fn jumlah_kolom(l: &Laporan, nama: &str) -> i64 {
    let i = kolom(l, nama);
    l.baris.iter().map(|b| match &b[i] { Sel::Rupiah(n) => *n, _ => 0 }).sum()
}

#[test]
fn laba_kotor_cocok() {
    let s = benih();
    let l = laporan::buat(&s.c, &pemilik(), Jenis::LabaKotor, DARI, SAMPAI).unwrap();
    assert_eq!(l.nama_toko, "Toko Uji");
    assert_eq!(l.baris.len(), 2, "penjualan batal tidak ikut");
    assert_eq!(ringkas(&l, "Penjualan"), Sel::Rupiah(1_350_000));
    assert_eq!(ringkas(&l, "Harga pokok"), Sel::Rupiah(1_162_000));
    assert_eq!(ringkas(&l, "Laba kotor"), Sel::Rupiah(188_000));
    assert_eq!(ringkas(&l, "Kerugian stok"), Sel::Rupiah(392_000));
    assert_eq!(jumlah_kolom(&l, "Laba"), 188_000);
    let total = l.total.as_ref().unwrap();
    assert_eq!(total.len(), l.kolom.len());
    assert_eq!(total[kolom(&l, "Laba")], Sel::Rupiah(188_000));
    // di luar periode: kosong
    let kosong = laporan::buat(&s.c, &pemilik(), Jenis::LabaKotor, "2026-01-01", "2026-01-31").unwrap();
    assert!(kosong.baris.is_empty());
    assert_eq!(ringkas(&kosong, "Laba kotor"), Sel::Rupiah(0));
}

#[test]
fn arus_kas_total() {
    let s = benih();
    let l = laporan::buat(&s.c, &pemilik(), Jenis::ArusKas, DARI, SAMPAI).unwrap();
    assert_eq!(l.baris.len(), 2, "hanya penjualan dan listrik; batal dan pindah akun tidak ikut");
    let total = l.total.as_ref().unwrap();
    assert_eq!(total[kolom(&l, "Masuk")], Sel::Rupiah(1_350_000));
    assert_eq!(total[kolom(&l, "Keluar")], Sel::Rupiah(250_000));
    assert_eq!(jumlah_kolom(&l, "Masuk"), 1_350_000);
    assert_eq!(jumlah_kolom(&l, "Keluar"), 250_000);
    assert_eq!(ringkas(&l, "Saldo awal"), Sel::Rupiah(0));
    assert_eq!(ringkas(&l, "Saldo akhir"), Sel::Rupiah(1_100_000));
    assert_eq!(ringkas(&l, "Masuk: Penjualan"), Sel::Rupiah(1_350_000));
    assert_eq!(ringkas(&l, "Keluar: Listrik/Air"), Sel::Rupiah(250_000));
}

#[test]
fn posisi_stok_nilai() {
    let s = benih();
    let l = laporan::buat(&s.c, &pemilik(), Jenis::PosisiStok, DARI, SAMPAI).unwrap();
    // Sep habis; Okt sisa 8 (10 − 1 terjual − 1 rusak); Nov 20
    assert_eq!(l.baris.len(), 2);
    let nilai = 8 * 392_000 + 20 * 398_000;
    assert_eq!(l.total.as_ref().unwrap()[kolom(&l, "Nilai")], Sel::Rupiah(nilai));
    assert_eq!(l.baris[0][kolom(&l, "Sisa")], Sel::Jumlah(8_000));
    assert_eq!(hitung(&s.c, "SELECT SUM((sisa*harga_beli+500)/1000) FROM kiriman WHERE batal=0"), nilai);
}

#[test]
fn kartu_stok_urut_dan_saldo() {
    let s = benih();
    let l = laporan::buat(&s.c, &pemilik(), Jenis::KartuStok { barang_id: s.barang }, DARI, SAMPAI).unwrap();
    assert!(l.judul.contains("Pakan Apung 781-2"), "{}", l.judul);
    let (it, im, ik, is) = (kolom(&l, "Tanggal"), kolom(&l, "Masuk"), kolom(&l, "Keluar"), kolom(&l, "Saldo"));
    // 3 masuk + 2 baris keluar + 1 penyesuaian
    assert_eq!(l.baris.len(), 6);
    let tgl: Vec<&Sel> = l.baris.iter().map(|b| &b[it]).collect();
    let mut urut = tgl.clone();
    urut.sort_by_key(|t| match t { Sel::Tanggal(x) => x.clone(), _ => String::new() });
    assert_eq!(tgl, urut);
    assert_eq!(l.baris[0][im], Sel::Jumlah(2_000));
    assert_eq!(l.baris[0][is], Sel::Jumlah(2_000));
    assert_eq!(l.baris[2][is], Sel::Jumlah(32_000));
    let keluar: i64 = l.baris.iter().map(|b| match &b[ik] { Sel::Jumlah(n) => *n, _ => 0 }).sum();
    assert_eq!(keluar, 4_000);
    assert_eq!(l.baris.last().unwrap()[is], Sel::Jumlah(28_000));
    assert_eq!(crate::barang::ambil(&s.c, &pemilik(), s.barang).unwrap().stok, 28_000);
    // periode sempit membawa saldo awal
    let nov = laporan::buat(&s.c, &pemilik(), Jenis::KartuStok { barang_id: s.barang }, "2026-11-21", SAMPAI).unwrap();
    assert_eq!(ringkas(&nov, "Saldo awal"), Sel::Jumlah(32_000));
    assert_eq!(nov.baris.len(), 3);
    assert_eq!(laporan::buat(&s.c, &pemilik(), Jenis::KartuStok { barang_id: 99 }, DARI, SAMPAI).unwrap_err().kode, "TIDAK_DITEMUKAN");
}

#[test]
fn barang_masuk_dan_keluar() {
    let s = benih();
    let m = laporan::buat(&s.c, &pemilik(), Jenis::BarangMasuk, DARI, SAMPAI).unwrap();
    assert_eq!(m.baris.len(), 3);
    assert_eq!(m.total.as_ref().unwrap()[kolom(&m, "Total")], Sel::Rupiah(2 * 385_000 + 10 * 392_000 + 20 * 398_000));
    let k = laporan::buat(&s.c, &pemilik(), Jenis::BarangKeluar, DARI, SAMPAI).unwrap();
    assert_eq!(k.baris.len(), 2);
    assert_eq!(k.total.as_ref().unwrap()[kolom(&k, "Subtotal")], Sel::Rupiah(1_350_000));
    assert_eq!(k.total.as_ref().unwrap()[kolom(&k, "Laba")], Sel::Rupiah(188_000));
}

#[test]
fn laporan_butuh_hak() {
    let s = benih();
    for j in [Jenis::ArusKas, Jenis::BarangKeluar, Jenis::LabaKotor] {
        assert_eq!(laporan::buat(&s.c, &kasir(), j, DARI, SAMPAI).unwrap_err().kode, "TIDAK_BERHAK");
    }
    let tanpa_harga = staf(&[(Modul::Laporan, Tingkat::Lihat)], false);
    for j in [Jenis::LabaKotor, Jenis::PosisiStok, Jenis::BarangMasuk] {
        assert_eq!(laporan::buat(&s.c, &tanpa_harga, j, DARI, SAMPAI).unwrap_err().kode, "TIDAK_BERHAK");
    }
    let k = laporan::buat(&s.c, &tanpa_harga, Jenis::BarangKeluar, DARI, SAMPAI).unwrap();
    assert!(k.kolom.iter().all(|c| c != "Harga beli" && c != "Laba"), "{:?}", k.kolom);
    assert_eq!(k.total.as_ref().unwrap().len(), k.kolom.len());
    assert!(!serde_json::to_string(&k).unwrap().contains("385000"));
    assert!(laporan::buat(&s.c, &tanpa_harga, Jenis::KartuStok { barang_id: s.barang }, DARI, SAMPAI).is_ok());
    assert_eq!(laporan::buat(&s.c, &pemilik(), Jenis::ArusKas, SAMPAI, DARI).unwrap_err().kode, "VALIDASI");
}

#[test]
fn ringkasan_pemilik() {
    let s = benih();
    let r = ringkasan::ringkasan(&s.c, &pemilik(), "2026-11-01", SAMPAI, Butir::Hari, HARI_INI).unwrap();
    let k = r.kas.as_ref().unwrap();
    assert_eq!((k.saldo_total, k.masuk, k.keluar), (1_100_000, 1_350_000, 250_000));
    assert_eq!(k.saldo_per_akun.len(), 2);
    assert_eq!(r.nilai_stok, Some(8 * 392_000 + 20 * 398_000));
    assert_eq!(r.laba, Some(188_000));
    let dk = r.deret_kas.as_ref().unwrap();
    assert_eq!(dk.len(), 30);
    assert_eq!(dk[9].label, "10 Nov");
    assert_eq!((dk[9].masuk, dk[9].keluar, dk[9].saldo_akhir), (0, 250_000, -250_000));
    assert_eq!((dk[23].masuk, dk[23].saldo_akhir), (1_350_000, 1_100_000));
    assert_eq!(dk[29].saldo_akhir, 1_100_000);
    assert_eq!(r.deret_barang.len(), 30);
    assert_eq!((r.deret_barang[19].masuk, r.deret_barang[23].keluar), (20_000, 3_000));
    assert_eq!(r.terlaris, vec![("Pakan Apung 781-2".to_string(), 3_000)]);
}

#[test]
fn ringkasan_menyaring_hak() {
    let s = benih();
    let r = ringkasan::ringkasan(&s.c, &kasir(), "2026-11-01", SAMPAI, Butir::Hari, HARI_INI).unwrap();
    assert!(r.kas.is_none() && r.nilai_stok.is_none() && r.laba.is_none() && r.deret_kas.is_none());
    assert_eq!(r.deret_barang.len(), 30);
    assert!(r.stok_lama.iter().all(|k| k.harga_beli.is_none()));
    let j = serde_json::to_string(&r).unwrap();
    assert!(!j.contains("392000") && !j.contains("398000"), "{j}");
    // tanpa hak barang sama sekali: tidak ada data barang
    let hanya_kas = staf(&[(Modul::Kas, Tingkat::Lihat)], false);
    let r = ringkasan::ringkasan(&s.c, &hanya_kas, "2026-11-01", SAMPAI, Butir::Hari, HARI_INI).unwrap();
    assert!(r.kas.is_some() && r.nilai_stok.is_none() && r.laba.is_none());
    assert!(r.deret_barang.is_empty() && r.stok_lama.is_empty() && r.terlaris.is_empty() && r.stok_menipis.is_empty());
}

#[test]
fn ringkasan_deret_bulanan() {
    let s = benih();
    let r = ringkasan::ringkasan(&s.c, &pemilik(), DARI, SAMPAI, Butir::Bulan, HARI_INI).unwrap();
    let dk = r.deret_kas.unwrap();
    assert_eq!(dk.iter().map(|t| t.label.as_str()).collect::<Vec<_>>(), vec!["Sep 2026", "Okt 2026", "Nov 2026"]);
    assert_eq!(dk[2].saldo_akhir, kas::saldo(&s.c, &pemilik()).unwrap().total);
    assert_eq!(r.deret_barang.iter().map(|t| t.masuk).collect::<Vec<_>>(), vec![2_000, 10_000, 20_000]);
    let minggu = ringkasan::ringkasan(&s.c, &pemilik(), "2026-11-01", SAMPAI, Butir::Minggu, HARI_INI).unwrap();
    assert_eq!(minggu.deret_barang.len(), 5);
    assert_eq!(minggu.deret_barang[0].label, "01 Nov");
    assert_eq!(ringkasan::ringkasan(&s.c, &pemilik(), "2020-01-01", SAMPAI, Butir::Hari, HARI_INI).unwrap_err().kode, "VALIDASI");
}

#[test]
fn daftar_perhatian() {
    let s = benih();
    // stok 28 karung, minimum 5 → tidak menipis
    let r = ringkasan::ringkasan(&s.c, &pemilik(), DARI, SAMPAI, Butir::Bulan, HARI_INI).unwrap();
    assert!(r.stok_menipis.is_empty());
    assert_eq!(r.stok_lama[0].id, s.okt, "kiriman Sep sudah habis, jadi Okt yang tertua");
    assert!(r.hampir_kedaluwarsa.is_empty());
    s.c.execute("UPDATE barang SET stok_minimum=30000", []).unwrap();
    s.c.execute("UPDATE kiriman SET kedaluwarsa='2026-12-20' WHERE id=?1", [s.nov]).unwrap();
    s.c.execute("UPDATE kiriman SET kedaluwarsa='2026-11-01' WHERE id=?1", [s.okt]).unwrap();
    let r = ringkasan::ringkasan(&s.c, &pemilik(), DARI, SAMPAI, Butir::Bulan, HARI_INI).unwrap();
    assert_eq!(r.stok_menipis.len(), 1);
    assert_eq!(r.hampir_kedaluwarsa.iter().map(|k| k.id).collect::<Vec<_>>(), vec![s.okt, s.nov]);
    assert!(r.hampir_kedaluwarsa[0].sudah_kedaluwarsa);
}

fn baca_xlsx(path: &std::path::Path) -> Vec<Vec<calamine::Data>> {
    use calamine::Reader;
    let mut wb: calamine::Xlsx<_> = calamine::open_workbook(path).unwrap();
    let nama = wb.sheet_names()[0].clone();
    wb.worksheet_range(&nama).unwrap().rows().map(|r| r.to_vec()).collect()
}

fn angka(d: &calamine::Data) -> Option<i64> {
    match d {
        calamine::Data::Float(f) => Some(f.round() as i64),
        calamine::Data::Int(i) => Some(*i),
        _ => None,
    }
}

#[test]
fn xlsx_terbentuk() {
    let s = benih();
    let l = laporan::buat(&s.c, &pemilik(), Jenis::LabaKotor, DARI, SAMPAI).unwrap();
    let f = s.d.path().join("laba.xlsx");
    ekspor::xlsx(&l, &f).unwrap();
    assert_eq!(&std::fs::read(&f).unwrap()[..2], b"PK");
    let baris = baca_xlsx(&f);
    let teks = |d: &calamine::Data| d.to_string();
    assert!(baris.iter().any(|r| teks(&r[0]).contains("Laba Kotor")));
    assert!(baris.iter().any(|r| teks(&r[0]) == "Toko Uji"));
    let kepala = baris.iter().position(|r| teks(&r[0]) == l.kolom[0]).expect("baris kepala");
    assert_eq!(baris[kepala].iter().map(teks).filter(|t| !t.is_empty()).collect::<Vec<_>>(), l.kolom);
    let total = baris.iter().find(|r| teks(&r[0]) == "Total").expect("baris total");
    assert_eq!(angka(&total[kolom(&l, "Laba")]), Some(188_000));
    assert_eq!(angka(&total[kolom(&l, "Penjualan")]), Some(1_350_000));
    let rugi = baris.iter().find(|r| teks(&r[0]) == "Kerugian stok").expect("ringkasan");
    assert_eq!(angka(&rugi[1]), Some(392_000));
    // jumlah pecahan tetap pecahan
    let kartu = laporan::buat(&s.c, &pemilik(), Jenis::KartuStok { barang_id: s.barang }, DARI, SAMPAI).unwrap();
    let f2 = s.d.path().join("kartu.xlsx");
    ekspor::xlsx(&kartu, &f2).unwrap();
    assert!(baca_xlsx(&f2).iter().any(|r| r.iter().any(|d| matches!(d, calamine::Data::Float(x) if (*x - 28.0).abs() < 1e-9))));
}

fn halaman_pdf(path: &std::path::Path) -> usize {
    let isi = std::fs::read(path).unwrap();
    assert_eq!(&isi[..4], b"%PDF");
    let teks = String::from_utf8_lossy(&isi).replace(char::is_whitespace, "");
    teks.matches("/Type/Page").count() - teks.matches("/Type/Pages").count()
}

#[test]
fn pdf_terbentuk() {
    let s = benih();
    let mut l = laporan::buat(&s.c, &pemilik(), Jenis::LabaKotor, DARI, SAMPAI).unwrap();
    let f = s.d.path().join("laba.pdf");
    ekspor::pdf(&l, &f).unwrap();
    assert_eq!(halaman_pdf(&f), 1);
    let contoh = l.baris[0].clone();
    l.baris = (0..200).map(|_| contoh.clone()).collect();
    ekspor::pdf(&l, &f).unwrap();
    assert!(halaman_pdf(&f) >= 2, "200 baris harus lebih dari satu halaman");
}

#[test]
fn laporan_kosong_dan_teks_aneh_tetap_terekspor() {
    let s = skenario();
    let p = pemilik();
    for (i, j) in [Jenis::ArusKas, Jenis::PosisiStok, Jenis::KartuStok { barang_id: s.barang }, Jenis::BarangMasuk, Jenis::BarangKeluar, Jenis::LabaKotor].into_iter().enumerate() {
        let l = laporan::buat(&s.c, &p, j, "2020-01-01", "2020-01-31").unwrap();
        ekspor::xlsx(&l, &s.d.path().join(format!("k{i}.xlsx"))).unwrap();
        ekspor::pdf(&l, &s.d.path().join(format!("k{i}.pdf"))).unwrap();
    }
    let mut l = laporan::buat(&s.c, &p, Jenis::PosisiStok, DARI, SAMPAI).unwrap();
    l.baris[0][0] = Sel::Teks("Pakan “super” → 鱼 🐟 ".repeat(30));
    l.nama_toko = "Toko 鱼".into();
    ekspor::pdf(&l, &s.d.path().join("aneh.pdf")).unwrap();
    ekspor::xlsx(&l, &s.d.path().join("aneh.xlsx")).unwrap();
    assert_eq!(ekspor::pdf(&l, &s.d.path().join("tidak/ada/x.pdf")).unwrap_err().kode, "VALIDASI");
    assert_eq!(ekspor::xlsx(&l, &s.d.path().join("tidak/ada/x.xlsx")).unwrap_err().kode, "VALIDASI");
}

#[test]
fn pdf_tidak_memotong_kepala_dan_total() {
    let s = benih();
    let l = laporan::buat(&s.c, &pemilik(), Jenis::LabaKotor, DARI, SAMPAI).unwrap();
    let f = s.d.path().join("laba.pdf");
    ekspor::pdf(&l, &f).unwrap();
    let isi = String::from_utf8_lossy(&std::fs::read(&f).unwrap()).into_owned();
    // printpdf menulis tiap teks sebagai satu string heksadesimal: <48617267...> Tj
    let heks = |t: &str| format!("<{}>", t.bytes().map(|b| format!("{b:02X}")).collect::<String>());
    for utuh in ["Harga beli", "Harga pokok", "188.000", "1.350.000", "Pakan Apung 781-2"] {
        assert!(isi.contains(&heks(utuh)), "'{utuh}' terpotong atau hilang di PDF");
    }
    assert!(!isi.contains(&heks("Harga b..")));
}

#[test]
fn keterangan_kas_penjualan_memakai_format_rupiah() {
    let s = skenario();
    jual_tiga(&s);
    let ket: String = s.c.query_row("SELECT keterangan FROM kas WHERE ref_jenis='transaksi_keluar'", [], |r| r.get(0)).unwrap();
    assert_eq!(ket, "Jual 3 karung Pakan Apung 781-2 @ Rp 450.000");
}
