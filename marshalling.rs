extern crate time;

use std::io::net::ip::SocketAddr;
use std::io::net::ip::{Ipv4Addr, Ipv6Addr};

pub struct Marshalling
{
    buf: Vec<u8>
}

impl Marshalling
{
    pub fn new() -> Marshalling
    {
        Marshalling { buf: Vec::new() }
    }

    pub fn write(&mut self, d : &[u8])
    {
        self.buf.push_all(d);
    }

    pub fn write_uint16(&mut self, v : u16)
    {
        for i in range(0u,2)
        {
            self.buf.push(((v>>8*i)&0xff) as u8);
        }
    }

    pub fn write_uint32(&mut self, v : u32)
    {
        for i in range(0u,4)
        {
            self.buf.push(((v>>8*i)&0xff) as u8);
        }
    }

    pub fn write_uint64(&mut self, v : u64)
    {
        for i in range(0u,8)
        {
            self.buf.push(((v>>8*i)&0xff) as u8);
        }
    }

    pub fn write_int32(&mut self, v : i32)
    {
        assert!(v >= 0); /* TODO: write negative values */

        self.write_uint32(v as u32);
    }

    pub fn write_int64(&mut self, v : i64)
    {
        assert!(v >= 0); /* TODO: write negative values */

        self.write_uint64(v as u64);
    }

    pub fn write_bool(&mut self, b : bool)
    {
        self.buf.push(if b { 1u8 } else { 0u8 });
    }

    pub fn write_varint(&mut self, v : u64)
    {
        match v
        {
            0u64           ... 252u64             => {
                self.buf.push(v as u8);
            },
            253u64         ... 0xffffu64             => {
                self.buf.push(253u8);
                self.write_uint16(v as u16);
            },
            0x10000u64     ... 0xffffffffu64         => {
                self.buf.push(254u8);
                self.write_uint32(v as u32);
            },
            0x100000000u64 ... 0xffffffffffffffffu64 => {
                self.buf.push(255u8);
                self.write_uint64(v);
            },
            _ => unreachable!()
        }
    }

    pub fn write_str12(&mut self, str : &String)
    {
        let bytes : &[u8] = str.as_bytes();

        assert!(bytes.len() <= 12);

        for i in range(0u,bytes.len())
        {
            self.buf.push(bytes[i]);
        }

        for _ in range(str.len(),12)
        {
            self.buf.push(0x00);
        }
    }

    pub fn write_varstr(&mut self, str : &String)
    {
        let bytes : &[u8] = str.as_bytes();

        self.write_varint(bytes.len() as u64);

        for i in range(0u,bytes.len())
        {
            self.buf.push(bytes[i]);
        }
    }

    pub fn write_timestamp(&mut self, time : time::Tm)
    {
        self.write_int64(time.to_timespec().sec);
    }

    // TODO maybe we should have a type netaddr
    pub fn write_netaddr(&mut self,
                         time     : Option<time::Tm>,
                         services : ::config::Services,
                         addr     : Option<SocketAddr>)
    {
        if time.is_some()
        {
            self.write_timestamp(time.unwrap());
        }

        self.write_uint64(services as u64);

        match addr
        {
            Some(addr) =>
            {
                match addr.ip
                {
                    Ipv4Addr(b3, b2, b1, b0) =>
                    {
                        self.write([0u8, ..10]);
                        self.write([0xffu8, ..2]);
                        self.write([b3,b2,b1,b0]);
                    },
                    Ipv6Addr(..) => unimplemented!() /* TODO ipv6 */
                };

                self.write_uint16(addr.port);
            }
            None => {
                self.write([0u8, ..10]);
                self.write([0xffu8, ..2]);
                self.write([0u8, ..4]); /* ip */
                self.write_uint16(0);   /* port */
            }
        };
    }

    pub fn get(&self) -> Vec<u8>
    {
        self.buf.clone()
    }

    pub fn len(&self) -> uint
    {
        self.buf.len()
    }
}

// TODO: Unmarchalling
pub struct Unmarshalling
{
    buf: Vec<u8>,
    pos: uint
}

impl Unmarshalling
{
    pub fn new(data : &Vec<u8>) -> Unmarshalling
    {
        Unmarshalling { buf: data.clone(),
                        pos: 0}
    }

    pub fn read(&mut self, d : &mut [u8])
    {
        assert!(self.pos+d.len() <= self.buf.len());

        for i in range(0,d.len())
        {
            d[i] = self.buf[self.pos];
            self.pos += 1;
        }
    }

    pub fn read_uint16(&mut self) -> u16
    {
        let mut v : u16 = 0;

        assert!(self.pos+2 <= self.buf.len());

        for i in range(0u,2)
        {
            v |= self.buf[self.pos] as u16 << 8*i;
            self.pos += 1;
        }

        v
    }

    pub fn read_uint32(&mut self) -> u32
    {
        let mut v : u32 = 0;

        assert!(self.pos+4 <= self.buf.len());

        for i in range(0u,4)
        {
            v |= self.buf[self.pos] as u32 << 8*i;
            self.pos += 1;
        }

        v
    }

    pub fn read_uint64(&mut self) -> u64
    {
        let mut v : u64 = 0;

        assert!(self.pos+8 <= self.buf.len());

        for i in range(0u,8)
        {
            v |= self.buf[self.pos] as u64 << 8*i;
            self.pos += 1;
        }

        v
    }

    /* TODO
    read_int64
    read_int32
     */

    pub fn read_bool(&mut self) -> bool
    {
        let b : u8;

        assert!(self.pos+1 <= self.buf.len());

        b = self.buf[self.pos];

        self.pos += 1;

        if b == 0u8 { false } else { true }
    }

    /*
    read_varint
     */

    pub fn read_str12(&mut self) -> String
    {
        let mut str : String = String::new();

        assert!(self.pos+12 <= self.buf.len());

        for i in range(0u, 12)
        {
            if self.buf[self.pos+i] == 0u8
            {
                break;
            }

            str.push(self.buf[self.pos+i] as char);
        }

        self.pos += 12;

        str
    }

    /* TODO
    read_varstr
    read_timestamp
    read_netaddr
     */
}
