use std::fmt::Show;
use std::fmt::Formatter;

use datatype::hash::Hash;

#[deriving(Clone)]
pub enum InvEntryType
{
    Error,
    MsgTx,
    MsgBlock
}

#[deriving(Clone)]
pub struct InvEntry
{
    pub typ  : InvEntryType,
    pub hash : Hash
}

impl Show for InvEntry
{
    fn fmt(&self, f : &mut Formatter) -> Result<(), ::std::fmt::Error>
    {
        match self.typ
        {
            InvEntryType::Error    => try!(write!(f, "ERROR ")),
            InvEntryType::MsgTx    => try!(write!(f, "TX    ")),
            InvEntryType::MsgBlock => try!(write!(f, "BLOCK "))
        }

        write!(f,"{}",self.hash)
    }
}

#[deriving(Clone)]
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

impl Index<uint, InvEntry> for InvVect
{
    fn index(&self, index: &uint) -> &InvEntry
    {
        &self.entries[*index]
    }
}

impl Show for InvVect
{
    fn fmt(&self, f : &mut Formatter) -> Result<(), ::std::fmt::Error>
    {
        let width = f.width().unwrap_or(0);
        let space = String::from_str(" ").repeat(width);

        for i in range(0,self.entries.len())
        {
            try!(write!(f,"{}#{:03} {}{}",space,i+1,self.entries[i],
                 if i == self.entries.len()-1 { "" } else { "\n" }));
        }

        Ok(())
    }
}
