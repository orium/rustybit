extern crate time;

use std::fmt::Show;
use std::fmt::Formatter;

pub enum Message
{
    MsgVersion(Version),
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

pub struct Version
{
    version     : String,
    time        : time::Tm,
    best_height : u32,
    nounce      : u64
}

impl Version
{
    pub fn new(version : String, best_height : u32) -> Version
    {
        // TODO: rnd should be a global variable. Is that possible in rust?
        // let mut rng : ::std::rand::OsRng = ::std::rand::OsRng::new().unwrap();

        Version
        {
            version:     version,
            time:        time::now_utc(),
            best_height: best_height,
            nounce:      0xababeface // TODO rng.gen()
        }
    }

    pub fn unserialize(data : &Vec<u8>) -> Version
    {
        let mut unmarshalling = ::marshalling::Unmarshalling::new(data);
        let version : String;
        let time : time::Tm;
        let best_height : u32;
        let nounce : u64;

        unmarshalling.read_uint32();
        unmarshalling.read_uint64();
        unmarshalling.skip(8); /* TODO */ /* timestamp */
        time = time::empty_tm();
        unmarshalling.skip(26); /* TODO */ /* recv addr */
        unmarshalling.skip(26); /* TODO */ /* send addr */
        nounce=unmarshalling.read_uint64();
        version=unmarshalling.read_varstr();
        best_height=unmarshalling.read_uint32();
        unmarshalling.read_bool();

        assert!(unmarshalling.is_end());

        Version
        {
            version:     version,
            time:        time,
            best_height: best_height,
            nounce:      nounce
        }
    }

    // TODO: create a trait for serialization
    pub fn serialize(&self) -> Vec<u8>
    {
        let mut msg = ::marshalling::Marshalling::new();
        let header : Header;

        /* TODO everything here should be in the struct.
         * eg. self.services
         */
        msg.write_uint32(::config::PROTOCOL_VERSION);
        msg.write_uint64(::config::SERVICES as u64);
        msg.write_timestamp(self.time);
        msg.write_netaddr(None,::config::SERVICES,None); /* recv addr */
        msg.write_netaddr(None,::config::SERVICES,None); /* send addr */
        msg.write_uint64(self.nounce);
        msg.write_varstr(&self.version);
        msg.write_uint32(self.best_height);
        msg.write_bool(true); /* relay transactions */

        header = Header::new(::config::MAIN_NET,
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
        try!(write!(f, "Version : {}\n", self.version));

        Ok(())
    }
}
