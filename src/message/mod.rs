extern crate time;

use std::fmt::Show;
use std::fmt::Formatter;

use std::io::net::ip::SocketAddr;

pub mod header;
pub mod version;
pub mod versionack;
pub mod ping;
pub mod pong;

pub enum Message
{
    MsgVersion(version::Version),
    MsgVersionAck(versionack::VersionAck),
    MsgPing(ping::Ping),
    MsgPong(pong::Pong)
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
