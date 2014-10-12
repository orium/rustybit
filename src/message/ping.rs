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
    pub fn new() -> Ping
    {
        Ping
        {
            nounce: 0xababeface // TODO use OsRnd
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
                             ::crypto::checksum(&msg.get()));

        header.serialize() + msg.get()
    }

    pub fn unserialize(data : &Vec<u8>) -> Ping
    {
        let mut unmarshalling = ::marshalling::Unmarshalling::new(data);
        let nounce : u64;

        nounce = unmarshalling.read_uint64();

        assert!(unmarshalling.is_end());

        Ping { nounce: nounce }
    }
}

impl Show for Ping
{
    fn fmt(&self, f : &mut Formatter) -> Result<(), ::std::fmt::FormatError>
    {
        let width = if f.width.is_some() { f.width.unwrap() } else { 0 };
        let space = String::from_str(" ").repeat(width);

        write!(f,"{}Ping {}", space, self.nounce)
    }
}
