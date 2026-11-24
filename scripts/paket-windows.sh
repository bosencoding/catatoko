#!/usr/bin/env bash
# Membangun Catat untuk Windows 64-bit dari Linux (kompilasi silang) dan mengemasnya
# sebagai folder portabel. Hasil: dist-portable/Catat-windows-x86_64.zip
#
# Prasyarat:
#   - mingw-w64 (x86_64-w64-mingw32-gcc, -g++, -ar, -windres) ada di PATH
#       Arch: pacman -S mingw-w64-gcc     Debian/Ubuntu: apt install mingw-w64
#   - rustup target add x86_64-pc-windows-gnu
#   - python3 (untuk membuat zip)
set -euo pipefail
akar="$(cd "$(dirname "$0")/.." && pwd)"
cd "$akar"
target=x86_64-pc-windows-gnu

command -v x86_64-w64-mingw32-gcc >/dev/null || { echo "x86_64-w64-mingw32-gcc tidak ditemukan di PATH (lihat prasyarat di atas)." >&2; exit 1; }
export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc
export CC_x86_64_pc_windows_gnu=x86_64-w64-mingw32-gcc
export CXX_x86_64_pc_windows_gnu=x86_64-w64-mingw32-g++
export AR_x86_64_pc_windows_gnu=x86_64-w64-mingw32-ar

pnpm install --frozen-lockfile
pnpm tauri build --no-bundle --target "$target"

hasil="$akar/src-tauri/target/$target/release"
keluar="$akar/dist-portable"
rm -rf "$keluar/Catat-windows"
mkdir -p "$keluar/Catat-windows/Catat"
cp "$hasil/catat.exe" "$hasil/WebView2Loader.dll" "$keluar/Catat-windows/Catat/"
# Notepad Windows membaca baris berakhiran CRLF.
sed 's/$/\r/' > "$keluar/Catat-windows/Catat/BACA-DULU.txt" <<'TXT'
CATAT - pencatatan kas dan stok
===============================

Menjalankan
  Klik dua kali "catat.exe". Tidak perlu dipasang.
  Berkas "WebView2Loader.dll" harus tetap berada di sebelah "catat.exe".

  Saat pertama dijalankan, Windows mungkin menampilkan peringatan
  "Windows protected your PC" karena program ini tidak bertanda tangan.
  Pilih "More info" lalu "Run anyway".

Data Anda
  Semua catatan dan foto ada di folder "data" di sebelah "catat.exe":
    data\catat.db    basis data
    data\foto\       foto barang dan kiriman
    data\cadangan\   hasil tombol Cadangkan
  Untuk pindah komputer atau flashdisk, salin SELURUH folder Catat ini.
  Folder ini harus bisa ditulis: jangan ditaruh di "Program Files", di CD,
  atau dijalankan langsung dari dalam berkas zip. Ekstrak dulu.

Yang dibutuhkan komputer
  Windows 10 atau 11, 64-bit, dengan Microsoft Edge WebView2 Runtime.
  Windows 11 sudah memilikinya. Di Windows 10 biasanya juga sudah ada;
  kalau jendela tidak muncul, pasang "WebView2 Runtime" dari situs Microsoft
  (sekali saja, butuh internet). Setelah itu Catat tidak butuh internet.

Penting
  - Catat kode pemulihan yang ditampilkan saat membuat akun pemilik.
    Tanpa kode itu, kata sandi pemilik yang lupa tidak bisa diatur ulang.
  - Buat cadangan secara berkala (Pengaturan > Cadangan data) dan simpan
    salinannya di tempat lain.
TXT
rm -f "$keluar/Catat-windows-x86_64.zip"
python3 - "$keluar/Catat-windows" "$keluar/Catat-windows-x86_64.zip" <<'PY'
import os, sys, zipfile
sumber, tujuan = sys.argv[1], sys.argv[2]
with zipfile.ZipFile(tujuan, "w", zipfile.ZIP_DEFLATED, compresslevel=9) as z:
    for dasar, _, berkas in os.walk(sumber):
        for b in sorted(berkas):
            jalur = os.path.join(dasar, b)
            z.write(jalur, os.path.relpath(jalur, sumber))
PY
rm -rf "$keluar/Catat-windows"
echo "Selesai: $keluar/Catat-windows-x86_64.zip"
python3 -c "import sys,zipfile; [print(f'{i.file_size:>10}  {i.filename}') for i in zipfile.ZipFile(sys.argv[1]).infolist()]" "$keluar/Catat-windows-x86_64.zip"
