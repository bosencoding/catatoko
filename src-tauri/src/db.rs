//! Membuka SQLite dan menjalankan migrasi bernomor (`PRAGMA user_version`).

use crate::galat::Hasil;
use rusqlite::Connection;
use std::path::Path;

const MIGRASI: &[&str] = &[include_str!("../migrations/001_init.sql")];

pub fn versi_skema_terbaru() -> i64 {
    MIGRASI.len() as i64
}

pub fn buka(path: &Path) -> Hasil<Connection> {
    let mut conn = Connection::open(path)?;
    conn.query_row("PRAGMA journal_mode=DELETE", [], |_| Ok(()))?;
    atur(&conn)?;
    migrasi(&mut conn)?;
    Ok(conn)
}

pub fn buka_memori() -> Connection {
    let mut conn = Connection::open_in_memory().expect("sqlite memori");
    atur(&conn).expect("pragma");
    migrasi(&mut conn).expect("migrasi");
    conn
}

fn atur(conn: &Connection) -> Hasil<()> {
    conn.execute_batch("PRAGMA foreign_keys=ON; PRAGMA synchronous=FULL;")?;
    Ok(())
}

pub fn migrasi(conn: &mut Connection) -> Hasil<()> {
    let versi: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    for (i, sql) in MIGRASI.iter().enumerate().skip(versi as usize) {
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.execute_batch(&format!("PRAGMA user_version={}", i + 1))?;
        tx.commit()?;
    }
    Ok(())
}
