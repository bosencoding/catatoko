//! Kata sandi (Argon2id), login dengan penahanan bertingkat, dan kode pemulihan.

use crate::akses::{Modul, Sesi, Tingkat};
use crate::galat::{Galat, Hasil};
use crate::{log, pengaturan, waktu};
use argon2::password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use chrono::{DateTime, Duration, Local, NaiveDateTime};
use rand::Rng;
use rusqlite::{params, Connection, OptionalExtension};

const ABJAD_KODE: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
const KUNCI_KODE: &str = "hash_kode_pemulihan";
const FORMAT_WAKTU: &str = "%Y-%m-%dT%H:%M:%S";
const PESAN_SALAH: &str = "Nama pengguna atau kata sandi salah.";

pub(crate) fn hash(sandi: &str) -> Hasil<String> {
    let garam = SaltString::generate(&mut OsRng);
    Ok(Argon2::default().hash_password(sandi.as_bytes(), &garam).map_err(Galat::internal)?.to_string())
}

pub(crate) fn cocok(sandi: &str, hash: &str) -> bool {
    PasswordHash::new(hash).map(|h| Argon2::default().verify_password(sandi.as_bytes(), &h).is_ok()).unwrap_or(false)
}

pub(crate) fn periksa_sandi_baru(sandi: &str) -> Hasil<()> {
    if sandi.chars().count() < 8 {
        return Err(Galat::validasi("Kata sandi minimal 8 karakter."));
    }
    Ok(())
}

fn kode_baru() -> String {
    let mut rng = rand::thread_rng();
    let huruf: Vec<char> = (0..20).map(|_| ABJAD_KODE[rng.gen_range(0..ABJAD_KODE.len())] as char).collect();
    huruf.chunks(5).map(|c| c.iter().collect::<String>()).collect::<Vec<_>>().join("-")
}

fn rapikan_kode(kode: &str) -> String {
    kode.chars().filter(|c| c.is_ascii_alphanumeric()).map(|c| c.to_ascii_uppercase()).collect()
}

pub fn perlu_setup(conn: &Connection) -> Hasil<bool> {
    let n: i64 = conn.query_row("SELECT count(*) FROM pengguna", [], |r| r.get(0))?;
    Ok(n == 0)
}

/// Membuat akun pemilik pertama. Mengembalikan kode pemulihan, yang hanya ditampilkan sekali.
pub fn setup_awal(conn: &mut Connection, nama_toko: &str, nama_pengguna: &str, nama: &str, sandi: &str) -> Hasil<String> {
    if !perlu_setup(conn)? {
        return Err(Galat::validasi("Aplikasi sudah disiapkan. Silakan masuk."));
    }
    let (nama_toko, nama_pengguna, nama) = (nama_toko.trim(), nama_pengguna.trim(), nama.trim());
    if nama_toko.is_empty() || nama_pengguna.is_empty() || nama.is_empty() {
        return Err(Galat::validasi("Nama toko, nama, dan nama pengguna wajib diisi."));
    }
    periksa_sandi_baru(sandi)?;
    let kode = kode_baru();
    let (h, hk) = (hash(sandi)?, hash(&rapikan_kode(&kode))?);
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO pengguna(nama_pengguna,nama,hash_sandi,peran,lihat_harga_beli,dibuat_pada) VALUES(?1,?2,?3,'pemilik',1,?4)",
        params![nama_pengguna, nama, h, waktu::kini()],
    )?;
    let id = tx.last_insert_rowid();
    pengaturan::tulis(&tx, "nama_toko", nama_toko)?;
    pengaturan::tulis(&tx, KUNCI_KODE, &hk)?;
    log::catat(&tx, Some(id), "auth.setup", &format!("Akun pemilik '{nama_pengguna}' dibuat untuk {nama_toko}"))?;
    tx.commit()?;
    Ok(kode)
}

pub(crate) fn muat_sesi(conn: &Connection, id: i64) -> Hasil<Sesi> {
    let (nama_pengguna, nama, peran, harga): (String, String, String, bool) = conn.query_row(
        "SELECT nama_pengguna, nama, peran, lihat_harga_beli FROM pengguna WHERE id=?1",
        [id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
    )?;
    let pemilik = peran == "pemilik";
    let mut hak = std::collections::HashMap::new();
    if !pemilik {
        let mut q = conn.prepare("SELECT modul, tingkat FROM hak_akses WHERE pengguna_id=?1")?;
        for baris in q.query_map([id], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))? {
            let (m, t) = baris?;
            if let Some(m) = Modul::dari_kunci(&m) {
                hak.insert(m, Tingkat::dari_angka(t));
            }
        }
    }
    Ok(Sesi { pengguna_id: id, nama_pengguna, nama, pemilik, lihat_harga_beli: pemilik || harga, hak })
}

/// Lama penahanan setelah `gagal` kali salah berturut-turut: 30 dtk × 2^(gagal−5), maks 900 dtk.
fn detik_tahan(gagal: i64) -> i64 {
    if gagal < 5 {
        return 0;
    }
    let pangkat = (gagal - 5).min(10) as u32;
    (30i64 << pangkat).min(900)
}

pub fn login(conn: &Connection, nama_pengguna: &str, sandi: &str, kini: DateTime<Local>) -> Hasil<Sesi> {
    let nama_pengguna = nama_pengguna.trim();
    let baris: Option<(i64, String, bool, i64, Option<String>)> = conn
        .query_row(
            "SELECT id, hash_sandi, aktif, gagal_login, ditahan_sampai FROM pengguna WHERE nama_pengguna=?1",
            [nama_pengguna],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .optional()?;
    let salah = || Galat::baru("SANDI_SALAH", PESAN_SALAH);
    let Some((id, hash_sandi, aktif, gagal, ditahan)) = baris else {
        // Tetap menghitung hash supaya lama respons tidak membocorkan ada/tidaknya akun.
        let _ = hash(sandi);
        // Nama yang diketik tidak dicatat: orang kerap salah mengetik kata sandi di kolom ini.
        log::catat(conn, None, "auth.gagal", "Percobaan masuk dengan nama pengguna yang tidak dikenal")?;
        return Err(salah());
    };
    let kini_naif = kini.naive_local();
    if let Some(sampai) = ditahan.as_deref().and_then(|t| NaiveDateTime::parse_from_str(t, FORMAT_WAKTU).ok()) {
        if kini_naif < sampai {
            let sisa = (sampai - kini_naif).num_seconds().max(1);
            return Err(Galat::baru("DITAHAN", format!("Terlalu banyak percobaan salah. Coba lagi dalam {sisa} detik.")));
        }
    }
    if !aktif || !cocok(sandi, &hash_sandi) {
        let gagal = gagal + 1;
        let tahan = detik_tahan(gagal);
        let sampai = (tahan > 0).then(|| (kini_naif + Duration::seconds(tahan)).format(FORMAT_WAKTU).to_string());
        conn.execute("UPDATE pengguna SET gagal_login=?1, ditahan_sampai=?2 WHERE id=?3", params![gagal, sampai, id])?;
        log::catat(conn, Some(id), "auth.gagal", &format!("Gagal masuk ke-{gagal} untuk '{nama_pengguna}'"))?;
        return Err(salah());
    }
    conn.execute("UPDATE pengguna SET gagal_login=0, ditahan_sampai=NULL WHERE id=?1", [id])?;
    log::catat(conn, Some(id), "auth.login", "Masuk")?;
    muat_sesi(conn, id)
}

/// Mengatur ulang sandi satu akun pemilik dengan kode pemulihan. Kode langsung diganti.
pub fn pulihkan_sandi(conn: &mut Connection, kode: &str, nama_pengguna: &str, sandi_baru: &str) -> Hasil<String> {
    let tersimpan = pengaturan::baca(conn, KUNCI_KODE)?.unwrap_or_default();
    if !cocok(&rapikan_kode(kode), &tersimpan) {
        log::catat(conn, None, "auth.gagal", "Kode pemulihan salah")?;
        return Err(Galat::baru("SANDI_SALAH", "Kode pemulihan salah."));
    }
    let id: Option<i64> = conn
        .query_row("SELECT id FROM pengguna WHERE nama_pengguna=?1 AND peran='pemilik' AND aktif=1", [nama_pengguna.trim()], |r| r.get(0))
        .optional()?;
    let Some(id) = id else {
        return Err(Galat::validasi("Kode pemulihan hanya untuk akun pemilik yang aktif. Periksa nama penggunanya."));
    };
    periksa_sandi_baru(sandi_baru)?;
    let kode_pengganti = kode_baru();
    let (h, hk) = (hash(sandi_baru)?, hash(&rapikan_kode(&kode_pengganti))?);
    let tx = conn.transaction()?;
    tx.execute("UPDATE pengguna SET hash_sandi=?1, gagal_login=0, ditahan_sampai=NULL WHERE id=?2", params![h, id])?;
    pengaturan::tulis(&tx, KUNCI_KODE, &hk)?;
    log::catat(&tx, Some(id), "auth.pulihkan", "Kata sandi pemilik diatur ulang dengan kode pemulihan")?;
    tx.commit()?;
    Ok(kode_pengganti)
}

pub fn ganti_sandi_sendiri(conn: &Connection, s: &Sesi, lama: &str, baru: &str) -> Hasil<()> {
    let h: String = conn.query_row("SELECT hash_sandi FROM pengguna WHERE id=?1", [s.pengguna_id], |r| r.get(0))?;
    if !cocok(lama, &h) {
        return Err(Galat::baru("SANDI_SALAH", "Kata sandi lama salah."));
    }
    periksa_sandi_baru(baru)?;
    conn.execute("UPDATE pengguna SET hash_sandi=?1 WHERE id=?2", params![hash(baru)?, s.pengguna_id])?;
    log::catat(conn, Some(s.pengguna_id), "auth.ganti_sandi", "Mengganti kata sandi sendiri")
}
