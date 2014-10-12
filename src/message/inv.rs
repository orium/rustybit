use std::fmt::Show;
use std::fmt::Formatter;

use message::header::Header;
use message::InvEntry;
use message::InvVect;

pub struct Inv
{
    vect : InvVect
}

#[allow(dead_code)]
impl Inv
{
    pub fn new() -> Inv
    {
        Inv
        {
            vect: InvVect::new()
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
                             "inv".to_string(),
                             msg.len() as u32,
                             ::crypto::checksum(&msg.get()));

        header.serialize() + msg.get()
    }

    pub fn unserialize(data : &Vec<u8>) -> Inv
    {
        let mut unmarshalling = ::marshalling::Unmarshalling::new(data);
        let vect : InvVect;

        vect = unmarshalling.read_invvect();

        assert!(unmarshalling.is_end());

        Inv
        {
            vect: vect
        }
    }
}

impl Show for Inv
{
    fn fmt(&self, f : &mut Formatter) -> Result<(), ::std::fmt::FormatError>
    {
        let width = if f.width.is_some() { f.width.unwrap() } else { 0 };
        let space = String::from_str(" ").repeat(width);

        try!(write!(f,"{}Inv:\n", space));

        // TODO this should be "{:4+space}"
        try!(write!(f,"{:8}", self.vect));

        Ok(())
    }
}
