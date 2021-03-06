extern crate time;

use std::rand::Rng;

use std::fmt::Show;
use std::fmt::Formatter;

use message::header::Header;
use datatype::netaddr::NetAddr;

#[deriving(Clone)]
pub struct Version
{
    proto_ver   : u32,
    services    : ::config::Services,
    version     : String,
    time        : time::Timespec,
    addr_recv   : NetAddr,
    addr_send   : NetAddr,
    best_height : u32,
    nounce      : u64,
    relay       : bool /* see BIP0037 */
}

impl Version
{
    pub fn new(version : String, best_height : u32) -> Version
    {
        Version
        {
            proto_ver:   ::config::PROTOCOL_VERSION,
            services:    ::config::SERVICES,
            version:     version,
            time:        time::now_utc().to_timespec(),
            addr_recv:   NetAddr::new(None,::config::SERVICES,None),
            addr_send:   NetAddr::new(None,::config::SERVICES,None),
            best_height: best_height,
            nounce:      ::crypto::rng().gen(),
            relay:       true
        }
    }

    pub fn get_protocol_version(&self) -> u32
    {
        self.proto_ver
    }

    pub fn get_addr_send(&self) -> &NetAddr
    {
        &self.addr_send
    }

    // TODO: create a trait for serialization (serialize::Encodable?)
    pub fn serialize(&self) -> Vec<u8>
    {
        let mut msg = ::marshalling::Marshalling::new();
        let header : Header;

        msg.write_uint32(self.proto_ver);
        msg.write_uint64(self.services);
        msg.write_timestamp64(self.time);
        msg.write_netaddr(&self.addr_recv,false);
        msg.write_netaddr(&self.addr_send,false);
        msg.write_uint64(self.nounce);
        msg.write_varstr(&self.version);
        msg.write_uint32(self.best_height);
        msg.write_bool(self.relay);

        header = Header::new(::config::NETWORK,
                             "version".to_string(),
                             msg.len() as u32,
                             ::crypto::checksum(msg.get().as_slice()));

        header.serialize() + msg.get()
    }

    pub fn unserialize(data : &Vec<u8>, size : uint) -> Version
    {
        let mut unmarshalling = ::marshalling::Unmarshalling::new(data);
        let proto_ver : u32;
        let services : ::config::Services;
        let version : String;
        let time : time::Timespec;
        let addr_recv: NetAddr;
        let addr_send: NetAddr;
        let best_height : u32;
        let nounce : u64;
        let relay : bool;

        proto_ver = unmarshalling.read_uint32();
        services = unmarshalling.read_uint64();
        time = unmarshalling.read_timestamp64();
        addr_recv = unmarshalling.read_netaddr(false);
        addr_send = unmarshalling.read_netaddr(false);
        nounce = unmarshalling.read_uint64();
        version = unmarshalling.read_varstr();
        best_height = unmarshalling.read_uint32();

        relay = if unmarshalling.consumed() < size { unmarshalling.read_bool() }
                                                   else { true };

        assert!(services == ::config::Service::NoService as u64
                || services == ::config::Service::NodeNetwork as u64);

        Version
        {
            proto_ver:   proto_ver,
            services:    services,
            version:     version,
            time:        time,
            addr_recv:   addr_recv,
            addr_send:   addr_send,
            best_height: best_height,
            nounce:      nounce,
            relay:       relay
        }
    }
}

impl Show for Version
{
    fn fmt(&self, f : &mut Formatter) -> Result<(), ::std::fmt::Error>
    {
        let width = f.width().unwrap_or(0);
        let space = String::from_str(" ").repeat(width);

        try!(write!(f, "{}Proto ver  : {}\n", space, self.proto_ver));
        try!(write!(f, "{}Version    : {}\n", space, self.version));
        try!(write!(f, "{}Addr recv  : {}\n", space, self.addr_recv));
        try!(write!(f, "{}Addr send  : {}\n", space, self.addr_send));
        try!(write!(f, "{}Best height: {}\n", space, self.best_height));
        try!(write!(f, "{}Relay      : {}", space, self.relay));

        Ok(())
    }
}
