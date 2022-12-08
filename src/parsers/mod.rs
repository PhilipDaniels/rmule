//! Implement parsers for legacy file formats
//! such as server.met.

use anyhow::{bail, Context, Result};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use core::panic;
use std::io::{Cursor, Read};
use std::net::{IpAddr, Ipv4Addr};

pub struct ParsedServer {
    pub ip_addr: IpAddr,
    pub port: u16,
    pub name: String,
    pub description: String,
    pub user_count: u32,
    pub low_id_user_count: u32,
    pub ping: u32,
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
        let server = parse_server(&mut input)?;
        servers.push(server);
    }

    Ok(servers)
}

fn parse_server(input: &mut Cursor<&[u8]>) -> Result<ParsedServer> {
    let ip_addr = input
        .read_u32::<BigEndian>()
        .with_context(|| "Could not read IP address")?;

    let port = input
        .read_u16::<LittleEndian>()
        .with_context(|| "Could not read port number")?;

    let tag_count = input
        .read_u32::<LittleEndian>()
        .with_context(|| "Could not read tag count")?;

    println!("expect {} tags", tag_count);

    // TODO: Consider the builder pattern.
    let mut server = ParsedServer {
        ip_addr: Ipv4Addr::from(ip_addr).into(),
        port,
        name: "".to_owned(),
        description: "".to_owned(),
        user_count: 0,
        low_id_user_count: 0,
        ping: 0,
    };

    for idx in 0..tag_count {
        let tag = parse_tag(input)?;
        match tag {
            ParsedTag::ServerName(s) => server.name = s,
            ParsedTag::Description(d) => server.description = d,
            ParsedTag::Ping(n) => server.ping = n,
            ParsedTag::Fail(_) => todo!(),
            ParsedTag::Preference(_) => todo!(),
            ParsedTag::DNS(_) => todo!(),
            ParsedTag::MaxUsers(_) => todo!(),
            ParsedTag::SoftFiles(_) => todo!(),
            ParsedTag::HardFiles(_) => todo!(),
            ParsedTag::LastPingTime(_) => todo!(),
            ParsedTag::Version(_) => todo!(),
            ParsedTag::UDPFlags(_) => todo!(),
            ParsedTag::AuxPortsList(_) => todo!(),
            ParsedTag::LowIdClients(_) => todo!(),
            ParsedTag::FileCount(_) => todo!(),
            ParsedTag::UserCount(n) => server.user_count = n,
            ParsedTag::LowIdUserCount(n) => server.low_id_user_count = n,
        }
    }

    Ok(server)
}

enum ParsedTag {
    ServerName(String),
    Description(String),
    Ping(u32),
    Fail(u32),
    Preference(u32),
    DNS(String),
    MaxUsers(u32),
    SoftFiles(u32),
    HardFiles(u32),
    LastPingTime(u32),
    Version(String),
    UDPFlags(u32),
    AuxPortsList(Vec<u16>),
    LowIdClients(u32),
    FileCount(u32),
    UserCount(u32),
    LowIdUserCount(u32),
}

fn read_string(input: &mut Cursor<&[u8]>, length: usize) -> Result<String> {
    let mut buf = vec![0u8; length];
    input.read_exact(&mut buf)?;
    Ok(String::from_utf8(buf)?)
}

fn parse_tag(input: &mut Cursor<&[u8]>) -> Result<ParsedTag> {
    let uName: u8;
    let mut mName: String = "".into();

    let mut uType = input.read_u8().with_context(|| "Could not read tag type")?;
    println!("\n\nGot tag type of {uType}");

    if (uType & 0x80) != 0 {
        uType &= 0x7F;
        uName = input.read_u8().with_context(|| "Could not read XXX")?;
    } else {
        let tag_name_length = input
            .read_u16::<LittleEndian>()
            .with_context(|| "Could not read tag length")?;

        println!("tag_name_length={tag_name_length}");

        if tag_name_length == 1 {
            uName = input.read_u8().with_context(|| "Could not read XXX")?;
        } else {
            uName = 0;
            mName = read_string(input, tag_name_length as usize)?;
        }
    }

    println!("utype={uType}, uname={uName}, mName={:?}", mName);

    let mut string_tag_value = "".to_owned();
    let mut numeric_tag_value = 0;

    if uType == 2 {
        let string_len = input.read_u16::<LittleEndian>()?;
        println!("We are reading a string of length {string_len}");
        string_tag_value = read_string(input, string_len as usize)?;
        println!("Got string_tag_value of {string_tag_value}");
    } else if uType == 3 {
        println!("We are reading a numeric tag named '{}'", mName);
        numeric_tag_value = input.read_u32::<LittleEndian>()?;
        println!("Got numeric_tag_value of {numeric_tag_value}");
    } else {
        println!("We appear to have gone wrong!");
    }

    Ok(match uName {
        0x01 => ParsedTag::ServerName(string_tag_value),
        0x0B => ParsedTag::Description(string_tag_value),
        0x0C => ParsedTag::Ping(numeric_tag_value),
        // TODO: Make this NONE
        0 => match mName.as_ref() {
            "users" => ParsedTag::UserCount(numeric_tag_value),
            "lowusers" => ParsedTag::LowIdUserCount(numeric_tag_value),
            _ => panic!("Unhandled named numeric tag"),
        },
        _ => panic!("Unhandled uName case"),
    })
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
        assert_eq!(s.name, "PeerBooter");
        assert_eq!(s.description, "eDonkey bridge for kademlia users");

        let s = &servers[1];
        assert_eq!(s.ip_addr, IpAddr::from([183, 136, 232, 234]));
        assert_eq!(s.port, 4244);
        assert_eq!(s.name, "WEB");
        assert_eq!(s.description, "eserver 17.15");

        let s = &servers[2];
        assert_eq!(s.ip_addr, IpAddr::from([80, 208, 228, 241]));
        assert_eq!(s.port, 8369);
        assert_eq!(s.name, "eMule Security");
        assert_eq!(s.description, "www.emule-security.org");

        let s = &servers[3];
        assert_eq!(s.ip_addr, IpAddr::from([62, 210, 28, 77]));
        assert_eq!(s.port, 7111);
        assert_eq!(s.name, "PEERATES.NET");
        assert_eq!(s.description, "http://edk.peerates.net");

        let s = &servers[4];
        assert_eq!(s.ip_addr, IpAddr::from([47, 37, 145, 12]));
        assert_eq!(s.port, 28288);
        assert_eq!(s.name, "new server");
        assert_eq!(s.description, "edonkey server");

        let s = &servers[5];
        assert_eq!(s.ip_addr, IpAddr::from([91, 208, 184, 143]));
        assert_eq!(s.port, 4232);
        assert_eq!(s.name, "!! Sharing-Devils No.1 !!");
        assert_eq!(s.description, "https://forum.sharing-devils.to");
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
        assert_eq!(s.name, "eMule Sunrise");
        assert_eq!(s.description, "Not perfect, but real");
    }
}
