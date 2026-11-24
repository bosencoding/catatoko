//! Mengubah `Laporan` menjadi file XLSX atau PDF.

use crate::galat::{Galat, Hasil};
use crate::laporan::{tanggal_panjang, Laporan, Sel};
use crate::stok::teks_jumlah;
use crate::waktu;
use std::path::Path;

fn tak_bisa_disimpan(e: impl std::fmt::Display) -> Galat {
    Galat::validasi(format!("File tidak bisa disimpan di lokasi itu. Pilih folder lain. ({e})"))
}

fn rupiah(n: i64) -> String {
    let s = n.abs().to_string();
    let mut hasil = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            hasil.push('.');
        }
        hasil.push(ch);
    }
    if n < 0 { format!("-{hasil}") } else { hasil }
}

// ---------------------------------------------------------------------------
// XLSX
// ---------------------------------------------------------------------------

pub fn xlsx(l: &Laporan, tujuan: &Path) -> Hasil<()> {
    use rust_xlsxwriter::{ExcelDateTime, Format, FormatAlign, FormatBorder, Workbook, XlsxError};
    let judul = Format::new().set_bold().set_font_size(14);
    let tebal = Format::new().set_bold();
    let kepala = Format::new().set_bold().set_border_bottom(FormatBorder::Thin).set_background_color(0xE5E7EB);
    let kepala_kanan = kepala.clone().set_align(FormatAlign::Right);
    let uang = Format::new().set_num_format("#,##0");
    let banyak = Format::new().set_num_format("#,##0.###");
    let tgl = Format::new().set_num_format("dd/mm/yyyy").set_align(FormatAlign::Left);

    let hasil: Result<(), XlsxError> = (|| {
        let mut wb = Workbook::new();
        let ws = wb.add_worksheet();
        ws.set_name("Laporan")?;
        ws.write_string_with_format(0, 0, &l.judul, &judul)?;
        ws.write_string(1, 0, &l.nama_toko)?;
        ws.write_string(2, 0, &l.periode)?;

        let angka = |s: &Sel| matches!(s, Sel::Rupiah(_) | Sel::Jumlah(_));
        let mut lebar: Vec<usize> = l.kolom.iter().map(|k| k.chars().count()).collect();
        let tulis = |ws: &mut rust_xlsxwriter::Worksheet, baris: u32, kolom: u16, sel: &Sel, tebal_juga: bool| -> Result<usize, XlsxError> {
            let pakai = |f: &Format| if tebal_juga { f.clone().set_bold() } else { f.clone() };
            Ok(match sel {
                Sel::Kosong => 0,
                Sel::Teks(t) => {
                    ws.write_string_with_format(baris, kolom, t, &pakai(&Format::new()))?;
                    t.chars().count()
                }
                Sel::Rupiah(n) => {
                    ws.write_number_with_format(baris, kolom, *n as f64, &pakai(&uang))?;
                    rupiah(*n).len()
                }
                Sel::Jumlah(n) => {
                    ws.write_number_with_format(baris, kolom, *n as f64 / 1000.0, &pakai(&banyak))?;
                    teks_jumlah(*n).len() + 1
                }
                Sel::Tanggal(t) => {
                    match ExcelDateTime::parse_from_str(t) {
                        Ok(d) => ws.write_datetime_with_format(baris, kolom, &d, &pakai(&tgl))?,
                        Err(_) => ws.write_string(baris, kolom, t)?,
                    };
                    10
                }
            })
        };

        let baris_kepala = 4u32;
        for (i, nama) in l.kolom.iter().enumerate() {
            let kanan = l.baris.first().and_then(|b| b.get(i)).or(l.total.as_ref().and_then(|t| t.get(i))).is_some_and(angka);
            ws.write_string_with_format(baris_kepala, i as u16, nama, if kanan { &kepala_kanan } else { &kepala })?;
        }
        ws.set_freeze_panes(baris_kepala + 1, 0)?;
        let mut baris = baris_kepala + 1;
        for isi in &l.baris {
            for (i, sel) in isi.iter().enumerate() {
                let n = tulis(ws, baris, i as u16, sel, false)?;
                if let Some(w) = lebar.get_mut(i) {
                    *w = (*w).max(n);
                }
            }
            baris += 1;
        }
        if let Some(total) = &l.total {
            for (i, sel) in total.iter().enumerate() {
                let n = tulis(ws, baris, i as u16, sel, true)?;
                if let Some(w) = lebar.get_mut(i) {
                    *w = (*w).max(n);
                }
            }
            baris += 1;
        }
        baris += 1;
        for (label, sel) in &l.ringkasan {
            ws.write_string_with_format(baris, 0, label, &tebal)?;
            tulis(ws, baris, 1, sel, false)?;
            if let Some(w) = lebar.get_mut(0) {
                *w = (*w).max(label.chars().count());
            }
            baris += 1;
        }
        for (i, w) in lebar.iter().enumerate() {
            ws.set_column_width(i as u16, (*w as f64 + 3.0).clamp(8.0, 50.0))?;
        }
        if l.kolom.len() > 6 {
            ws.set_landscape();
        }
        wb.save(tujuan)
    })();
    hasil.map_err(|e| match e {
        XlsxError::IoError(e) => tak_bisa_disimpan(e),
        e => Galat::internal(e),
    })
}

// ---------------------------------------------------------------------------
// PDF
// ---------------------------------------------------------------------------

/// Font bawaan PDF hanya mengenal Latin-1; huruf lain diganti supaya tidak jadi sampah.
fn latin(teks: &str) -> String {
    teks.chars()
        .flat_map(|c| match c {
            '→' => vec!['-', '>'],
            '“' | '”' => vec!['"'],
            '‘' | '’' => vec!['\''],
            '–' | '—' => vec!['-'],
            '·' | '•' => vec!['-'],
            '…' => vec!['.', '.', '.'],
            c if (c as u32) < 0x20 => vec![' '],
            c if (c as u32) < 0x7f || (0xa1..=0xff).contains(&(c as u32)) => vec![c],
            _ => vec!['?'],
        })
        .collect()
}

/// Lebar huruf Helvetica dalam seperseribu em (perkiraan yang cukup untuk rata kanan angka).
fn lebar_huruf(c: char, tebal: bool) -> f32 {
    let w = match c {
        '0'..='9' => 556.0,
        '.' | ',' | ':' | ';' | ' ' | '/' | '!' | '\'' | '|' | 'i' | 'j' | 'l' | 'I' => 278.0,
        '-' | '(' | ')' | 'f' | 't' | 'r' => 333.0,
        'm' | 'M' | 'W' => 833.0,
        'w' => 722.0,
        'A'..='Z' => 690.0,
        's' | 'c' | 'k' | 'v' | 'x' | 'y' | 'z' => 500.0,
        _ => 556.0,
    };
    // Perkiraan sengaja dilebihkan sedikit: kelebihan hanya menambah jarak, kekurangan memotong teks.
    if tebal { w * 1.16 } else { w * 1.04 }
}

const PT_KE_MM: f32 = 0.352_778;

fn lebar_teks(teks: &str, ukuran: f32, tebal: bool) -> f32 {
    teks.chars().map(|c| lebar_huruf(c, tebal)).sum::<f32>() / 1000.0 * ukuran * PT_KE_MM
}

fn potong(teks: &str, ukuran: f32, tebal: bool, maks: f32) -> String {
    if lebar_teks(teks, ukuran, tebal) <= maks {
        return teks.to_string();
    }
    let mut hasil = String::new();
    for c in teks.chars() {
        let calon = format!("{hasil}{c}..");
        if lebar_teks(&calon, ukuran, tebal) > maks {
            break;
        }
        hasil.push(c);
    }
    format!("{hasil}..")
}

fn teks_sel(sel: &Sel) -> (String, bool) {
    match sel {
        Sel::Kosong => (String::new(), false),
        Sel::Teks(t) => (latin(t), false),
        Sel::Rupiah(n) => (rupiah(*n), true),
        Sel::Jumlah(n) => (teks_jumlah(*n), true),
        Sel::Tanggal(t) => (waktu::tanggal(t).map(tanggal_panjang).unwrap_or_else(|_| latin(t)), false),
    }
}

pub fn pdf(l: &Laporan, tujuan: &Path) -> Hasil<()> {
    use printpdf::{BuiltinFont, Color, Line, Mm, PdfDocument, Point, Rgb};
    const TEPI: f32 = 12.0;
    const UKURAN: f32 = 8.5;
    const TINGGI_BARIS: f32 = 5.2;
    const JARAK: f32 = 2.0;

    let (lebar_hal, tinggi_hal) = if l.kolom.len() > 6 { (297.0, 210.0) } else { (210.0, 297.0) };
    let (doc, hal, lap) = PdfDocument::new(latin(&l.judul), Mm(lebar_hal), Mm(tinggi_hal), "Isi");
    let biasa = doc.add_builtin_font(BuiltinFont::Helvetica).map_err(Galat::internal)?;
    let tebal = doc.add_builtin_font(BuiltinFont::HelveticaBold).map_err(Galat::internal)?;

    // Lebar kolom: mengikuti isi terlebar, lalu diskalakan agar pas di halaman.
    let n = l.kolom.len().max(1);
    let lebar_isi = lebar_hal - 2.0 * TEPI;
    let mut lebar: Vec<f32> = l.kolom.iter().map(|k| lebar_teks(&latin(k), UKURAN, true)).collect();
    let mut kanan = vec![false; n];
    let mut alami = vec![0.0f32; n];
    for (baris, tebal_juga) in l.baris.iter().map(|b| (b, false)).chain(l.total.iter().map(|t| (t, true))) {
        for (i, sel) in baris.iter().enumerate().take(n) {
            let (t, angka) = teks_sel(sel);
            let w = lebar_teks(&t, UKURAN, tebal_juga);
            // Angka tidak pernah dipotong; teks panjang dibatasi agar tidak menghabiskan halaman.
            lebar[i] = lebar[i].max(if angka { w } else { w.min(70.0) });
            alami[i] = alami[i].max(w);
            kanan[i] = kanan[i] || angka;
        }
    }
    for w in &mut lebar {
        *w += 2.0 * JARAK;
    }
    let jumlah: f32 = lebar.iter().sum();
    // Kolom teks yang isinya paling panjang: tempat menaruh sisa ruang, atau yang dikecilkan bila kelebihan.
    let lentur = (0..n).filter(|i| !kanan[*i]).max_by(|a, b| alami[*a].total_cmp(&alami[*b]));
    match lentur {
        Some(i) if jumlah <= lebar_isi => lebar[i] += lebar_isi - jumlah,
        Some(i) if lebar[i] - (jumlah - lebar_isi) >= 22.0 => lebar[i] -= jumlah - lebar_isi,
        _ if jumlah > lebar_isi => {
            let skala = lebar_isi / jumlah;
            lebar.iter_mut().for_each(|w| *w *= skala);
        }
        _ => {}
    }
    let mut x_kolom = Vec::with_capacity(n);
    let mut x = TEPI;
    for w in &lebar {
        x_kolom.push(x);
        x += w;
    }

    let mut lapisan = doc.get_page(hal).get_layer(lap);
    let mut nomor = 1;
    let mut y;

    let tulis_baris = |lapisan: &printpdf::PdfLayerReference, y: f32, sel: &[Sel], font: &printpdf::IndirectFontRef, huruf_tebal: bool| {
        for (i, s) in sel.iter().enumerate().take(n) {
            let (t, _) = teks_sel(s);
            if t.is_empty() {
                continue;
            }
            let t = potong(&t, UKURAN, huruf_tebal, lebar[i] - 2.0 * JARAK);
            let x = if kanan[i] { x_kolom[i] + lebar[i] - JARAK - lebar_teks(&t, UKURAN, huruf_tebal) } else { x_kolom[i] + JARAK };
            lapisan.use_text(t, UKURAN, Mm(x), Mm(y), font);
        }
    };
    let garis = |lapisan: &printpdf::PdfLayerReference, y: f32, tebal_garis: f32| {
        lapisan.set_outline_color(Color::Rgb(Rgb::new(0.45, 0.45, 0.45, None)));
        lapisan.set_outline_thickness(tebal_garis);
        lapisan.add_line(Line { points: vec![(Point::new(Mm(TEPI), Mm(y)), false), (Point::new(Mm(lebar_hal - TEPI), Mm(y)), false)], is_closed: false });
    };
    let kepala_sel: Vec<Sel> = l.kolom.iter().map(|k| Sel::Teks(k.clone())).collect();
    let kepala_tabel = |lapisan: &printpdf::PdfLayerReference, y: f32| {
        // Kepala kolom angka ikut rata kanan.
        for (i, s) in kepala_sel.iter().enumerate() {
            let (t, _) = teks_sel(s);
            let t = potong(&t, UKURAN, true, lebar[i] - 2.0 * JARAK);
            let x = if kanan[i] { x_kolom[i] + lebar[i] - JARAK - lebar_teks(&t, UKURAN, true) } else { x_kolom[i] + JARAK };
            lapisan.use_text(t, UKURAN, Mm(x), Mm(y), &tebal);
        }
        garis(lapisan, y - 1.6, 0.6);
    };
    let kaki = |lapisan: &printpdf::PdfLayerReference, nomor: usize| {
        let t = format!("Halaman {nomor}");
        lapisan.use_text(t.clone(), 7.5, Mm(lebar_hal - TEPI - lebar_teks(&t, 7.5, false)), Mm(7.0), &biasa);
        lapisan.use_text(latin(&format!("{} - {}", l.nama_toko, l.judul)), 7.5, Mm(TEPI), Mm(7.0), &biasa);
    };

    // Kepala laporan (halaman pertama)
    y = tinggi_hal - TEPI - 4.0;
    lapisan.use_text(latin(&l.judul), 14.0, Mm(TEPI), Mm(y), &tebal);
    y -= 6.0;
    lapisan.use_text(latin(&l.nama_toko), 10.0, Mm(TEPI), Mm(y), &biasa);
    y -= 5.0;
    lapisan.use_text(latin(&l.periode), 9.0, Mm(TEPI), Mm(y), &biasa);
    y -= 9.0;
    kepala_tabel(&lapisan, y);
    y -= TINGGI_BARIS + 0.8;
    kaki(&lapisan, nomor);

    let dasar = TEPI + 6.0;
    let mut halaman_baru = |lapisan: &mut printpdf::PdfLayerReference, y: &mut f32, dengan_kepala: bool| {
        let (h, lp) = doc.add_page(Mm(lebar_hal), Mm(tinggi_hal), "Isi");
        *lapisan = doc.get_page(h).get_layer(lp);
        nomor += 1;
        kaki(lapisan, nomor);
        *y = tinggi_hal - TEPI - 4.0;
        if dengan_kepala {
            kepala_tabel(lapisan, *y);
            *y -= TINGGI_BARIS + 0.8;
        }
    };

    if l.baris.is_empty() {
        lapisan.use_text("Tidak ada data pada periode ini.", UKURAN, Mm(TEPI + JARAK), Mm(y), &biasa);
        y -= TINGGI_BARIS;
    }
    for baris in &l.baris {
        if y < dasar {
            halaman_baru(&mut lapisan, &mut y, true);
        }
        tulis_baris(&lapisan, y, baris, &biasa, false);
        y -= TINGGI_BARIS;
    }
    if let Some(total) = &l.total {
        if y < dasar + TINGGI_BARIS {
            halaman_baru(&mut lapisan, &mut y, true);
        }
        garis(&lapisan, y + TINGGI_BARIS - 1.6, 0.4);
        tulis_baris(&lapisan, y, total, &tebal, true);
        y -= TINGGI_BARIS;
    }
    if !l.ringkasan.is_empty() {
        y -= 4.0;
        if y < dasar + TINGGI_BARIS * l.ringkasan.len() as f32 {
            halaman_baru(&mut lapisan, &mut y, false);
        }
        for (label, sel) in &l.ringkasan {
            let (t, _) = teks_sel(sel);
            lapisan.use_text(latin(label), UKURAN, Mm(TEPI + JARAK), Mm(y), &tebal);
            lapisan.use_text(t.clone(), UKURAN, Mm(TEPI + 95.0 - lebar_teks(&t, UKURAN, false)), Mm(y), &biasa);
            y -= TINGGI_BARIS;
        }
    }

    let file = std::fs::File::create(tujuan).map_err(tak_bisa_disimpan)?;
    doc.save(&mut std::io::BufWriter::new(file)).map_err(Galat::internal)
}
