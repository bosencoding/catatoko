#!/usr/bin/env bash
# Membangun Catat versi rilis dan mengemasnya sebagai folder portabel untuk Linux.
# Hasil: dist-portable/Catat/ dan dist-portable/Catat-linux-<arsitektur>.tar.gz
set -euo pipefail
akar="$(cd "$(dirname "$0")/.." && pwd)"
cd "$akar"

pnpm install --frozen-lockfile
pnpm tauri build --no-bundle

keluar="$akar/dist-portable"
rm -rf "$keluar/Catat"
mkdir -p "$keluar/Catat"
install -m 755 "$akar/src-tauri/target/release/catat" "$keluar/Catat/catat"
cat > "$keluar/Catat/BACA-DULU.txt" <<'TXT'
CATAT - pencatatan kas dan stok
===============================

Menjalankan
  Klik dua kali berkas "catat", atau dari terminal:  ./catat

Data Anda
  Semua catatan dan foto ada di folder "data" di sebelah berkas "catat":
    data/catat.db    basis data
    data/foto/       foto barang dan kiriman
    data/cadangan/   hasil tombol Cadangkan
  Untuk pindah komputer atau flashdisk, salin SELURUH folder Catat ini.
  Folder ini harus bisa ditulis (jangan di CD atau flashdisk yang dikunci).

Yang dibutuhkan komputer
  Linux 64-bit dengan pustaka WebKitGTK 4.1 (paket "webkit2gtk-4.1" di Arch,
  "libwebkit2gtk-4.1-0" di Debian/Ubuntu). Tidak perlu internet.

Penting
  - Catat kode pemulihan yang ditampilkan saat membuat akun pemilik.
    Tanpa kode itu, kata sandi pemilik yang lupa tidak bisa diatur ulang.
  - Buat cadangan secara berkala (Pengaturan > Cadangan data) dan simpan
    salinannya di tempat lain.
TXT
tar -C "$keluar" -czf "$keluar/Catat-linux-$(uname -m).tar.gz" Catat
echo "Selesai: $keluar/Catat-linux-$(uname -m).tar.gz"
ls -lh "$keluar/Catat" "$keluar"/*.tar.gz
