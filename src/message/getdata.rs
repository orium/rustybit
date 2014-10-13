use std::fmt::Show;
use std::fmt::Formatter;

use message::header::Header;

use datatype::invvect::InvVect;
use datatype::invvect::InvEntry;

pub struct GetData
{
    vect : InvVect
}

#[allow(dead_code)]
impl GetData
{
    pub fn new() -> GetData
    {
        GetData
        {
            vect: InvVect::new()
        }
    }

    pub fn from_inv(invvect : &InvVect) -> GetData
    {
        GetData
        {
            vect: invvect.clone()
        }
    }

    pub fn add(&mut self, entry : InvEntry)
    {
        self.vect.add(entry);

        assert!(self.vect.len() <= 50000);
    }

    pub fn serialize(&self) -> Vec<u8>
    {
        let mut msg = ::marshalling::Marshalling::new();
        let header : Header;

        msg.write_invvect(&self.vect);

        header = Header::new(::config::NETWORK,
                             "getdata".to_string(),
                             msg.len() as u32,
                             ::crypto::checksum(&msg.get()));

        header.serialize() + msg.get()
    }

    pub fn unserialize(data : &Vec<u8>) -> GetData
    {
        let mut unmarshalling = ::marshalling::Unmarshalling::new(data);
        let vect : InvVect;

        vect = unmarshalling.read_invvect();

        assert!(unmarshalling.is_end());

        GetData
        {
            vect: vect
        }
    }
}

impl Show for GetData
{
    fn fmt(&self, f : &mut Formatter) -> Result<(), ::std::fmt::FormatError>
    {
        let width = if f.width.is_some() { f.width.unwrap() } else { 0 };
        let space = String::from_str(" ").repeat(width);

        try!(write!(f,"{}GetData:\n", space));

        // TODO this should be "{:4+space}"
        try!(write!(f,"{:8}", self.vect));

        Ok(())
    }
}
