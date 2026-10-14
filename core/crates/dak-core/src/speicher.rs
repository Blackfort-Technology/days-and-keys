// SPDX-License-Identifier: MPL-2.0
//! Speicher auf dem Gerät: eine SQLite-Datei, nur Chiffretext und Metadaten ohne Inhalte
//! (Architektur AE-15). Nur dieses Modul öffnet SQLite.

use rusqlite::{Connection, OptionalExtension, params};

use crate::fehler::Ergebnis;
use crate::modell::Id;

pub const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS meta (k TEXT PRIMARY KEY, v BLOB NOT NULL) WITHOUT ROWID;
CREATE TABLE IF NOT EXISTS kette (nr INTEGER PRIMARY KEY, glied BLOB NOT NULL);
CREATE TABLE IF NOT EXISTS bund (klasse INTEGER PRIMARY KEY, blob BLOB NOT NULL);
CREATE TABLE IF NOT EXISTS datensatz (
    datensatz_id BLOB NOT NULL,
    revision_id BLOB NOT NULL,
    aktuell INTEGER NOT NULL,
    roh BLOB NOT NULL,
    PRIMARY KEY (datensatz_id, revision_id)
) WITHOUT ROWID;
CREATE TABLE IF NOT EXISTS tombstone (datensatz_id BLOB PRIMARY KEY) WITHOUT ROWID;
CREATE TABLE IF NOT EXISTS entwurf (id BLOB PRIMARY KEY, roh BLOB NOT NULL) WITHOUT ROWID;
";

pub fn oeffnen(pfad: &str) -> Ergebnis<Connection> {
    let c = Connection::open(pfad)?;
    einrichten(&c)?;
    Ok(c)
}

pub fn im_speicher() -> Ergebnis<Connection> {
    let c = Connection::open_in_memory()?;
    einrichten(&c)?;
    Ok(c)
}

fn einrichten(c: &Connection) -> Ergebnis<()> {
    c.pragma_update(None, "journal_mode", "WAL")?;
    c.pragma_update(None, "secure_delete", "ON")?;
    c.pragma_update(None, "foreign_keys", "ON")?;
    c.pragma_update(None, "synchronous", "FULL")?;
    c.execute_batch(SCHEMA)?;
    Ok(())
}

pub fn meta(c: &Connection, k: &str) -> Ergebnis<Option<Vec<u8>>> {
    Ok(
        c.query_row("SELECT v FROM meta WHERE k = ?1", [k], |r| r.get(0))
            .optional()?,
    )
}

pub fn meta_setzen(c: &Connection, k: &str, v: &[u8]) -> Ergebnis<()> {
    c.execute(
        "INSERT INTO meta (k, v) VALUES (?1, ?2) ON CONFLICT(k) DO UPDATE SET v = excluded.v",
        params![k, v],
    )?;
    Ok(())
}

pub fn meta_loeschen(c: &Connection, k: &str) -> Ergebnis<()> {
    c.execute("DELETE FROM meta WHERE k = ?1", [k])?;
    Ok(())
}

pub fn meta_u64(c: &Connection, k: &str) -> Ergebnis<Option<u64>> {
    Ok(meta(c, k)?
        .and_then(|v| v.try_into().ok())
        .map(u64::from_be_bytes))
}

pub fn meta_u64_setzen(c: &Connection, k: &str, v: u64) -> Ergebnis<()> {
    meta_setzen(c, k, &v.to_be_bytes())
}

pub fn kette(c: &Connection) -> Ergebnis<Vec<Vec<u8>>> {
    let mut s = c.prepare("SELECT glied FROM kette ORDER BY nr")?;
    let v = s
        .query_map([], |r| r.get(0))?
        .collect::<Result<Vec<Vec<u8>>, _>>()?;
    Ok(v)
}

pub fn kette_ersetzen(c: &Connection, glieder: &[Vec<u8>]) -> Ergebnis<()> {
    c.execute("DELETE FROM kette", [])?;
    for (i, g) in glieder.iter().enumerate() {
        c.execute(
            "INSERT INTO kette (nr, glied) VALUES (?1, ?2)",
            params![i as i64 + 1, g],
        )?;
    }
    Ok(())
}

pub fn kette_anhaengen(c: &Connection, nr: u64, g: &[u8]) -> Ergebnis<()> {
    c.execute(
        "INSERT INTO kette (nr, glied) VALUES (?1, ?2)",
        params![nr as i64, g],
    )?;
    Ok(())
}

pub fn bund(c: &Connection, klasse: u8) -> Ergebnis<Option<Vec<u8>>> {
    Ok(
        c.query_row("SELECT blob FROM bund WHERE klasse = ?1", [klasse], |r| {
            r.get(0)
        })
        .optional()?,
    )
}

pub fn bund_setzen(c: &Connection, klasse: u8, blob: &[u8]) -> Ergebnis<()> {
    c.execute("INSERT INTO bund (klasse, blob) VALUES (?1, ?2) ON CONFLICT(klasse) DO UPDATE SET blob = excluded.blob", params![klasse, blob])?;
    Ok(())
}

/// Schreibt eine neue Revision; frühere Revisionen desselben Datensatzes sind danach nicht mehr aktuell.
pub fn datensatz_schreiben(
    c: &Connection,
    datensatz_id: &Id,
    revision_id: &Id,
    roh: &[u8],
) -> Ergebnis<()> {
    c.execute(
        "UPDATE datensatz SET aktuell = 0 WHERE datensatz_id = ?1",
        [&datensatz_id[..]],
    )?;
    c.execute(
        "INSERT INTO datensatz (datensatz_id, revision_id, aktuell, roh) VALUES (?1, ?2, 1, ?3)",
        params![&datensatz_id[..], &revision_id[..], roh],
    )?;
    Ok(())
}

/// Ersetzt die Bytes einer vorhandenen Revision (neue Umhüllung bei Rotation).
pub fn datensatz_ersetzen(
    c: &Connection,
    datensatz_id: &Id,
    revision_id: &Id,
    roh: &[u8],
) -> Ergebnis<()> {
    let n = c.execute(
        "UPDATE datensatz SET roh = ?3 WHERE datensatz_id = ?1 AND revision_id = ?2",
        params![&datensatz_id[..], &revision_id[..], roh],
    )?;
    if n != 1 {
        return Err(crate::fehler::Fehler::NichtGefunden("Revision"));
    }
    Ok(())
}

pub fn datensaetze(c: &Connection, nur_aktuell: bool) -> Ergebnis<Vec<Vec<u8>>> {
    let sql = if nur_aktuell {
        "SELECT roh FROM datensatz WHERE aktuell = 1"
    } else {
        "SELECT roh FROM datensatz ORDER BY datensatz_id, revision_id"
    };
    let mut s = c.prepare(sql)?;
    let v = s
        .query_map([], |r| r.get(0))?
        .collect::<Result<Vec<Vec<u8>>, _>>()?;
    Ok(v)
}

pub fn datensatz_anzahl(c: &Connection) -> Ergebnis<u64> {
    Ok(c.query_row("SELECT COUNT(*) FROM datensatz", [], |r| r.get::<_, i64>(0))? as u64)
}

/// Löschen (Protokoll 7.2): alle Revisionen entfernen, Tombstone setzen.
pub fn loeschen(c: &Connection, datensatz_id: &Id) -> Ergebnis<()> {
    c.execute(
        "DELETE FROM datensatz WHERE datensatz_id = ?1",
        [&datensatz_id[..]],
    )?;
    c.execute(
        "INSERT OR IGNORE INTO tombstone (datensatz_id) VALUES (?1)",
        [&datensatz_id[..]],
    )?;
    Ok(())
}

pub fn tombstones(c: &Connection) -> Ergebnis<Vec<Vec<u8>>> {
    let mut s = c.prepare("SELECT datensatz_id FROM tombstone ORDER BY datensatz_id")?;
    let v = s
        .query_map([], |r| r.get(0))?
        .collect::<Result<Vec<Vec<u8>>, _>>()?;
    Ok(v)
}

pub fn ist_geloescht(c: &Connection, datensatz_id: &Id) -> Ergebnis<bool> {
    Ok(c.query_row(
        "SELECT 1 FROM tombstone WHERE datensatz_id = ?1",
        [&datensatz_id[..]],
        |_| Ok(()),
    )
    .optional()?
    .is_some())
}

pub fn entwurf_setzen(c: &Connection, id: &Id, roh: &[u8]) -> Ergebnis<()> {
    c.execute("INSERT INTO entwurf (id, roh) VALUES (?1, ?2) ON CONFLICT(id) DO UPDATE SET roh = excluded.roh", params![&id[..], roh])?;
    Ok(())
}

pub fn entwuerfe(c: &Connection) -> Ergebnis<Vec<(Vec<u8>, Vec<u8>)>> {
    let mut s = c.prepare("SELECT id, roh FROM entwurf")?;
    let v = s
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(v)
}

pub fn entwurf_loeschen(c: &Connection, id: &[u8]) -> Ergebnis<()> {
    c.execute("DELETE FROM entwurf WHERE id = ?1", [id])?;
    Ok(())
}

/// Leert alle Tabellen (für die Wiederherstellung in einen leeren Bestand).
pub fn leeren(c: &Connection) -> Ergebnis<()> {
    c.execute_batch("DELETE FROM meta; DELETE FROM kette; DELETE FROM bund; DELETE FROM datensatz; DELETE FROM tombstone; DELETE FROM entwurf;")?;
    Ok(())
}
