extern crate time;

use std::io::net::ip::SocketAddr;

pub mod header;
pub mod version;
pub mod versionack;

pub enum Message
{
    MsgVersion(version::Version),
    MsgVersionAck(versionack::VersionAck)
}

pub struct NetAddr
{
    pub time     : Option<time::Tm>,
    pub services : ::config::Services,
    pub addr     : Option<SocketAddr>
}

impl NetAddr
{
    pub fn new(time     : Option<time::Tm>,
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

/* TODO: NetAddr Show */
