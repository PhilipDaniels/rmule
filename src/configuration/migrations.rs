use anyhow::{bail, Result};
use rusqlite::Connection;
use tracing::info;

use super::sqlite_extensions::ConnectionExtensions;

/// Applies all necessary database migrations to bring the database up to date.
pub fn apply_database_migrations(conn: &Connection) -> Result<()> {
    let db_version = match conn.table_exists("version") {
        Ok(_) => get_database_version(conn)? as usize,
        Err(_) => 0,
    };

    info!("db_version is {}", db_version);

    // If db_version is 0 it means the 'version' table does not exist. We therefore
    // want to run the first migration, which creates it. And so on.
    let mut num_migrations = 0;
    for (idx, &mig) in MIGRATIONS.iter().enumerate().filter(|(idx, _)| *idx >= db_version) {
        apply_migration(idx, conn, mig)?;
        num_migrations += 1;
    }

    match num_migrations {
        0 => info!("Database is up to date"),
        1 => info!("Applied 1 migration"),
        _ => info!("Applied {num_migrations} migrations"),
    }

    Ok(())
}

fn apply_migration(idx: usize, conn: &Connection, migration: &str) -> Result<()> {
    let msg = migration.lines().take(1).next();

    match msg {
        Some(msg) => {
            // Trim off the start of the SQL comment (so we expect each script
            // to start with a descriptive comment...)
            let msg = &msg[3..];
            info!("Executing migration {}: {}", idx, msg);
            conn.execute_batch(migration)?;
            set_database_version(conn, idx + 1)?;
            info!(" SUCCESS.");
        }
        None => panic!("Empty migration detected, number = {}", idx),
    }

    Ok(())
}

static MIGRATIONS: [&str; 4] = [
    include_str!("migration_files/0000.sql"),
    include_str!("migration_files/0001.sql"),
    include_str!("migration_files/0002.sql"),
    include_str!("migration_files/0003.sql"),
];

fn get_database_version(conn: &Connection) -> Result<usize> {
    match conn.query_row("SELECT version FROM version", [], |row| row.get(0)) {
        Ok(v) => Ok(v),
        Err(_) => Ok(0),
    }
}

fn set_database_version(conn: &Connection, version: usize) -> Result<()> {
    match conn.execute("UPDATE version SET version = ?", [version]) {
        Ok(_) => Ok(()),
        Err(e) => bail!(e),
    }
}
