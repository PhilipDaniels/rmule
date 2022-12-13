use anyhow::{bail, Result};
use rusqlite::types::FromSql;
use rusqlite::{Connection, Params};

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

/*
pub trait RowExtensions {
    fn get_ip_addr<I: RowIndex>(&self, idx: I) -> Result<Option<IpAddr>>;
    fn get_path_buf<I: RowIndex>(&self, idx: I) -> Result<Option<PathBuf>>;
}

impl<'a> RowExtensions for Row<'a> {
    fn get_ip_addr<I: RowIndex>(&self, idx: I) -> Result<Option<IpAddr>> {
        let s = self.get_ref(idx)?.as_str_or_null()?;
        match s {
            Some(s) => Ok(Some(IpAddr::from_str(s)?)),
            None => Ok(None),
        }
    }

    fn get_path_buf<I: RowIndex>(&self, idx: I) -> Result<Option<PathBuf>> {
        let s = self.get_ref(idx)?.as_str_or_null()?;
        match s {
            Some(s) => Ok(Some(s.into())),
            None => Ok(None),
        }
    }
}
*/
