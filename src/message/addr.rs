use std::fmt::Show;
use std::fmt::Formatter;

use message::header::Header;

use datatype::netaddr::NetAddr;

pub const MSG_ADDR_MAX : uint = 1000;

pub struct Addr
{
    addresses : Vec<NetAddr>
}

#[allow(dead_code)]
impl Addr
{
    pub fn new() -> Addr
    {
        Addr
        {
            addresses: Vec::new()
        }
    }

    pub fn from_addrs(addrs : &Vec<NetAddr>) -> Addr
    {
        Addr
        {
            addresses: addrs.clone()
        }
    }

    pub fn add(&mut self, addr : NetAddr)
    {
        self.addresses.push(addr);

        assert!(self.addresses.len() <= MSG_ADDR_MAX);
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
            msg.write_netaddr(addr,true);
        }

        header = Header::new(::config::NETWORK,
                             "addr".to_string(),
                             msg.len() as u32,
                             ::crypto::checksum(msg.get().as_slice()));

        header.serialize() + msg.get()
    }

    pub fn unserialize(data : &Vec<u8>) -> Addr
    {
        let mut unmarshalling = ::marshalling::Unmarshalling::new(data);
        let mut addresses : Addr = Addr::new();
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

impl Show for Addr
{
    fn fmt(&self, f : &mut Formatter) -> Result<(), ::std::fmt::Error>
    {
        let width = f.width().unwrap_or(0);
        let space = String::from_str(" ").repeat(width);

        try!(write!(f,"{}Addr:\n", space));

        for i in range(0,self.addresses.len())
        {
            try!(write!(f,"{}    Address #{} {}{}",space,i+1,self.addresses[i],
                 if i == self.addresses.len()-1 { "" } else { "\n" }));
        }

        Ok(())
    }
}
