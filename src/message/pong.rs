use std::fmt::Show;
use std::fmt::Formatter;

use message::header::Header;

pub struct Pong
{
    nounce : u64
}

impl Pong
{
    pub fn new(nounce : u64) -> Pong
    {
        Pong
        {
            nounce: nounce
        }
    }

    pub fn get_nounce(&self) -> u64
    {
        self.nounce
    }

    pub fn serialize(&self) -> Vec<u8>
    {
        let mut msg = ::marshalling::Marshalling::new();
        let header : Header;

        msg.write_uint64(self.nounce);

        header = Header::new(::config::NETWORK,
                             "pong".to_string(),
                             msg.len() as u32,
                             ::crypto::checksum(msg.get().as_slice()));

        header.serialize() + msg.get()
    }

    pub fn unserialize(data : &Vec<u8>) -> Pong
    {
        let mut unmarshalling = ::marshalling::Unmarshalling::new(data);
        let nounce : u64;

        nounce = unmarshalling.read_uint64();

        Pong { nounce: nounce }
    }
}

impl Show for Pong
{
    fn fmt(&self, f : &mut Formatter) -> Result<(), ::std::fmt::FormatError>
    {
        let width = if f.width.is_some() { f.width.unwrap() } else { 0 };
        let space = String::from_str(" ").repeat(width);

        write!(f, "{}Pong {}", space, self.nounce)
    }
}
