use crate::file;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput};
use rusqlite::ToSql;
use std::borrow::Cow;
use std::fmt::Display;
use std::ops::Deref;
use std::str::FromStr;

/// A type that represents a PathBuf as we hold them in SQLite.
/// In the database they are stored as strings.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PathBuf(std::path::PathBuf);

impl PathBuf {
    // Ensures that the path is absolute, placing it into a directory if necessary.
    // Returns true if it was necessary to change the path.
    pub fn make_absolute(&mut self, dir: &std::path::Path) -> bool {
        match file::make_absolute(&self.0, dir) {
            Cow::Borrowed(_) => false,
            Cow::Owned(p) => {
                self.0 = p;
                true
            }
        }
    }
}

impl Deref for PathBuf {
    type Target = std::path::PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ToSql for PathBuf {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let path_as_string: String = self.0.to_string_lossy().into();
        Ok(ToSqlOutput::from(path_as_string))
    }
}

impl FromSql for PathBuf {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> FromSqlResult<Self> {
        value
            .as_str()
            .and_then(|s| FromSqlResult::Ok(Self(s.into())))
    }
}

impl From<std::path::PathBuf> for PathBuf {
    fn from(rhs: std::path::PathBuf) -> Self {
        Self(rhs)
    }
}

impl From<&std::path::Path> for PathBuf {
    fn from(rhs: &std::path::Path) -> Self {
        Self(rhs.into())
    }
}

impl From<&str> for PathBuf {
    fn from(rhs: &str) -> Self {
        Self(rhs.into())
    }
}

/// A type that represents an IpAddr as we hold them in SQLite.
/// In the database they are stored as strings.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct IpAddr(std::net::IpAddr);

impl Deref for IpAddr {
    type Target = std::net::IpAddr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ToSql for IpAddr {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let ip_as_string: String = self.0.to_string();
        Ok(ToSqlOutput::from(ip_as_string))
    }
}

impl FromSql for IpAddr {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> FromSqlResult<Self> {
        let value = value.as_str()?;
        let ip_addr = match std::net::IpAddr::from_str(value) {
            Ok(ip) => ip,
            Err(_) => return FromSqlResult::Err(FromSqlError::InvalidType),
        };

        FromSqlResult::Ok(Self(ip_addr))
    }
}

impl From<std::net::IpAddr> for IpAddr {
    fn from(rhs: std::net::IpAddr) -> Self {
        Self(rhs)
    }
}

impl From<&std::net::IpAddr> for IpAddr {
    fn from(rhs: &std::net::IpAddr) -> Self {
        Self(*rhs)
    }
}

impl Display for IpAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Default for IpAddr {
    fn default() -> Self {
        std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED).into()
    }
}
