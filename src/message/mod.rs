extern crate time;

use std::fmt::Show;
use std::fmt::Formatter;

use std::io::net::ip::SocketAddr;

pub mod header;
pub mod version;
pub mod versionack;
pub mod ping;
pub mod pong;
pub mod addresses;
pub mod inv;

pub enum Message
{
    MsgVersion(version::Version),
    MsgVersionAck(versionack::VersionAck),
    MsgPing(ping::Ping),
    MsgPong(pong::Pong),
    MsgAddresses(addresses::Addresses),
    MsgInv(inv::Inv),
}

pub struct NetAddr
{
    pub time     : Option<time::Timespec>,
    pub services : ::config::Services,
    pub addr     : Option<SocketAddr>
}

impl NetAddr
{
    pub fn new(time     : Option<time::Timespec>,
               services : ::config::Services,
               addr     : Option<SocketAddr>) -> NetAddr
    {
        NetAddr
        {
            time:     time,
            services: services,
            addr:     addr
        }
    }
}

impl Show for NetAddr
{
    fn fmt(&self, f : &mut Formatter) -> Result<(), ::std::fmt::FormatError>
    {
        match self.addr
        {
            Some(addr) => try!(write!(f, "{}", addr)),
            None       => try!(write!(f, "None"))
        };

        Ok(())
    }
}

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
        self.len()
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
