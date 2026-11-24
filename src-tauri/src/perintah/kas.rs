use super::dengan_sesi;
use crate::galat::Hasil;
use crate::kas::{self, AkunKas, BarisKas, KasMasukan, KategoriKas, Saldo, SaringKas};
use crate::state::AppState;
use tauri::State;

#[tauri::command(rename_all = "snake_case")]
pub fn kas_daftar(st: State<AppState>, saring: SaringKas) -> Hasil<Vec<BarisKas>> {
    dengan_sesi(&st, |c, s| kas::daftar(c, s, &saring))
}

#[tauri::command(rename_all = "snake_case")]
pub fn kas_saldo(st: State<AppState>) -> Hasil<Saldo> {
    dengan_sesi(&st, |c, s| kas::saldo(c, s))
}

#[tauri::command(rename_all = "snake_case")]
pub fn kas_tambah(st: State<AppState>, masukan: KasMasukan) -> Hasil<i64> {
    dengan_sesi(&st, |c, s| kas::tambah_manual(c, s, masukan))
}

#[tauri::command(rename_all = "snake_case")]
pub fn kas_ubah(st: State<AppState>, id: i64, masukan: KasMasukan) -> Hasil<()> {
    dengan_sesi(&st, |c, s| kas::ubah_manual(c, s, id, masukan))
}

#[tauri::command(rename_all = "snake_case")]
pub fn kas_batal(st: State<AppState>, id: i64, alasan: String) -> Hasil<()> {
    dengan_sesi(&st, |c, s| kas::batal_manual(c, s, id, &alasan))
}

#[tauri::command(rename_all = "snake_case")]
pub fn kas_pindah(st: State<AppState>, tanggal: String, dari_akun: i64, ke_akun: i64, jumlah: i64, keterangan: Option<String>) -> Hasil<i64> {
    dengan_sesi(&st, |c, s| kas::pindah(c, s, &tanggal, dari_akun, ke_akun, jumlah, keterangan.as_deref()))
}

#[tauri::command(rename_all = "snake_case")]
pub fn akun_kas_daftar(st: State<AppState>) -> Hasil<Vec<AkunKas>> {
    dengan_sesi(&st, |c, _| kas::akun_daftar(c))
}

#[tauri::command(rename_all = "snake_case")]
pub fn akun_kas_tambah(st: State<AppState>, nama: String) -> Hasil<i64> {
    dengan_sesi(&st, |c, s| kas::akun_tambah(c, s, &nama))
}

#[tauri::command(rename_all = "snake_case")]
pub fn akun_kas_ubah(st: State<AppState>, id: i64, nama: String, aktif: bool) -> Hasil<()> {
    dengan_sesi(&st, |c, s| kas::akun_ubah(c, s, id, &nama, aktif))
}

#[tauri::command(rename_all = "snake_case")]
pub fn kategori_kas_daftar(st: State<AppState>) -> Hasil<Vec<KategoriKas>> {
    dengan_sesi(&st, |c, _| kas::kategori_daftar(c))
}

#[tauri::command(rename_all = "snake_case")]
pub fn kategori_kas_tambah(st: State<AppState>, nama: String) -> Hasil<i64> {
    dengan_sesi(&st, |c, s| kas::kategori_tambah(c, s, &nama))
}
