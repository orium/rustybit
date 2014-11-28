use std::fmt::Show;
use std::fmt::Formatter;

use message::header::Header;

pub struct Ping
{
    nounce : u64
}

#[allow(dead_code)]
impl Ping
{
    pub fn new(nounce : u64) -> Ping
    {
        Ping
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
                             "ping".to_string(),
                             msg.len() as u32,
                             ::crypto::checksum(msg.get().as_slice()));

        header.serialize() + msg.get()
    }

    pub fn unserialize(data : &Vec<u8>) -> Ping
    {
        let mut unmarshalling = ::marshalling::Unmarshalling::new(data);
        let nounce : u64;

        nounce = unmarshalling.read_uint64();

        Ping { nounce: nounce }
    }
}

impl Show for Ping
{
    fn fmt(&self, f : &mut Formatter) -> Result<(), ::std::fmt::Error>
    {
        let width = f.width().unwrap_or(0);
        let space = String::from_str(" ").repeat(width);

        write!(f,"{}Ping {}", space, self.nounce)
    }
}
