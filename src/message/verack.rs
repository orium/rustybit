use std::fmt::Show;
use std::fmt::Formatter;

use message::header::Header;

pub struct VerAck;

impl VerAck
{
    pub fn new() -> VerAck
    {
        VerAck
    }

    pub fn serialize(&self) -> Vec<u8>
    {
        let header : Header;

        header = Header::new(::config::NETWORK,
                             "verack".to_string(),
                             0u32,
                             ::crypto::checksum([]));

        header.serialize()
    }

    pub fn unserialize(_data : &Vec<u8>) -> VerAck
    {
        VerAck::new()
    }
}

impl Show for VerAck
{
    fn fmt(&self, f : &mut Formatter) -> Result<(), ::std::fmt::FormatError>
    {
        let width = if f.width.is_some() { f.width.unwrap() } else { 0 };
        let space = String::from_str(" ").repeat(width);

        write!(f, "{}VerAck", space)
    }
}
