use anyhow::{Result, bail};
use rusqlite::Connection;

use super::sqlite_extensions::ConnectionExtensions;

pub fn get_database_version(db: &Connection) -> Result<usize> {
    match db.query_row("SELECT version FROM version", [], |row| row.get(0)) {
        Ok(v) => Ok(v),
        Err(_) => Ok(0)
    }
}

pub fn set_database_version(db: &Connection, version: usize) -> Result<()> {
    match db.execute("UPDATE version SET version = ?", [version]) {
        Ok(_) => Ok(()),
        Err(e) => bail!(e)
    }
}

/// Applies all necessary database migrations to bring the database up to date.
pub fn apply_database_migrations(db: &Connection) -> Result<()> {
    let db_version = match db.table_exists("version") {
        Ok(_) => get_database_version(db)? as usize,
        Err(_) => 0
    };
    
    eprintln!("db_version is {}", db_version);

    // If db_version is 0 it means the 'version' table does not exist. We therefore
    // want to run the first migration, which creates it. And so on.
    let mut num_migrations = 0;
    for (idx, &mig) in MIGRATIONS.iter().enumerate().filter(|(idx, _)| *idx >= db_version) {
        apply_migration(idx, db, mig)?;
        num_migrations += 1;
    }

    match num_migrations {
        0 => eprintln!("Database is up to date"),
        1 => eprintln!("Applied 1 migration"),
        _ => eprintln!("Applied {num_migrations} migrations")
    }

    Ok(())
}

fn apply_migration(idx: usize, db: &Connection, migration: &str) -> Result<()> {
    let msg = migration.lines().take(1).next();
    match msg {
        Some(msg) => {
            eprint!("Executing migration {}: {}", idx, msg);
            db.execute_batch(migration)?;
            set_database_version(db, idx + 1)?;
            eprintln!(" SUCCESS");
        }
        None => panic!("Empty migration detected, number = {}", idx)
    }

    Ok(())
}

static MIGRATIONS: [&str; 2] = [
    include_str!("migration_files/000_000.sql"),
    include_str!("migration_files/000_001.sql")
];
