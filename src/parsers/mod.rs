//! Implement parsers for legacy file formats
//! such as server.met.

use anyhow::Result;

pub struct ParsedServer {}

pub fn parse_servers(bytes: &[u8]) -> Result<Vec<ParsedServer>> {
    Ok(vec![])
}

#[cfg(test)]
mod test {
    #[test]
    pub fn test_parse_of_valid_server_data() {}

    pub fn test_parse_of_invalid_server_data() {}
}
