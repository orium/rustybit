use std::fmt::Show;
use std::fmt::Formatter;

/* TODO inside Header */
pub const HEADER_SIZE : uint = 24;

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

    pub fn get_command(&self) -> &String
    {
        &self.command
    }

    pub fn get_payload_size(&self) -> uint
    {
        self.len as uint
    }

    pub fn get_network(&self) -> u32
    {
        self.network
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

    pub fn unserialize(data : &Vec<u8>) -> Header
    {
        let mut unmarshalling = ::marshalling::Unmarshalling::new(data);
        let header : Header;

        assert!(data.len() >= 24);

        header = Header
        {
            network:  unmarshalling.read_uint32(),
            command:  unmarshalling.read_str12(),
            len:      unmarshalling.read_uint32(),
            checksum: unmarshalling.read_uint32()
        };

        header
    }
}

impl Show for Header
{
    fn fmt(&self, f : &mut Formatter) -> Result<(), ::std::fmt::Error>
    {
        try!(write!(f, "Network : {:08x}\n", self.network));
        try!(write!(f, "Command : {}\n", self.command));
        try!(write!(f, "Len     : {}\n", self.len));
        try!(write!(f, "Checksum: {:08x}", self.checksum));

        Ok(())
    }
}
