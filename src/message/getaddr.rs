use std::fmt::Show;
use std::fmt::Formatter;

use message::header::Header;

pub struct GetAddr;

#[allow(dead_code)]
impl GetAddr
{
    pub fn new() -> GetAddr
    {
        GetAddr
    }

    pub fn serialize(&self) -> Vec<u8>
    {
        let msg = ::marshalling::Marshalling::new();
        let header : Header;

        header = Header::new(::config::NETWORK,
                             "getaddr".to_string(),
                             msg.len() as u32,
                             ::crypto::checksum(&msg.get()));

        header.serialize()
    }

    pub fn unserialize(_data : &Vec<u8>) -> GetAddr
    {
        GetAddr::new()
    }
}

impl Show for GetAddr
{
    fn fmt(&self, f : &mut Formatter) -> Result<(), ::std::fmt::FormatError>
    {
        let width = if f.width.is_some() { f.width.unwrap() } else { 0 };
        let space = String::from_str(" ").repeat(width);

        write!(f,"{}GetAddr", space)
    }
}
