extern crate time;

use std::fmt::Show;
use std::fmt::Formatter;

use std::io::net::ip::SocketAddr;
use std::io::net::ip::Ipv4Addr;
use std::io::net::ip::Ipv6Addr;

#[deriving(Clone)]
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

    pub fn is_valid_addr(&self) -> bool
    {
        match self.addr
        {
            Some(SocketAddr { ip: _, port: 0 })                 => false,
            Some(SocketAddr { ip: Ipv6Addr(..), port: _ })      => false, /* TODO */
            Some(SocketAddr { ip: Ipv4Addr(0,0,0,0), port: _ }) => false,
            Some(SocketAddr { ip: Ipv4Addr(..), port: _ })      => true,
            None                                                => false
        }
    }
}

impl Show for NetAddr
{
    fn fmt(&self, f : &mut Formatter) -> Result<(), ::std::fmt::Error>
    {
        match self.addr
        {
            Some(addr) => try!(write!(f, "{}", addr)),
            None       => try!(write!(f, "None"))
        };

        Ok(())
    }
}
