use std::fmt::Show;
use std::fmt::Formatter;

use std::clone::Clone;

use message::header::Header;

pub enum InvEntryType
{
    Error,
    MsgTx,
    MsgBlock
}

pub struct InvEntry
{
    pub typ  : InvEntryType,
    pub hash : Vec<u8>
}

impl Show for InvEntry
{
    fn fmt(&self, f : &mut Formatter) -> Result<(), ::std::fmt::FormatError>
    {
        match self.typ
        {
            Error    => try!(write!(f, "ERROR ")),
            MsgTx    => try!(write!(f, "TX    ")),
            MsgBlock => try!(write!(f, "BLOCK "))
        }

        try!(write!(f, "{}", ::crypto::hash_to_hexstr(&self.hash)));

        Ok(())
    }
}


impl Clone for InvEntry
{
    fn clone(&self) -> InvEntry
    {
        InvEntry
        {
            typ : self.typ,
            hash: self.hash.clone()
        }
    }
}

pub struct InvVect
{
    entries : Vec<InvEntry>
}

#[allow(dead_code)]
impl InvVect
{
    pub fn new() -> InvVect
    {
        InvVect
        {
            entries: Vec::new()
        }
    }

    pub fn add(&mut self, entry : InvEntry)
    {
        self.entries.push(entry)
    }

    pub fn get(&self, i : uint) -> &InvEntry
    {
        &self.entries[i]
    }

    pub fn len(&self) -> uint
    {
        self.entries.len()
    }
}

impl Show for InvVect
{
    fn fmt(&self, f : &mut Formatter) -> Result<(), ::std::fmt::FormatError>
    {
        let width = if f.width.is_some() { f.width.unwrap() } else { 0 };
        let space = String::from_str(" ").repeat(width);

        for i in range(0,self.entries.len())
        {
            try!(write!(f,"{}#{:03} {}{}",space,i+1,self.entries[i],
                 if i == self.entries.len()-1 { "" } else { "\n" }));
        }

        Ok(())
    }
}

impl Clone for InvVect
{
    fn clone(&self) -> InvVect
    {
        InvVect
        {
            entries: self.entries.clone()
        }
    }
}


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
