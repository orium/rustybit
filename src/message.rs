extern crate time;

use std::fmt::Show;
use std::fmt::Formatter;

use std::io::net::ip::SocketAddr;

pub enum Message
{
    MsgVersion(Version),
    MsgVersionAck(VersionAck)
}

pub struct Header
{
    network  : u32,
    command  : String,
    len      : u32,
    checksum : u32
}

impl Header
{
    pub fn new(network  : u32,
               command  : String,
               len      : u32,
               checksum : u32) -> Header
    {
        Header
        {
            network:  network,
            command:  command,
            len:      len,
            checksum: checksum
        }
    }

    pub fn get_checksum(&self) -> u32
    {
        self.checksum
    }

    pub fn get_command<'a>(&'a self) -> &'a String
    {
        &self.command
    }

    pub fn get_payload_len(&self) -> uint
    {
        self.len as uint
    }

    pub fn unserialize(data : &Vec<u8>) -> Header
    {
        let mut unmarshalling = ::marshalling::Unmarshalling::new(data);
        let header : Header;

        assert!(data.len() == 24);

        header = Header
        {
            network:  unmarshalling.read_uint32(),
            command:  unmarshalling.read_str12(),
            len:      unmarshalling.read_uint32(),
            checksum: unmarshalling.read_uint32()
        };

        assert!(unmarshalling.is_end());

        header
    }

    pub fn serialize(&self) -> Vec<u8>
    {
        let mut header = ::marshalling::Marshalling::new();

        header.write_uint32(self.network);
        header.write_str12(&self.command);
        header.write_uint32(self.len);
        header.write_uint32(self.checksum);

        header.get()
    }
}

impl Show for Header
{
    fn fmt(&self, f : &mut Formatter) -> Result<(), ::std::fmt::FormatError>
    {
        try!(write!(f, "Network : {:08x}\n", self.network));
        try!(write!(f, "Command : {}\n", self.command));
        try!(write!(f, "Len     : {}\n", self.len));
        try!(write!(f, "Checksum: {:08x}", self.checksum));

        Ok(())
    }
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

pub struct Version
{
    proto_ver   : u32,
    services    : ::config::Services,
    version     : String,
    time        : time::Tm,
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
        // TODO: rnd should be a global variable. Is that possible in rust?
        // let mut rng : ::std::rand::OsRng = ::std::rand::OsRng::new().unwrap();

        Version
        {
            proto_ver:   ::config::PROTOCOL_VERSION,
            services:    ::config::SERVICES,
            version:     version,
            time:        time::now_utc(),
            addr_recv:   NetAddr::new(None,::config::SERVICES,None),
            addr_send:   NetAddr::new(None,::config::SERVICES,None),
            best_height: best_height,
            nounce:      0xababeface, // TODO rng.gen()
            relay:       true
        }
    }

    pub fn unserialize(data : &Vec<u8>) -> Version
    {
        let mut unmarshalling = ::marshalling::Unmarshalling::new(data);
        let proto_ver : u32;
        let services : ::config::Services;
        let version : String;
        let time : time::Tm;
        let addr_recv: NetAddr;
        let addr_send: NetAddr;
        let best_height : u32;
        let nounce : u64;
        let relay : bool;

        proto_ver = unmarshalling.read_uint32();
        services = unmarshalling.read_uint64();
        time = time::empty_tm();
        unmarshalling.skip(8);  /* TODO  timestamp */
        addr_recv = NetAddr::new(None,::config::SERVICES,None);
        unmarshalling.skip(26); /* TODO   recv addr */
        addr_send = NetAddr::new(None,::config::SERVICES,None);
        unmarshalling.skip(26); /* TODO   send addr */
        nounce = unmarshalling.read_uint64();
        version = unmarshalling.read_varstr();
        best_height = unmarshalling.read_uint32();
        relay = unmarshalling.read_bool();

        assert!(services == ::config::None as u64
                || services == ::config::NodeNetwork as u64);
        assert!(unmarshalling.is_end());

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

    // TODO: create a trait for serialization
    pub fn serialize(&self) -> Vec<u8>
    {
        let mut msg = ::marshalling::Marshalling::new();
        let header : Header;

        msg.write_uint32(self.proto_ver);
        msg.write_uint64(self.services);
        msg.write_timestamp(self.time);
        msg.write_netaddr(&self.addr_recv);
        msg.write_netaddr(&self.addr_send);
        msg.write_uint64(self.nounce);
        msg.write_varstr(&self.version);
        msg.write_uint32(self.best_height);
        msg.write_bool(self.relay);

        header = Header::new(::config::NETWORK,
                             "version".to_string(),
                             msg.len() as u32,
                             ::crypto::checksum(&msg.get()));

        header.serialize() + msg.get()
    }
}

impl Show for Version
{
    fn fmt(&self, f : &mut Formatter) -> Result<(), ::std::fmt::FormatError>
    {
        try!(write!(f, "Proto ver   : {}\n", self.proto_ver));
        try!(write!(f, "Version     : {}\n", self.version));
        try!(write!(f, "Best height : {}\n", self.best_height));
        try!(write!(f, "Relay       : {}", self.relay));

        Ok(())
    }
}

pub struct VersionAck;

impl VersionAck
{
    pub fn new() -> VersionAck
    {
        VersionAck
    }

    pub fn serialize(&self) -> Vec<u8>
    {
        let header : Header;

        header = Header::new(::config::NETWORK,
                             "verack".to_string(),
                             0u32,
                             ::crypto::checksum(&Vec::new()));

        header.serialize()
    }

    pub fn unserialize(data : &Vec<u8>) -> VersionAck
    {
        assert!(data.len() == 0);

        VersionAck::new()
    }
}

impl Show for VersionAck
{
    fn fmt(&self, f : &mut Formatter) -> Result<(), ::std::fmt::FormatError>
    {
        write!(f, "VersionAck")
    }
}
