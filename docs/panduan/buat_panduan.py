#!/usr/bin/env python3
"""Membuat docs/Panduan-Pengguna-Catat.pdf.

Butuh: pip install reportlab pillow
Jalankan: python docs/panduan/buat_panduan.py
"""
from pathlib import Path

from reportlab.lib import colors
from reportlab.lib.enums import TA_CENTER
from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import ParagraphStyle
from reportlab.lib.units import mm
from reportlab.platypus import (
    BaseDocTemplate,
    CondPageBreak,
    Frame,
    Image,
    KeepTogether,
    NextPageTemplate,
    PageBreak,
    PageTemplate,
    Paragraph,
    Spacer,
    Table,
    TableStyle,
)
from reportlab.platypus.tableofcontents import TableOfContents

SINI = Path(__file__).resolve().parent
GAMBAR = SINI / "gambar"
HASIL = SINI.parent / "Panduan-Pengguna-Catat.pdf"
VERSI = "Versi 0.1, November 2026"

UTAMA = colors.HexColor("#0f6b6b")
UTAMA_MUDA = colors.HexColor("#e3f1f0")
TEKS = colors.HexColor("#1f2933")
REDUP = colors.HexColor("#5f6b76")
GARIS = colors.HexColor("#d5dbe0")
KUNING = colors.HexColor("#fff6dd")
KUNING_TEPI = colors.HexColor("#e0b23c")
MERAH = colors.HexColor("#fdecec")
MERAH_TEPI = colors.HexColor("#d9534f")

LEBAR, TINGGI = A4
TEPI = 20 * mm
LEBAR_ISI = LEBAR - 2 * TEPI

g = {
    "isi": ParagraphStyle("isi", fontName="Helvetica", fontSize=10.5, leading=15.5, textColor=TEKS, spaceAfter=6),
    "bab": ParagraphStyle("bab", fontName="Helvetica-Bold", fontSize=20, leading=25, textColor=UTAMA, spaceBefore=0, spaceAfter=10, keepWithNext=1),
    "sub": ParagraphStyle("sub", fontName="Helvetica-Bold", fontSize=13, leading=17, textColor=TEKS, spaceBefore=12, spaceAfter=5, keepWithNext=1),
    "langkah": ParagraphStyle("langkah", fontName="Helvetica", fontSize=10.5, leading=15.5, textColor=TEKS, leftIndent=18, firstLineIndent=-18, spaceAfter=3.5),
    "butir": ParagraphStyle("butir", fontName="Helvetica", fontSize=10.5, leading=15.5, textColor=TEKS, leftIndent=14, firstLineIndent=-10, spaceAfter=3),
    "kotak": ParagraphStyle("kotak", fontName="Helvetica", fontSize=10, leading=14.5, textColor=TEKS),
    "ket": ParagraphStyle("ket", fontName="Helvetica-Oblique", fontSize=9, leading=12, textColor=REDUP, alignment=TA_CENTER, spaceBefore=3, spaceAfter=10),
    "sel": ParagraphStyle("sel", fontName="Helvetica", fontSize=9.5, leading=13, textColor=TEKS),
    "selkepala": ParagraphStyle("selkepala", fontName="Helvetica-Bold", fontSize=9.5, leading=13, textColor=colors.white),
    "toc1": ParagraphStyle("toc1", fontName="Helvetica-Bold", fontSize=11, leading=18, textColor=TEKS, spaceBefore=4),
    "toc2": ParagraphStyle("toc2", fontName="Helvetica", fontSize=10, leading=15, textColor=REDUP, leftIndent=14),
}

isi_dokumen = []
nomor_bab = 0


def p(teks):
    isi_dokumen.append(Paragraph(teks, g["isi"]))


def bab(judul, butuh=95):
    """`butuh`: sisa tinggi halaman (mm) yang harus ada agar bab tidak pindah halaman."""
    global nomor_bab
    nomor_bab += 1
    # Bab pertama mulai di halaman baru; bab berikutnya menyambung, kecuali sisa halaman tinggal sedikit.
    if nomor_bab == 1:
        isi_dokumen.append(PageBreak())
    else:
        isi_dokumen.append(CondPageBreak(butuh * mm))
        isi_dokumen.append(Spacer(1, 16))
    isi_dokumen.append(Paragraph(f"{nomor_bab}. {judul}", g["bab"]))


def sub(judul):
    isi_dokumen.append(Paragraph(judul, g["sub"]))


def langkah(*butir):
    """Daftar bernomor."""
    for i, t in enumerate(butir, 1):
        isi_dokumen.append(Paragraph(f"<b>{i}.</b>&nbsp;&nbsp;{t}", g["langkah"]))
    isi_dokumen.append(Spacer(1, 4))


def butir(*isi):
    for t in isi:
        isi_dokumen.append(Paragraph(f"<font color='#0f6b6b'>&bull;</font>&nbsp;&nbsp;{t}", g["butir"]))
    isi_dokumen.append(Spacer(1, 4))


def kotak(judul, teks, jenis="info"):
    latar, tepi = {"info": (UTAMA_MUDA, UTAMA), "awas": (KUNING, KUNING_TEPI), "bahaya": (MERAH, MERAH_TEPI)}[jenis]
    t = Table([[Paragraph(f"<b>{judul}</b><br/>{teks}", g["kotak"])]], colWidths=[LEBAR_ISI])
    t.setStyle(
        TableStyle(
            [
                ("BACKGROUND", (0, 0), (-1, -1), latar),
                ("LINEBEFORE", (0, 0), (0, -1), 3, tepi),
                ("LEFTPADDING", (0, 0), (-1, -1), 10),
                ("RIGHTPADDING", (0, 0), (-1, -1), 10),
                ("TOPPADDING", (0, 0), (-1, -1), 7),
                ("BOTTOMPADDING", (0, 0), (-1, -1), 8),
            ]
        )
    )
    isi_dokumen.append(KeepTogether([Spacer(1, 3), t, Spacer(1, 9)]))


def gambar(nama, keterangan, lebar=LEBAR_ISI):
    berkas = GAMBAR / nama
    im = Image(str(berkas))
    skala = lebar / im.imageWidth
    im.drawWidth, im.drawHeight = lebar, im.imageHeight * skala
    bingkai = Table([[im]], colWidths=[lebar + 2])
    bingkai.setStyle(TableStyle([("BOX", (0, 0), (-1, -1), 0.6, GARIS), ("LEFTPADDING", (0, 0), (-1, -1), 1), ("RIGHTPADDING", (0, 0), (-1, -1), 1), ("TOPPADDING", (0, 0), (-1, -1), 1), ("BOTTOMPADDING", (0, 0), (-1, -1), 1)]))
    bingkai.hAlign = "CENTER"
    isi_dokumen.append(KeepTogether([Spacer(1, 4), bingkai, Paragraph(keterangan, g["ket"])]))


def tabel(kepala, baris, lebar_kolom):
    data = [[Paragraph(k, g["selkepala"]) for k in kepala]] + [[Paragraph(s, g["sel"]) for s in b] for b in baris]
    t = Table(data, colWidths=[w * LEBAR_ISI for w in lebar_kolom], repeatRows=1)
    t.setStyle(
        TableStyle(
            [
                ("BACKGROUND", (0, 0), (-1, 0), UTAMA),
                ("ROWBACKGROUNDS", (0, 1), (-1, -1), [colors.white, colors.HexColor("#f5f7f8")]),
                ("GRID", (0, 0), (-1, -1), 0.4, GARIS),
                ("VALIGN", (0, 0), (-1, -1), "TOP"),
                ("LEFTPADDING", (0, 0), (-1, -1), 7),
                ("RIGHTPADDING", (0, 0), (-1, -1), 7),
                ("TOPPADDING", (0, 0), (-1, -1), 5),
                ("BOTTOMPADDING", (0, 0), (-1, -1), 6),
            ]
        )
    )
    isi_dokumen.append(t)
    isi_dokumen.append(Spacer(1, 10))


# ---------------------------------------------------------------------------
# Kerangka halaman
# ---------------------------------------------------------------------------


class Dokumen(BaseDocTemplate):
    def afterFlowable(self, f):
        if isinstance(f, Paragraph):
            if f.style.name == "bab":
                kunci = f"bab-{self.seq.nextf('penanda')}"
                self.canv.bookmarkPage(kunci)
                self.canv.addOutlineEntry(f.getPlainText(), kunci, 0)
                self.notify("TOCEntry", (0, f.getPlainText(), self.page, kunci))
            elif f.style.name == "sub":
                kunci = f"sub-{self.seq.nextf('penanda')}"
                self.canv.bookmarkPage(kunci)
                self.canv.addOutlineEntry(f.getPlainText(), kunci, 1)
                self.notify("TOCEntry", (1, f.getPlainText(), self.page, kunci))


def sampul(c, doc):
    c.saveState()
    c.setFillColor(UTAMA)
    c.rect(0, TINGGI * 0.42, LEBAR, TINGGI * 0.58, stroke=0, fill=1)
    c.setFillColor(colors.white)
    c.setFont("Helvetica-Bold", 54)
    c.drawString(TEPI, TINGGI - 95 * mm, "Catat")
    c.setFont("Helvetica", 20)
    c.drawString(TEPI, TINGGI - 108 * mm, "Pencatatan kas dan stok toko")
    c.setFillColor(colors.HexColor("#bfe3e0"))
    c.setFont("Helvetica", 12)
    c.drawString(TEPI, TINGGI - 120 * mm, "Untuk Linux dan Windows, tanpa internet, data ikut di dalam foldernya")
    c.setFillColor(TEKS)
    c.setFont("Helvetica-Bold", 26)
    c.drawString(TEPI, TINGGI * 0.42 - 28 * mm, "Panduan Pengguna")
    c.setFillColor(REDUP)
    c.setFont("Helvetica", 12)
    c.drawString(TEPI, TINGGI * 0.42 - 38 * mm, "Untuk pemilik toko dan staf")
    c.drawString(TEPI, 22 * mm, VERSI)
    c.restoreState()


def biasa(c, doc):
    c.saveState()
    c.setStrokeColor(GARIS)
    c.setLineWidth(0.5)
    c.line(TEPI, 15 * mm, LEBAR - TEPI, 15 * mm)
    c.setFont("Helvetica", 8.5)
    c.setFillColor(REDUP)
    c.drawString(TEPI, 10.5 * mm, "Catat  |  Panduan Pengguna")
    c.drawRightString(LEBAR - TEPI, 10.5 * mm, f"Halaman {doc.page}")
    c.restoreState()


# ---------------------------------------------------------------------------
# Isi
# ---------------------------------------------------------------------------

isi_dokumen.append(NextPageTemplate("biasa"))
isi_dokumen.append(PageBreak())
isi_dokumen.append(Paragraph("Daftar isi", ParagraphStyle("judul-daftar", parent=g["bab"])))
daftar_isi = TableOfContents()
daftar_isi.levelStyles = [g["toc1"], g["toc2"]]
daftar_isi.dotsMinLevel = 0
isi_dokumen.append(daftar_isi)

# ---- 1
bab("Mengenal Catat")
p(
    "Catat adalah aplikasi untuk mencatat <b>uang</b> dan <b>barang</b> toko dalam satu tempat. "
    "Aplikasi ini dibuat untuk toko perikanan (pakan ikan dan perlengkapannya), tempat barang yang sama "
    "datang berkali-kali dengan harga dan spek yang bisa berbeda."
)
sub("Cara Catat memandang stok: per kiriman")
p(
    "Setiap kali barang datang, Catat mencatatnya sebagai satu <b>kiriman</b>. Satu barang bisa punya banyak kiriman, "
    "dan tiap kiriman punya tanggal masuk, jumlah, harga beli, spek, tanggal kedaluwarsa, dan foto sendiri."
)
tabel(
    ["Masuk", "Sisa", "Harga beli", "Spek", "Kedaluwarsa"],
    [
        ["03 Sep 2026  <b>(keluar dulu)</b>", "2 karung", "Rp 385.000", "protein 30%", "31 Jan 2027"],
        ["15 Okt 2026", "10 karung", "Rp 392.000", "protein 31%", "31 Mar 2027"],
        ["20 Nov 2026", "20 karung", "Rp 398.000", "protein 31%", "31 Mei 2027"],
    ],
    [0.32, 0.15, 0.17, 0.18, 0.18],
)
p(
    "Saat barang keluar, Catat <b>mengambil dari kiriman terlama lebih dulu</b>. Dengan begitu stok lama habis sebelum "
    "stok baru dipakai, dan laba dihitung dari harga beli kiriman yang benar-benar diambil."
)
sub("Yang bisa dilakukan")
butir(
    "Mencatat barang masuk dan barang keluar, lengkap dengan foto.",
    "Mencatat kas. Penjualan dan pembelian barang tercatat otomatis; uang lain (modal, listrik, gaji) dicatat sendiri.",
    "Melihat grafik arus kas dan arus barang, stok lama, stok menipis, dan barang yang mendekati kedaluwarsa.",
    "Membuat laporan dan mengunduhnya sebagai Excel atau PDF.",
    "Membuat akun untuk staf dan mengatur apa saja yang boleh mereka lihat dan lakukan.",
    "Mencadangkan semua data ke satu berkas, dan memulihkannya kembali.",
)
sub("Yang belum ada di versi ini")
butir(
    "Hutang dan piutang. Semua transaksi dianggap lunas.",
    "Konversi satuan. Satu barang punya satu satuan. Kalau dijual eceran per kg, catat barangnya dalam kg.",
    "Cetak struk, pemindai barcode, dan pajak.",
    "Pemakaian oleh beberapa komputer sekaligus. Satu folder data dipakai satu jendela pada satu waktu.",
)

# ---- 2
bab("Memulai")
sub("Menjalankan aplikasi")
p("Catat tidak perlu dipasang. Cukup ekstrak berkasnya, lalu jalankan dari folder <b>Catat</b>.")
tabel(
    ["Sistem", "Cara menjalankan", "Yang dibutuhkan"],
    [
        ["Windows 10/11", "Ekstrak <b>Catat-windows-x86_64.zip</b>, buka folder Catat, klik dua kali <b>catat.exe</b>. Berkas WebView2Loader.dll harus tetap di sebelahnya.", "WebView2 Runtime. Windows 11 sudah punya; di Windows 10 biasanya juga sudah ada."],
        ["Linux", "Ekstrak <b>Catat-linux-x86_64.tar.gz</b>, buka folder Catat, jalankan <b>catat</b>.", "Pustaka WebKitGTK 4.1."],
    ],
    [0.17, 0.5, 0.33],
)
kotak(
    "Peringatan Windows saat pertama dibuka",
    "Windows bisa menampilkan pesan \"Windows protected your PC\" karena programnya tidak bertanda tangan. "
    "Pilih <b>More info</b>, lalu <b>Run anyway</b>.",
    "awas",
)
sub("Di mana data disimpan")
p(
    "Semua catatan dan foto ada di folder <b>data</b> di sebelah program. Folder ini dibuat sendiri saat aplikasi pertama dibuka."
)
tabel(
    ["Isi folder data", "Kegunaan"],
    [
        ["catat.db", "Semua catatan: barang, kiriman, kas, pengguna, log."],
        ["foto", "Foto barang dan kiriman."],
        ["cadangan", "Hasil tombol Cadangkan."],
    ],
    [0.3, 0.7],
)
butir(
    "Untuk pindah komputer atau flashdisk, salin <b>seluruh folder Catat</b>, bukan hanya programnya.",
    "Folder itu harus bisa ditulis. Jangan menaruhnya di Program Files, di CD, atau menjalankannya langsung dari dalam berkas zip.",
    "Catat tidak pernah memindahkan data ke tempat lain secara diam-diam. Kalau foldernya tidak bisa ditulis, aplikasi menampilkan pesan.",
)

sub("Membuat akun pemilik")
p("Saat pertama dibuka, Catat meminta Anda membuat akun pemilik toko.")
langkah(
    "Isi nama toko, nama Anda, nama pengguna (dipakai saat masuk, misalnya <b>budi</b>), dan kata sandi minimal 8 karakter.",
    "Tekan <b>Buat akun pemilik</b>.",
    "Catat menampilkan <b>kode pemulihan</b>. Tulis kode itu di kertas dan simpan di tempat aman.",
    "Centang \"Sudah saya catat dan simpan\", lalu lanjut ke layar masuk.",
)
gambar("kode-pemulihan.png", "Kode pemulihan hanya ditampilkan sekali.", LEBAR_ISI * 0.46)
kotak(
    "Kode pemulihan tidak bisa dilihat lagi",
    "Kode ini satu-satunya cara mengatur ulang kata sandi pemilik yang lupa. Tanpa kode ini dan tanpa kata sandi, "
    "data tidak bisa dibuka oleh pemilik. Kalau kode dipakai, Catat memberi kode baru sebagai penggantinya.",
    "bahaya",
)
sub("Masuk dan mengenal layar")
gambar("masuk-aplikasi.png", "Layar masuk.", LEBAR_ISI * 0.46)
p("Setelah masuk, menu ada di sisi kiri. Menu yang tampil mengikuti hak akses akun Anda.")
tabel(
    ["Menu", "Isinya"],
    [
        ["Beranda", "Ringkasan: saldo, grafik, stok lama, stok menipis, yang mendekati kedaluwarsa."],
        ["Barang", "Daftar barang, rincian kiriman tiap barang."],
        ["Barang Masuk", "Mencatat kiriman yang datang."],
        ["Barang Keluar", "Mencatat barang yang terjual atau keluar."],
        ["Kas", "Semua uang masuk dan keluar."],
        ["Laporan", "Enam laporan, bisa diunduh sebagai Excel atau PDF."],
        ["Pengguna", "Akun pemilik dan staf (hanya pemilik)."],
        ["Pengaturan", "Kata sandi sendiri; untuk pemilik: nama toko, akun kas, cadangan, log."],
    ],
    [0.22, 0.78],
)
p(
    "Dua tombol di kiri atas, <b>Barang Keluar</b> dan <b>Barang Masuk</b>, selalu terlihat supaya pekerjaan yang paling sering "
    "bisa dibuka dengan sekali klik. Di kiri bawah ada nama Anda, tombol mode terang/gelap, dan tombol keluar."
)

# ---- 3
bab("Cara mengetik angka dan tanggal")
p("Bagian ini pendek tetapi penting, karena salah ketik angka berarti salah catat.")
sub("Uang")
p("Ketik angkanya saja. Titik hanya dibaca sebagai pemisah ribuan, dan Catat merapikannya sendiri.")
tabel(
    ["Anda mengetik", "Dibaca sebagai"],
    [["385000", "Rp 385.000"], ["385.000", "Rp 385.000"], ["1.250.000", "Rp 1.250.000"], ["12,5", "Ditolak (kolom menjadi merah): uang tidak memakai desimal."]],
    [0.35, 0.65],
)
sub("Jumlah barang")
p("Jumlah boleh pecahan sampai tiga angka di belakang koma, misalnya 0,5 kg.")
tabel(
    ["Anda mengetik", "Dibaca sebagai", "Catatan"],
    [
        ["10", "10", ""],
        ["0,5", "0,5", "Koma selalu desimal."],
        ["0.5 atau 0.500", "0,5", "Satu titik juga dibaca desimal, seperti angka di timbangan."],
        ["1.250", "1,25", "<b>Bukan</b> seribu dua ratus lima puluh."],
        ["1250", "1.250", "Untuk ribuan, ketik tanpa titik."],
        ["1.250,75", "1.250,75", "Titik sebagai ribuan hanya bila ada koma."],
    ],
    [0.27, 0.2, 0.53],
)
kotak(
    "Periksa angka setelah pindah kolom",
    "Begitu Anda pindah ke kolom lain, Catat menulis ulang angkanya dalam bentuk baku. "
    "Lihat sekilas angka itu dan total di bawahnya sebelum menekan Simpan.",
    "awas",
)
sub("Tanggal")
p(
    "Tanggal diketik <b>hari/bulan/tahun</b>, misalnya <b>31/01/2027</b>. Boleh juga 31-01-2027 atau 31/1/27. "
    "Di sisi kanan kolom, Catat menampilkan tanggal yang terbaca (31 Jan 2027) supaya bisa diperiksa."
)
p(
    "Kalau tanggalnya tidak ada (misalnya 31/11/2027) atau tidak lengkap, kolom menjadi merah dan catatan "
    "tidak bisa disimpan sampai diperbaiki. Kolom tanggal yang bersifat opsional boleh dikosongkan."
)

# ---- 4
bab("Langkah pertama: barang dan stok awal")
p("Sebelum mencatat penjualan, masukkan dulu barang yang Anda jual dan stok yang ada sekarang.")
sub("Menambah barang")
langkah(
    "Buka menu <b>Barang</b>, tekan <b>Barang baru</b>.",
    "Isi nama barang dan satuannya (karung, kg, buah, botol).",
    "Isi harga jual per satuan. Harga ini menjadi isian bawaan saat barang keluar dan masih bisa diubah tiap transaksi.",
    "Bila perlu, isi stok minimum. Barang yang stoknya di bawah angka ini diberi tanda \"Stok menipis\".",
    "Pilih kategori, atau tekan <b>Kategori baru</b> untuk membuatnya.",
    "Bila perlu, tekan <b>Pilih foto</b>. Foto diperkecil otomatis.",
    "Tekan <b>Simpan</b>. Kode barang dibuat otomatis (B0001, B0002, dan seterusnya) kalau dikosongkan.",
)
gambar("barang.png", "Daftar barang. Klik satu barang untuk melihat kirimannya.")
kotak(
    "Stok dan harga beli tidak diisi di sini",
    "Formulir barang hanya berisi hal yang tetap: nama, satuan, harga jual. Jumlah stok dan harga beli dicatat per kiriman lewat Barang Masuk.",
)
sub("Memasukkan stok yang sudah ada")
p("Stok yang sudah ada di toko sebelum memakai Catat dimasukkan lewat <b>Barang Masuk</b> dengan centang <b>Stok awal</b>.")
langkah(
    "Buka <b>Barang Masuk</b>, pilih barangnya.",
    "Isi jumlah dan harga beli per satuan (harga saat Anda membelinya dulu).",
    "Isi <b>tanggal masuk</b> dengan tanggal barang itu sebenarnya datang, bukan tanggal hari ini.",
    "Centang <b>Stok awal</b>, lalu Simpan.",
)
butir(
    "Dengan centang Stok awal, kas tidak berkurang, karena uangnya sudah keluar sebelum Anda memakai Catat.",
    "Kalau barang yang sama punya stok lama dan stok baru dengan harga atau spek berbeda, masukkan sebagai <b>dua kiriman terpisah</b> dengan tanggal masuk masing-masing.",
    "Tanggal masuk menentukan urutan keluar, dan barang tidak bisa dicatat keluar pada tanggal sebelum kirimannya masuk.",
)

# ---- 5
bab("Barang Masuk", butuh=165)
p("Setiap kali barang datang, catat sebagai satu kiriman.")
gambar("barang-masuk.png", "Formulir di kiri, kiriman yang terakhir dicatat di kanan.")
langkah(
    "Pilih barang. Ketik sebagian nama atau kodenya untuk mencari. Kalau barangnya belum ada, tekan <b>Baru</b>.",
    "Isi <b>jumlah</b> dan <b>harga beli per satuan</b>.",
    "Periksa <b>tanggal masuk</b> (bawaannya hari ini).",
    "Bila ada, isi <b>kedaluwarsa</b>, <b>spek</b> (yang membedakan kiriman ini, misalnya protein 31%), <b>pemasok</b>, dan foto.",
    "Pilih <b>Dibayar dari</b>: akun kas yang dipakai membayar.",
    "Periksa angka <b>Kas keluar</b> di bawah, lalu tekan <b>Simpan</b>.",
)
p(
    "Begitu disimpan, stok bertambah dan Catat membuat catatan kas keluar \"Pembelian barang\" sebesar jumlah kali harga beli. "
    "Anda tidak perlu mencatatnya lagi di menu Kas."
)
kotak(
    "Mencatat beberapa kiriman berturut-turut",
    "Setelah Simpan, tanggal masuk, akun kas, dan centang Stok awal tetap seperti isian terakhir, supaya memasukkan banyak kiriman lebih cepat. Periksa tanggalnya bila kiriman berikutnya berbeda hari.",
)

# ---- 6
bab("Barang Keluar", butuh=165)
p("Dipakai setiap kali barang terjual atau keluar dari toko.")
gambar("barang-keluar.png", "Catat menunjukkan dari kiriman mana barang diambil.")
langkah(
    "Pilih barang. Di daftar pilihan terlihat stok tiap barang.",
    "Isi <b>jumlah</b>. Di bawah baris muncul rincian, misalnya \"2 dari 03 Sep\" dan \"1 dari 15 Okt\": dua diambil dari kiriman September, satu dari kiriman Oktober.",
    "Periksa <b>harga jual</b>. Isian bawaannya harga jual barang; ubah bila harganya berbeda untuk transaksi ini.",
    "Untuk barang lain dalam transaksi yang sama, tekan <b>Tambah barang lain</b>.",
    "Periksa tanggal dan <b>Uang masuk ke</b> (Tunai, Bank, dan sebagainya). Catatan seperti nama pembeli boleh diisi.",
    "Periksa <b>Total</b>, lalu tekan <b>Simpan</b>.",
)
p(
    "Begitu disimpan, sisa tiap kiriman berkurang dan Catat membuat catatan kas masuk \"Penjualan\". "
    "Transaksi itu muncul di daftar di bawah formulir."
)
sub("Urutan pengambilan")
butir(
    "Kiriman dengan <b>tanggal masuk terlama</b> diambil lebih dulu.",
    "Kalau satu kiriman tidak cukup, sisanya diambil dari kiriman berikutnya.",
    "Tanggal kedaluwarsa tidak mengubah urutan, hanya memunculkan peringatan.",
    "Kiriman yang <b>sudah lewat kedaluwarsa</b> tidak pernah diambil otomatis.",
)
sub("Memilih kiriman sendiri")
p(
    "Kadang pembeli meminta spek tertentu, atau Anda perlu mengeluarkan kiriman tertentu. Tekan <b>Ganti kiriman</b> di bawah baris, "
    "lalu isi jumlah yang diambil dari tiap kiriman. Baris itu diberi tanda \"dipilih sendiri\"."
)
p(
    "Kiriman yang sudah kedaluwarsa hanya bisa dikeluarkan lewat cara ini, dan Anda harus mencentang persetujuan dulu. "
    "Ini berguna misalnya untuk menjualnya murah atau mengeluarkannya dari toko."
)
sub("Kalau stok tidak cukup")
p(
    "Catat menampilkan pesan di bawah baris, misalnya \"Stok Pakan Apung 781-2 tidak cukup: diminta 33 karung, tersedia 32 karung\", "
    "dan tombol Simpan tidak bisa ditekan. Kurangi jumlahnya, atau catat dulu barang yang baru datang di Barang Masuk."
)
kotak(
    "Tidak ada penjualan setengah jadi",
    "Kalau satu baris dalam transaksi bermasalah, seluruh transaksi tidak disimpan. Tidak akan ada keadaan stok sudah berkurang tetapi kas belum bertambah.",
)

# ---- 7
bab("Memperbaiki kesalahan")
p(
    "Di Catat, transaksi barang <b>tidak dihapus</b>. Yang ada adalah <b>membatalkan</b>: catatannya tetap terlihat (dicoret) "
    "beserta alasannya, tetapi tidak dihitung lagi. Dengan begitu selalu ada jejak siapa mengubah apa."
)
tabel(
    ["Yang salah", "Cara memperbaiki", "Akibatnya"],
    [
        ["Barang keluar salah (jumlah, harga, barang)", "Di daftar bawah Barang Keluar, tekan <b>Batalkan</b>, tulis alasannya. Lalu catat ulang yang benar.", "Stok kembali ke kirimannya; kas penjualannya ikut batal."],
        ["Barang masuk salah dan <b>belum ada yang keluar</b>", "Di Barang Masuk atau rincian barang, tekan <b>Batalkan</b>. Lalu catat ulang.", "Stok kiriman itu hilang; kas pembeliannya ikut batal."],
        ["Harga beli, spek, kedaluwarsa, atau pemasok kiriman salah", "Buka barangnya, tekan <b>Perbaiki</b> pada kiriman itu.", "Kalau harga beli diubah, kas pembelian dan laba ikut menyesuaikan."],
        ["Jumlah kiriman salah dan <b>sudah ada yang keluar</b>", "Buka barangnya, tekan <b>Sesuaikan</b> dengan alasan \"Koreksi hitung\".", "Sisa kiriman berubah; kas tidak berubah."],
        ["Kas manual salah", "Di menu Kas, tekan <b>Ubah</b> atau <b>Batalkan</b>.", "Saldo menyesuaikan."],
    ],
    [0.27, 0.43, 0.3],
)
gambar("rincian-barang.png", "Rincian barang: tiap kiriman punya tombol Perbaiki, Sesuaikan, dan Batalkan.", LEBAR_ISI * 0.8)
sub("Penyesuaian stok")
p("Dipakai bila jumlah barang di rak berbeda dengan catatan, tanpa ada jual-beli:")
butir(
    "<b>Rusak</b>, <b>Kedaluwarsa</b>, <b>Hilang</b>: barang berkurang.",
    "<b>Pakai sendiri</b>: barang dipakai untuk keperluan toko atau pemilik.",
    "<b>Koreksi hitung</b>: hasil hitung ulang, bisa berkurang atau bertambah.",
)
p(
    "Penyesuaian tidak memengaruhi kas. Pengurangannya muncul sebagai <b>Kerugian stok</b> di laporan Laba Kotor, "
    "dinilai menurut harga beli kiriman itu."
)
kotak(
    "Catatan kas otomatis tidak bisa diubah langsung",
    "Kas \"Penjualan\" dan \"Pembelian barang\" bertanda <b>otomatis</b> dan selalu mengikuti transaksi barangnya. "
    "Untuk mengubahnya, batalkan atau perbaiki transaksi barangnya.",
)

# ---- 8
bab("Kas", butuh=165)
gambar("kas.png", "Saldo per akun di atas, lalu daftar catatan dengan saldo berjalan.", LEBAR_ISI * 0.84)
p("Penjualan dan pembelian barang sudah tercatat otomatis. Menu Kas dipakai untuk uang lainnya.")
sub("Mencatat kas masuk atau kas keluar")
langkah(
    "Tekan <b>Kas masuk</b> atau <b>Kas keluar</b>.",
    "Isi jumlah, pilih kategori (Modal, Listrik/Air, Gaji, Lain-lain, atau kategori buatan Anda) dan akunnya.",
    "Periksa tanggal, isi keterangan bila perlu, lalu Simpan.",
)
sub("Akun kas dan pindah antar akun")
p(
    "Akun kas adalah tempat uang disimpan. Bawaannya hanya <b>Tunai</b>. Pemilik bisa menambah akun lain, misalnya <b>Bank</b>, "
    "di menu Pengaturan. Dengan begitu saldo Tunai bisa dicocokkan dengan uang di laci."
)
p(
    "Saat menyetor uang tunai ke bank, pakai <b>Pindah akun</b>. Ini bukan pemasukan atau pengeluaran, "
    "jadi tidak ikut dihitung di laporan arus kas dan tidak mengubah total saldo."
)
sub("Membaca daftar kas")
butir(
    "Kolom <b>Saldo</b> adalah saldo setelah baris itu. Kalau menyaring satu akun, yang tampil saldo akun itu.",
    "Baris bertanda <b>otomatis</b> berasal dari Barang Masuk atau Barang Keluar.",
    "Baris yang dicoret sudah dibatalkan dan tidak dihitung.",
    "Di atas daftar, Anda bisa memilih periode, akun, kategori, dan arah (masuk atau keluar).",
)
kotak(
    "Saldo bisa minus",
    "Kalau Anda mencatat pembelian barang sebelum mencatat modal, saldo akan minus. Catat modal awal sebagai <b>Kas masuk</b> kategori Modal supaya saldonya sesuai kenyataan.",
    "awas",
)

# ---- 9
bab("Beranda", butuh=165)
gambar("beranda.png", "Kartu ringkasan dan grafik mengikuti periode yang dipilih di kanan atas.")
p("Beranda memberi gambaran cepat keadaan toko. Pilih periode di kanan atas; bawaannya bulan ini.")
tabel(
    ["Bagian", "Artinya"],
    [
        ["Saldo kas", "Total uang di semua akun saat ini, dengan rincian per akun."],
        ["Kas masuk / Kas keluar", "Jumlah pada periode yang dipilih. Pindah akun tidak dihitung."],
        ["Laba kotor penjualan", "Harga jual dikurangi harga beli barang yang terjual pada periode itu."],
        ["Nilai stok sekarang", "Sisa semua kiriman dikali harga belinya."],
        ["Grafik arus kas", "Batang hijau kas masuk, batang merah kas keluar, garis biru saldo."],
        ["Grafik arus barang", "Jumlah satuan yang masuk dan keluar, dijumlahkan dari semua barang."],
        ["Stok lama, habiskan dulu", "Kiriman tertua yang masih bersisa."],
        ["Mendekati kedaluwarsa", "Kiriman yang kedaluwarsa dalam 30 hari, atau sudah lewat."],
        ["Stok menipis", "Barang yang stoknya di bawah stok minimum."],
        ["Paling banyak keluar", "Lima barang yang paling banyak keluar pada periode itu."],
    ],
    [0.3, 0.7],
)
p(
    "Staf hanya melihat bagian yang sesuai haknya. Kartu laba dan nilai stok, misalnya, hanya tampil untuk akun yang boleh melihat harga beli."
)

# ---- 10
bab("Laporan", butuh=165)
gambar("laporan.png", "Pilih jenis laporan dan periodenya, lalu unduh.")
tabel(
    ["Laporan", "Isinya"],
    [
        ["Arus kas", "Semua uang masuk dan keluar, ringkasan per kategori, saldo awal dan saldo akhir."],
        ["Laba kotor", "Tiap barang yang terjual: harga jual, harga beli kirimannya, dan labanya. Di bawahnya ada kerugian stok."],
        ["Posisi stok", "Sisa dan nilai tiap kiriman <b>saat ini</b>. Laporan ini tidak memakai periode."],
        ["Kartu stok", "Riwayat satu barang: masuk, keluar, penyesuaian, dan saldonya."],
        ["Barang masuk", "Semua kiriman yang diterima pada periode itu."],
        ["Barang keluar", "Semua barang yang keluar pada periode itu."],
    ],
    [0.22, 0.78],
)
sub("Mengunduh sebagai Excel atau PDF")
langkah(
    "Pilih jenis laporan dan periodenya. Untuk Kartu stok, pilih juga barangnya.",
    "Periksa isinya di layar.",
    "Tekan <b>Unduh Excel</b> atau <b>Unduh PDF</b>, pilih tempat menyimpan.",
)
butir(
    "Simpan laporan di luar folder data Catat, misalnya di folder Dokumen.",
    "Transaksi yang sudah dibatalkan tidak ikut di laporan.",
    "Laporan Laba kotor, Posisi stok, dan Barang masuk memuat harga beli, jadi hanya tersedia bagi akun yang boleh melihat harga beli.",
)

# ---- 11
bab("Pengguna dan hak akses")
p("Hanya pemilik yang bisa membuka menu ini. Pemilik selalu punya akses penuh. Untuk staf, pemilik memilih haknya per bagian.")
gambar("pengguna.png", "Formulir akun baru dengan hak akses per bagian.", LEBAR_ISI * 0.68)
sub("Membuat akun staf")
langkah(
    "Buka <b>Pengguna</b>, tekan <b>Akun baru</b>.",
    "Isi nama, nama pengguna, dan kata sandi awal (minimal 8 karakter).",
    "Biarkan peran <b>Staf</b>. Hak bawaannya adalah kasir: melihat barang dan mencatat barang keluar.",
    "Ubah hak tiap bagian bila perlu, lalu Simpan. Beri tahu staf nama pengguna dan kata sandinya.",
)
sub("Arti tingkat hak")
tabel(
    ["Tingkat", "Yang boleh dilakukan"],
    [
        ["Tidak ada", "Menu bagian itu tidak tampil."],
        ["Lihat", "Melihat saja."],
        ["Tambah", "Melihat dan menambah catatan baru."],
        ["Ubah & batalkan", "Semua di atas, ditambah mengubah dan membatalkan catatan."],
    ],
    [0.25, 0.75],
)
sub("Harga beli dan laba")
butir(
    "Tanpa centang <b>Lihat harga beli dan laba</b>, akun itu tidak menerima harga beli, nilai stok, atau laba sama sekali.",
    "Hak <b>Barang Masuk</b> dan <b>Laporan</b> memuat harga beli, jadi centang itu otomatis ikut wajib.",
    "Hak <b>Kas</b> berarti melihat semua catatan kas, termasuk pembelian barang. Kalau staf tidak boleh tahu nilai pembelian, jangan beri hak Kas.",
    "Hak <b>Laporan</b> juga membuka laporan Arus kas, walaupun akun itu tidak punya hak Kas.",
    "Penjualan yang dicatat kasir tetap masuk ke kas, walaupun kasir tidak punya hak Kas.",
)
sub("Mengubah, menonaktifkan, dan mengatur ulang kata sandi")
butir(
    "<b>Ubah</b>: mengganti nama, hak, atau peran. Perubahan berlaku saat pengguna itu masuk lagi.",
    "<b>Akun aktif</b>: hilangkan centangnya agar akun tidak bisa masuk lagi. Catatan lamanya tetap ada.",
    "<b>Atur ulang sandi</b>: untuk staf yang lupa kata sandi.",
    "Harus selalu ada minimal satu akun pemilik yang aktif.",
)

# ---- 12
bab("Pengaturan, cadangan, dan keamanan", butuh=165)
gambar("pengaturan.png", "Pengaturan untuk pemilik. Staf hanya melihat bagian \"Kata sandi saya\".")
sub("Mengganti kata sandi sendiri")
p("Semua pengguna bisa melakukannya di <b>Pengaturan</b>, bagian Kata sandi saya. Kata sandi lama harus diisi.")
sub("Pengaturan toko (pemilik)")
butir(
    "<b>Nama toko</b>: tampil di layar dan di kepala laporan.",
    "<b>Kunci otomatis</b>: aplikasi meminta masuk lagi bila tidak dipakai selama sekian menit (1 sampai 120; bawaannya 10).",
    "<b>Akun kas</b> dan <b>Kategori kas</b>: menambah akun atau kategori baru. Akun yang tidak dipakai bisa dinonaktifkan.",
)
sub("Mencadangkan data")
p(
    "Tekan <b>Cadangkan sekarang</b>. Catat membuat satu berkas zip berisi semua catatan dan foto di folder data/cadangan, "
    "dan menampilkan lokasinya. <b>Cadangkan ke lokasi lain</b> menyimpannya ke tempat yang Anda pilih, misalnya flashdisk lain."
)
kotak(
    "Simpan salinan di tempat lain",
    "Cadangan yang hanya ada di folder yang sama tidak menolong kalau flashdisk atau komputernya rusak atau hilang. "
    "Buat cadangan secara berkala, misalnya tiap minggu, dan salin ke tempat lain.",
    "awas",
)
sub("Memulihkan dari cadangan")
langkah(
    "Tekan <b>Pulihkan dari cadangan</b>, pilih berkas zip cadangannya.",
    "Baca peringatannya, lalu tekan <b>Ya, ganti data sekarang</b>.",
    "Aplikasi kembali ke layar masuk. Masuk dengan akun dan kata sandi yang berlaku <b>saat cadangan itu dibuat</b>.",
)
butir(
    "Semua data yang ada diganti dengan isi cadangan.",
    "Sebelum mengganti, Catat mencadangkan data yang sekarang secara otomatis (berkas berawalan catat-sebelum-pulih), jadi masih bisa dikembalikan.",
    "Berkas yang rusak atau bukan cadangan Catat ditolak, dan data Anda tidak diubah.",
)
sub("Lupa kata sandi")
tabel(
    ["Siapa yang lupa", "Yang dilakukan"],
    [
        ["Staf", "Minta pemilik membuka Pengguna, lalu tekan <b>Atur ulang sandi</b>."],
        ["Pemilik", "Di layar masuk, tekan <b>Lupa kata sandi?</b>, isi kode pemulihan, nama pengguna pemilik, dan kata sandi baru. Catat kode pemulihan baru yang ditampilkan."],
    ],
    [0.25, 0.75],
)
sub("Pengaman lain yang bekerja sendiri")
butir(
    "Setelah lima kali salah kata sandi berturut-turut, akun ditahan sebentar. Tiap salah berikutnya, waktu tunggunya makin lama (paling lama 15 menit).",
    "<b>Log aktivitas</b> di Pengaturan mencatat siapa masuk, siapa menambah, mengubah, atau membatalkan apa. Log tidak bisa dihapus dan hanya bisa dilihat pemilik.",
    "Satu folder data hanya bisa dibuka satu jendela Catat. Jendela kedua menampilkan pesan.",
)
kotak(
    "Berkas data tidak dienkripsi",
    "Kata sandi melindungi pemakaian lewat aplikasi. Orang yang memegang flashdisk atau komputernya tetap bisa menyalin berkas datanya. "
    "Jaga flashdisk dan komputer Anda seperti menjaga buku kas.",
    "awas",
)

# ---- 13
bab("Tanya jawab")
tanya = [
    ("Jendela tidak muncul di Windows 10.", "Kemungkinan WebView2 Runtime belum ada. Pasang \"Microsoft Edge WebView2 Runtime\" dari situs Microsoft (sekali saja, butuh internet), lalu buka Catat lagi."),
    ("Muncul pesan \"Catat tidak bisa dibuka: Folder data tidak bisa ditulis\".", "Pindahkan seluruh folder Catat ke tempat yang bisa ditulis, misalnya folder Dokumen atau flashdisk yang tidak dikunci, lalu buka lagi. Data Anda tidak diubah."),
    ("Muncul pesan \"Catat sudah terbuka di jendela lain\".", "Pakai jendela yang sudah terbuka. Kalau tidak terlihat, tutup dulu semua jendela Catat, lalu buka lagi."),
    ("Aplikasi tiba-tiba kembali ke layar masuk.", "Itu kunci otomatis karena aplikasi lama tidak dipakai. Masuk lagi; data tidak hilang. Pemilik bisa mengubah lamanya di Pengaturan."),
    ("Barang yang baru saya catat masuk tidak bisa dijual pada tanggal kemarin.", "Barang tidak bisa keluar sebelum tanggal masuk kirimannya. Perbaiki tanggal masuk kiriman itu: batalkan lalu catat ulang dengan tanggal yang benar."),
    ("Stok ada, tetapi Catat bilang tidak cukup.", "Periksa apakah sebagian kirimannya sudah lewat kedaluwarsa. Kiriman itu tidak diambil otomatis; pakai Ganti kiriman bila tetap ingin mengeluarkannya."),
    ("Tombol Batalkan pada barang masuk tidak ada.", "Kiriman itu sudah pernah keluar atau disesuaikan. Batalkan dulu transaksi keluarnya, atau pakai Perbaiki dan Sesuaikan."),
    ("Staf saya tidak melihat menu Kas atau Laporan.", "Akunnya belum diberi hak itu. Pemilik mengaturnya di menu Pengguna; berlaku saat staf masuk lagi."),
    ("Saya ingin pindah ke komputer lain.", "Tutup Catat, salin seluruh folder Catat ke komputer baru, lalu jalankan dari sana. Atau buat cadangan, lalu pulihkan di Catat yang baru."),
    ("Foto dari HP tampil miring.", "Versi ini belum membaca arah foto dari kamera. Putar fotonya dulu di HP atau komputer sebelum dipilih."),
]
for t, j in tanya:
    isi_dokumen.append(KeepTogether([Paragraph(f"<b>{t}</b>", g["isi"]), Paragraph(j, ParagraphStyle("jawab", parent=g["isi"], leftIndent=10, spaceAfter=3, textColor=REDUP))]))

sub("Kebiasaan yang disarankan")
butir(
    "Catat transaksi saat terjadi, jangan ditumpuk sampai sore.",
    "Cocokkan saldo Tunai dengan uang di laci tiap tutup toko.",
    "Hitung ulang stok secara berkala; catat selisihnya lewat Sesuaikan.",
    "Buat cadangan tiap minggu dan simpan salinannya di tempat lain.",
    "Beri tiap staf akun sendiri. Jangan berbagi kata sandi pemilik.",
)


def buat():
    doc = Dokumen(str(HASIL), pagesize=A4, leftMargin=TEPI, rightMargin=TEPI, topMargin=18 * mm, bottomMargin=22 * mm, title="Catat - Panduan Pengguna", author="Catat", subject="Panduan pengguna aplikasi Catat", lang="id")
    bingkai = Frame(TEPI, 22 * mm, LEBAR_ISI, TINGGI - 40 * mm, id="isi", leftPadding=0, rightPadding=0, topPadding=0, bottomPadding=0)
    doc.addPageTemplates([PageTemplate(id="sampul", frames=[bingkai], onPage=sampul), PageTemplate(id="biasa", frames=[bingkai], onPage=biasa)])
    doc.multiBuild(isi_dokumen)
    print(f"Selesai: {HASIL}")


if __name__ == "__main__":
    buat()
