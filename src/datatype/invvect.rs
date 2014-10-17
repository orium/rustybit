use std::fmt::Show;
use std::fmt::Formatter;

use std::clone::Clone;

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

    pub fn len(&self) -> uint
    {
        self.entries.len()
    }

    pub fn iter(&self) -> ::std::slice::Items<InvEntry>
    {
        self.entries.iter()
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
