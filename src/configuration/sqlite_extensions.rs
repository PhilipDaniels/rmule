use crate::{file, times};
use anyhow::{bail, Result};
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput};
use rusqlite::{Connection, Params, ToSql};
use std::borrow::Cow;
use std::net::IpAddr;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub trait ConnectionExtensions {
    /// Execute a scalar query. The query is expected to return 1 row with 1
    /// column, the value in that cell is returned to the user. (The query
    /// can actually return more than 1 column, but the others will be
    /// ignored.)
    fn execute_scalar<T, P>(&self, sql: &str, params: P) -> Result<T>
    where
        P: Params,
        T: FromSql;

    /// Check to see if a table with the specified name exist.
    fn table_exists(&self, table_name: &str) -> Result<bool>;
}

impl ConnectionExtensions for Connection {
    fn execute_scalar<T, P>(&self, sql: &str, params: P) -> Result<T>
    where
        P: Params,
        T: FromSql,
    {
        match self.query_row(sql, params, |row| row.get::<_, T>(0)) {
            Ok(value) => Ok(value),
            Err(e) => bail!(e),
        }
    }

    fn table_exists(&self, table_name: &str) -> Result<bool> {
        let count: usize = self.execute_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?",
            [&table_name],
        )?;
        Ok(count == 1)
    }
}

/// A type that represents timestamps as we hold them in SQLite.
/// Can be easily serialized to and from the SQLite types.
/// Since we are storing datetimes in SQLite as strings, this is
/// easy (assuming nobody goes into the db and starts messing
/// with the strings...). I will live with that.
#[derive(Debug)]
pub struct DatabaseTime(String);

impl DatabaseTime {
    pub fn now() -> Self {
        Self(times::now_to_sql())
    }
}

impl ToSql for DatabaseTime {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.0.clone()))
    }
}

impl FromSql for DatabaseTime {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        value
            .as_str()
            .and_then(|s| FromSqlResult::Ok(Self(s.to_owned())))
    }
}

/// A type that represents a PathBuf as we hold them in SQLite.
/// In the database they are stored as strings.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DatabasePathBuf(PathBuf);

impl DatabasePathBuf {
    // Ensures that the path is absolute, placing it into a directory if necessary.
    // Returns true if it was necessary to change the path.
    pub fn make_absolute(&mut self, dir: &Path) -> bool {
        match file::make_absolute(&self.0, dir) {
            Cow::Borrowed(_) => false,
            Cow::Owned(p) => {
                self.0 = p;
                true
            }
        }
    }
}

impl Deref for DatabasePathBuf {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ToSql for DatabasePathBuf {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let path_as_string: String = self.0.to_string_lossy().into();
        Ok(ToSqlOutput::from(path_as_string))
    }
}

impl FromSql for DatabasePathBuf {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> FromSqlResult<Self> {
        value
            .as_str()
            .and_then(|s| FromSqlResult::Ok(Self(s.into())))
    }
}

impl From<PathBuf> for DatabasePathBuf {
    fn from(rhs: PathBuf) -> Self {
        Self(rhs)
    }
}

impl From<&Path> for DatabasePathBuf {
    fn from(rhs: &Path) -> Self {
        Self(rhs.into())
    }
}

impl From<&str> for DatabasePathBuf {
    fn from(rhs: &str) -> Self {
        Self(rhs.into())
    }
}

/// A type that represents an IpAddr as we hold them in SQLite.
/// In the database they are stored as strings.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DatabaseIpAddr(IpAddr);

impl Deref for DatabaseIpAddr {
    type Target = IpAddr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ToSql for DatabaseIpAddr {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let ip_as_string: String = self.0.to_string().into();
        Ok(ToSqlOutput::from(ip_as_string))
    }
}

impl FromSql for DatabaseIpAddr {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> FromSqlResult<Self> {
        let value = value.as_str()?;
        let ip_addr = match IpAddr::from_str(value) {
            Ok(ip) => ip,
            Err(e) => return FromSqlResult::Err(FromSqlError::InvalidType),
        };

        FromSqlResult::Ok(Self(ip_addr))
    }
}

impl From<IpAddr> for DatabaseIpAddr {
    fn from(rhs: IpAddr) -> Self {
        Self(rhs)
    }
}

impl From<&IpAddr> for DatabaseIpAddr {
    fn from(rhs: &IpAddr) -> Self {
        Self(rhs.clone())
    }
}
