use std::fmt::Show;
use std::fmt::Formatter;

use message::header::Header;

pub struct VersionAck;

impl VersionAck
{
    pub fn new() -> VersionAck
    {
        VersionAck
    }

    pub fn serialize(&self) -> Vec<u8>
    {
        let header : Header;

        header = Header::new(::config::NETWORK,
                             "verack".to_string(),
                             0u32,
                             ::crypto::checksum(&Vec::new()));

        header.serialize()
    }

    pub fn unserialize(data : &Vec<u8>) -> VersionAck
    {
        assert!(data.len() == 0);

        VersionAck::new()
    }
}

impl Show for VersionAck
{
    fn fmt(&self, f : &mut Formatter) -> Result<(), ::std::fmt::FormatError>
    {
        let width = if f.width.is_some() { f.width.unwrap() } else { 0 };
        let space = String::from_str(" ").repeat(width);

        write!(f, "{}VersionAck", space)
    }
}
