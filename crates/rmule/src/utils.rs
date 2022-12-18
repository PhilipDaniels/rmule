use anyhow::{bail, Result};
use std::fmt::{Display, Write};
use std::str::FromStr;

pub trait StringExtensions {
    fn split_comma_str_to_vec<T>(&self) -> Result<Vec<T>>
    where
        T: FromStr;
}

impl StringExtensions for &str {
    fn split_comma_str_to_vec<T>(&self) -> Result<Vec<T>>
    where
        T: FromStr,
    {
        let mut v = Vec::new();

        for s in self.split(',') {
            if s.trim().is_empty() {
                continue;
            }

            match s.parse() {
                Ok(n) => v.push(n),
                Err(_) => bail!("{s} cannot be converted to the specified type"),
            }
        }
        Ok(v)
    }
}

impl StringExtensions for String {
    fn split_comma_str_to_vec<T>(&self) -> Result<Vec<T>>
    where
        T: FromStr,
    {
        self.as_str().split_comma_str_to_vec()
    }
}

pub trait SliceExtensions<T> {
    fn to_comma_string(&self) -> Option<String>
    where
        T: Display;
}

impl<T> SliceExtensions<T> for [T]
where
    T: Display,
{
    fn to_comma_string(&self) -> Option<String>
    where
        T: Display,
    {
        if self.is_empty() {
            return None;
        }

        let mut s = String::new();

        for v in self {
            if !s.is_empty() {
                s += ",";
            }
            write!(s, "{v}").unwrap();
        }

        Some(s)
    }
}
