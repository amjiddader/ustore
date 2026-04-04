use anyhow::{Context, Result};
use rusqlite::Connection;
use serde::Serialize;
use std::time::SystemTime;

#[derive(Debug, Serialize)]
pub struct InstalledPackage {
    pub id: String,
    pub name: String,
    pub version: String,
    pub pkg_type: String,
    pub installed_date: String,
    pub dpkg_name: Option<String>,
    pub binary_name: Option<String>,
}

pub fn open_db() -> Result<Connection> {
    let path = crate::config::db_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).context("failed to create db directory")?;
    }
    let conn = Connection::open(&path).context("failed to open database")?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS installed (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            version TEXT NOT NULL,
            pkg_type TEXT NOT NULL,
            installed_date TEXT NOT NULL,
            dpkg_name TEXT,
            binary_name TEXT
        );",
    )
    .context("failed to run migrations")?;
    Ok(conn)
}

pub fn record_install(
    id: &str,
    name: &str,
    version: &str,
    pkg_type: &str,
    dpkg_name: Option<&str>,
    binary_name: Option<&str>,
) -> Result<()> {
    let conn = open_db()?;
    let date = current_date();
    conn.execute(
        "INSERT OR REPLACE INTO installed (id, name, version, pkg_type, installed_date, dpkg_name, binary_name)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![id, name, version, pkg_type, date, dpkg_name, binary_name],
    )?;
    Ok(())
}

pub fn record_remove(id: &str) -> Result<()> {
    let conn = open_db()?;
    conn.execute("DELETE FROM installed WHERE id = ?1", rusqlite::params![id])?;
    Ok(())
}

pub fn is_tracked(id: &str) -> Result<bool> {
    let conn = open_db()?;
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM installed WHERE id = ?1)",
        rusqlite::params![id],
        |row| row.get(0),
    )?;
    Ok(exists)
}

pub fn list_installed() -> Result<Vec<InstalledPackage>> {
    let conn = open_db()?;
    let mut stmt = conn.prepare(
        "SELECT id, name, version, pkg_type, installed_date, dpkg_name, binary_name FROM installed",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(InstalledPackage {
            id: row.get(0)?,
            name: row.get(1)?,
            version: row.get(2)?,
            pkg_type: row.get(3)?,
            installed_date: row.get(4)?,
            dpkg_name: row.get(5)?,
            binary_name: row.get(6)?,
        })
    })?;
    let mut packages = Vec::new();
    for row in rows {
        packages.push(row?);
    }
    Ok(packages)
}

fn current_date() -> String {
    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    let days = secs / 86400;
    let years_approx = days / 365;
    let year = 1970 + years_approx;
    let remaining_days = days - years_approx * 365;
    let month = remaining_days / 30 + 1;
    let day = remaining_days % 30 + 1;
    format!("{:04}-{:02}-{:02}", year, month.min(12), day.min(31))
}
