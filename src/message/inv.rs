use std::fmt::Show;
use std::fmt::Formatter;

use message::header::Header;

use datatype::invvect::InvEntry;
use datatype::invvect::InvVect;

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

    pub fn get_invvect(&self) -> &InvVect
    {
        &self.vect
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
                             ::crypto::checksum(msg.get().as_slice()));

        header.serialize() + msg.get()
    }

    pub fn unserialize(data : &Vec<u8>) -> Inv
    {
        let mut unmarshalling = ::marshalling::Unmarshalling::new(data);
        let vect : InvVect;

        vect = unmarshalling.read_invvect();

        Inv
        {
            vect: vect
        }
    }
}

impl Show for Inv
{
    fn fmt(&self, f : &mut Formatter) -> Result<(), ::std::fmt::Error>
    {
        let width = f.width().unwrap_or(0);
        let space = String::from_str(" ").repeat(width);

        try!(write!(f,"{}Inv:\n", space));

        // TODO this should be "{:2+space}"
        try!(write!(f,"{:6}", self.vect));

        Ok(())
    }
}
