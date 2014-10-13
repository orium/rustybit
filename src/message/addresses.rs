use std::fmt::Show;
use std::fmt::Formatter;

use message::header::Header;

use datatype::netaddr::NetAddr;

pub struct Addresses
{
    addresses : Vec<NetAddr>
}

#[allow(dead_code)]
impl Addresses
{
    pub fn new() -> Addresses
    {
        Addresses
        {
            addresses: Vec::new()
        }
    }

    pub fn add(&mut self, addr : NetAddr)
    {
        self.addresses.push(addr);

        assert!(self.addresses.len() <= 1000);
    }

    pub fn get_addresses(&self) -> &Vec<NetAddr>
    {
        &self.addresses
    }

    pub fn serialize(&self) -> Vec<u8>
    {
        let mut msg = ::marshalling::Marshalling::new();
        let header : Header;

        msg.write_varint(self.addresses.len() as u64);

        for addr in self.addresses.iter()
        {
            msg.write_netaddr(addr);
        }

        header = Header::new(::config::NETWORK,
                             "addr".to_string(),
                             msg.len() as u32,
                             ::crypto::checksum(&msg.get()));

        header.serialize() + msg.get()
    }

    pub fn unserialize(data : &Vec<u8>) -> Addresses
    {
        let mut unmarshalling = ::marshalling::Unmarshalling::new(data);
        let mut addresses : Addresses = Addresses::new();
        let count : u64;

        count = unmarshalling.read_varint();

        assert!(count <= 1000);

        for _ in range(0,count)
        {
            let netaddr = unmarshalling.read_netaddr(true);

            addresses.add(netaddr);
        }

        addresses
    }
}

impl Show for Addresses
{
    fn fmt(&self, f : &mut Formatter) -> Result<(), ::std::fmt::FormatError>
    {
        let width = if f.width.is_some() { f.width.unwrap() } else { 0 };
        let space = String::from_str(" ").repeat(width);

        try!(write!(f,"{}Addresses:\n", space));

        for i in range(0,self.addresses.len())
        {
            try!(write!(f,"{}    Address #{} {}{}",space,i+1,self.addresses[i],
                 if i == self.addresses.len()-1 { "" } else { "\n" }));
        }

        Ok(())
    }
}
