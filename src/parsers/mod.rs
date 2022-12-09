//! Implement parsers for legacy file formats
//! such as server.met.

use anyhow::{bail, Context, Result};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};
use std::net::{IpAddr, Ipv4Addr};

pub struct ParsedServer {
    pub ip_addr: IpAddr,
    pub port: u16,
    pub name: Option<String>,
    pub description: Option<String>,
    pub user_count: Option<u32>,
    pub low_id_user_count: Option<u32>,
    pub ping: Option<u32>,
    pub country: Option<String>,
    pub max_users: Option<u32>,
    pub file_count: Option<u32>,
    pub soft_file_limit: Option<u32>,
    pub hard_file_limit: Option<u32>,
    pub udp_flags: Option<u32>,
    pub version: Option<String>,
}

pub fn parse_servers(input: &[u8]) -> Result<Vec<ParsedServer>> {
    let mut input = Cursor::new(input);

    let header_byte = input
        .read_u8()
        .with_context(|| "Could not read header byte")?;

    if !(header_byte == 0x0E || header_byte == 0xE0) {
        bail!("Header byte is {}, which is not valid", header_byte);
    }

    let server_count = input
        .read_u32::<LittleEndian>()
        .with_context(|| "Could not read server count")?;

    if server_count == 0 {
        println!("Server count is 0, returning empty list");
        return Ok(Vec::new());
    }

    println!("Expecting {server_count} servers");

    let mut servers = Vec::new();

    for idx in 0..server_count {
        servers.push(parse_server(&mut input)?);
    }

    Ok(servers)
}

fn parse_server(input: &mut Cursor<&[u8]>) -> Result<ParsedServer> {
    // Yes, BigEndian, but when converted to an IP address
    // it comes out right.
    let ip_addr = input
        .read_u32::<BigEndian>()
        .with_context(|| "Could not read IP address")?;

    let port = input
        .read_u16::<LittleEndian>()
        .with_context(|| "Could not read port number")?;

    let tag_count = input
        .read_u32::<LittleEndian>()
        .with_context(|| "Could not read tag count")?;

    println!("Expecting {} tags", tag_count);

    let mut server = ParsedServer {
        ip_addr: Ipv4Addr::from(ip_addr).into(),
        port,
        name: None,
        description: None,
        user_count: None,
        low_id_user_count: None,
        ping: None,
        country: None,
        file_count: None,
        max_users: None,
        soft_file_limit: None,
        hard_file_limit: None,
        udp_flags: None,
        version: None,
    };

    for idx in 0..tag_count {
        let tag = parse_tag(input)?;

        // Parsed a tag we don't support?
        if tag.is_none() {
            continue;
        }

        let tag = tag.unwrap();

        match tag {
            ParsedTag::NoTag => {}
            ParsedTag::ServerName(s) => server.name = Some(s),
            ParsedTag::Description(d) => server.description = Some(d),
            ParsedTag::Ping(n) => server.ping = Some(n),
            ParsedTag::Fail(_) => todo!(),
            ParsedTag::Preference(_) => todo!(),
            ParsedTag::Dns(_) => todo!(),
            ParsedTag::MaxUsers(n) => server.max_users = Some(n),
            ParsedTag::SoftFiles(n) => server.soft_file_limit = Some(n),
            ParsedTag::HardFiles(n) => server.hard_file_limit = Some(n),
            ParsedTag::LastPingTime(_) => todo!(),
            ParsedTag::Version(s) => server.version = Some(s),
            ParsedTag::UDPFlags(n) => server.udp_flags = Some(n),
            ParsedTag::AuxPortsList(_) => todo!(),
            ParsedTag::FileCount(n) => server.file_count = Some(n),
            ParsedTag::UserCount(n) => server.user_count = Some(n),
            ParsedTag::LowIdUserCount(n) => server.low_id_user_count = Some(n),
            ParsedTag::Country(s) => server.country = Some(s),
        }
    }

    Ok(server)
}

enum ParsedTag {
    NoTag,
    ServerName(String),
    Description(String),
    Ping(u32),
    Fail(u32),
    Preference(u32),
    Dns(String),
    MaxUsers(u32),
    SoftFiles(u32),
    HardFiles(u32),
    LastPingTime(u32),
    Version(String),
    UDPFlags(u32),
    AuxPortsList(Vec<u16>),
    FileCount(u32),
    UserCount(u32),
    LowIdUserCount(u32),
    Country(String),
}

fn read_string(input: &mut Cursor<&[u8]>, length: usize) -> Result<String> {
    let mut buf = vec![0u8; length];
    input.read_exact(&mut buf)?;
    Ok(String::from_utf8(buf)?)
}

// Parses a single tag. If we encounter a tag that we do not
// recognise then we return None. This gives us forward compatibility
// with any tags that might suddenly appear out in the wild reaches
// of t'internet (and means that we don't need to support everything
// that *already* exists.)
fn parse_tag(input: &mut Cursor<&[u8]>) -> Result<Option<ParsedTag>> {
    let mut tag_type = input.read_u8().with_context(|| "Could not read tag_type")?;

    let mut numeric_tag_name: Option<u8> = None;
    let mut textual_tag_name: Option<String> = None;

    if (tag_type & 0x80) != 0 {
        tag_type &= 0x7F;
        numeric_tag_name = Some(input.read_u8().with_context(|| "Could not read XXX")?);
    } else {
        let tag_name_length = input
            .read_u16::<LittleEndian>()
            .with_context(|| "Could not read tag_name_length")?;

        if tag_name_length == 1 {
            numeric_tag_name = Some(input.read_u8().with_context(|| "Could not read XXX")?);
        } else {
            textual_tag_name = Some(read_string(input, tag_name_length as usize)?);
        }
    }

    let mut string_tag_value: Option<String> = None;
    let mut numeric_tag_value: Option<u32> = None;

    if tag_type == 2 {
        let string_len = input.read_u16::<LittleEndian>()?;
        //println!("We are reading a string of length {string_len}");
        string_tag_value = Some(read_string(input, string_len as usize)?);
        //println!("Got string_tag_value of {:?}", string_tag_value);
    } else if tag_type == 3 {
        // println!(
        //     "We are reading a numeric tag named '{:?}' or of numeric value {:?}",
        //     textual_tag_name, numeric_tag_name
        // );
        numeric_tag_value = Some(input.read_u32::<LittleEndian>()?);
        //println!("Got numeric_tag_value of {:?}", numeric_tag_value);
    } else {
        println!("Invalid tag type of {}", tag_type);
        bail!("Invalid tag type of {}", tag_type);
    }

    // The tag "name" is a number stored in numeric_tag_name XOR a string stored in
    // textual_tag_name.
    let tag = match numeric_tag_name {
        Some(0x01) => {
            ParsedTag::ServerName(string_tag_value.expect("Should have a string_tag_value"))
        }
        Some(0x0B) => {
            ParsedTag::Description(string_tag_value.expect("Should have a string_tag_value"))
        }
        Some(0x0C) => ParsedTag::Ping(numeric_tag_value.expect("Should have a numeric_tag_value")),
        Some(0x0D) => {
            //ParsedTag::Fail(numeric_tag_value.expect("Should have a numeric_tag_value"))
            todo!()
        }
        Some(0x0E) => {
            // aka priority
            //ParsedTag::Preference(numeric_tag_value.expect("Should have a
            // numeric_tag_value"))
            todo!()
        }
        Some(0x85) => {
            //ParsedTag::DNS(numeric_tag_value.expect("Should have a
            // numeric_tag_value"))
            todo!()
        }
        Some(0x87) => {
            ParsedTag::MaxUsers(numeric_tag_value.expect("Should have a numeric_tag_value"))
        }
        Some(0x88) => {
            ParsedTag::SoftFiles(numeric_tag_value.expect("Should have a numeric_tag_value"))
        }
        Some(0x89) => {
            ParsedTag::HardFiles(numeric_tag_value.expect("Should have a numeric_tag_value"))
        }
        Some(0x90) => {
            //ParsedTag::LastPingTime(numeric_tag_value.expect("Should have a
            // numeric_tag_value"))
            todo!()
        }
        Some(0x91) => {
            if string_tag_value.is_none() {
                let numeric_tag_value = numeric_tag_value.expect("Should have a numeric_tag_value");
                let major = numeric_tag_value >> 16;
                let minor = numeric_tag_value & 0xFFFF;
                ParsedTag::Version(format!("{}.{}", major, minor))
            } else {
                ParsedTag::Version(string_tag_value.expect("Should have a string_tag_value"))
            }
        }
        Some(0x92) => {
            // TODO: Unpack.
            ParsedTag::UDPFlags(numeric_tag_value.expect("Should have a numeric_tag_value"))
        }
        Some(0x93) => {
            //ParsedTag::AuxPortsList(numeric_tag_value.expect("Should have a
            // numeric_tag_value"))
            todo!()
        }
        Some(0x94) => {
            ParsedTag::LowIdUserCount(numeric_tag_value.expect("Should have a numeric_tag_value"))
        }
        None => match textual_tag_name.as_deref() {
            Some("users") => {
                ParsedTag::UserCount(numeric_tag_value.expect("Should have a numeric_tag_value"))
            }
            Some("lowusers") => ParsedTag::LowIdUserCount(
                numeric_tag_value.expect("Should have a numeric_tag_value"),
            ),
            Some("files") => {
                ParsedTag::FileCount(numeric_tag_value.expect("Should have a numeric_tag_value"))
            }
            Some("maxusers") => {
                ParsedTag::MaxUsers(numeric_tag_value.expect("Should have a numeric_tag_value"))
            }
            Some("country") => {
                ParsedTag::Country(string_tag_value.expect("Should have a numeric_tag_value"))
            }
            x @ _ => {
                println!(" >>>> Currently unhandled textual_tag_name: {:?}", x);
                ParsedTag::NoTag
            }
        },
        x @ _ => {
            println!(" >>>> Currently unhandled numeric_tag_name: {:?}", x);
            ParsedTag::NoTag
        }
    };

    Ok(Some(tag))
}

#[cfg(test)]
mod test {
    use super::parse_servers;
    use std::net::IpAddr;

    //#[test]
    pub fn test_parse_of_valid_server_data_minimal() {
        // This is a minimal, uncompressed file with only server name and description
        // tags.
        let input = include_bytes!("www.gruk.org.server.met");
        let servers = parse_servers(input).unwrap();
        assert_eq!(servers.len(), 6);

        let s = &servers[0];
        assert_eq!(s.ip_addr, IpAddr::from([212, 83, 184, 152]));
        assert_eq!(s.port, 7111);
        assert_eq!(s.name.as_deref(), Some("PeerBooter"));
        assert_eq!(
            s.description.as_deref(),
            Some("eDonkey bridge for kademlia users")
        );

        let s = &servers[1];
        assert_eq!(s.ip_addr, IpAddr::from([183, 136, 232, 234]));
        assert_eq!(s.port, 4244);
        assert_eq!(s.name.as_deref(), Some("WEB"));
        assert_eq!(s.description.as_deref(), Some("eserver 17.15"));

        let s = &servers[2];
        assert_eq!(s.ip_addr, IpAddr::from([80, 208, 228, 241]));
        assert_eq!(s.port, 8369);
        assert_eq!(s.name.as_deref(), Some("eMule Security"));
        assert_eq!(s.description.as_deref(), Some("www.emule-security.org"));

        let s = &servers[3];
        assert_eq!(s.ip_addr, IpAddr::from([62, 210, 28, 77]));
        assert_eq!(s.port, 7111);
        assert_eq!(s.name.as_deref(), Some("PEERATES.NET"));
        assert_eq!(s.description.as_deref(), Some("http://edk.peerates.net"));

        let s = &servers[4];
        assert_eq!(s.ip_addr, IpAddr::from([47, 37, 145, 12]));
        assert_eq!(s.port, 28288);
        assert_eq!(s.name.as_deref(), Some("new server"));
        assert_eq!(s.description.as_deref(), Some("edonkey server"));

        let s = &servers[5];
        assert_eq!(s.ip_addr, IpAddr::from([91, 208, 184, 143]));
        assert_eq!(s.port, 4232);
        assert_eq!(s.name.as_deref(), Some("!! Sharing-Devils No.1 !!"));
        assert_eq!(
            s.description.as_deref(),
            Some("https://forum.sharing-devils.to")
        );
    }

    #[test]
    pub fn test_parse_of_valid_server_data_maximal() {
        // This is a maximal, uncompressed file with most tags set.
        let input = include_bytes!("shortypower.org.server.met");
        let servers = parse_servers(input).unwrap();
        assert_eq!(servers.len(), 10);

        let s = &servers[0];
        assert_eq!(s.ip_addr, IpAddr::from([176, 123, 5, 89]));
        assert_eq!(s.port, 4725);
        assert_eq!(s.name.as_deref(), Some("eMule Sunrise"));
        assert_eq!(s.description.as_deref(), Some("Not perfect, but real"));
        assert_eq!(s.user_count, Some(10_114));
        assert_eq!(s.low_id_user_count, Some(7_583));
        assert_eq!(s.file_count, Some(5_120_913));
        assert_eq!(s.max_users, Some(50_000));
        assert_eq!(s.country.as_deref(), Some("md")); // Moldova
        assert_eq!(s.soft_file_limit, Some(100_000));
        assert_eq!(s.hard_file_limit, Some(100_001));
        assert_eq!(s.udp_flags, Some(6139)); // TODO: unpack into a flags enum.
        assert_eq!(s.version.as_deref(), Some("17.15")); // Moldova
        assert_eq!(s.ping, Some(47));
    }
}
