use anyhow::{bail, Result};
use std::fmt::{Display, Write};
use std::str::FromStr;

pub trait StringExtensions {
    fn split_comma_to_vec<T>(&self) -> Result<Vec<T>>
    where
        T: FromStr;
}

impl StringExtensions for &str {
    fn split_comma_to_vec<T>(&self) -> Result<Vec<T>>
    where
        T: FromStr,
    {
        let mut v = Vec::new();

        for s in self.split(',') {
            match s.parse() {
                Ok(n) => v.push(n),
                Err(e) => bail!("{s} cannot be converted to the specified type"),
            }
        }
        Ok(v)
    }
}

impl StringExtensions for String {
    fn split_comma_to_vec<T>(&self) -> Result<Vec<T>>
    where
        T: FromStr,
    {
        let mut v = Vec::new();

        for s in self.split(',') {
            match s.parse() {
                Ok(n) => v.push(n),
                Err(e) => bail!("{s} cannot be converted to the specified type"),
            }
        }
        Ok(v)
    }
}

pub trait SliceExtensions<T> {
    fn to_comma_string(values: &[T]) -> String
    where
        T: Display;
}

impl<T> SliceExtensions<T> for &[T]
where
    T: Display,
{
    fn to_comma_string(values: &[T]) -> String
    where
        T: Display,
    {
        let mut s = String::new();

        for v in values {
            if s.len() > 0 {
                s += ",";
            }
            write!(s, "{v}").unwrap();
        }
        s
    }
}
