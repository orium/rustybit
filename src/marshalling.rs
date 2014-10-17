extern crate time;

use std::io::net::ip::SocketAddr;
use std::io::net::ip::{IpAddr, Ipv4Addr, Ipv6Addr};

use datatype::netaddr::NetAddr;
use datatype::invvect::InvVect;

static VARSTR_MAX_LENGTH : uint = 256;
static VARSTR_SAFE_CHARS : &'static str
    = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ01234567890 .,;_/:?@";

pub struct Marshalling
{
    buf: Vec<u8>
}

#[allow(dead_code)]
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

    pub fn write_uint8(&mut self, v : u8)
    {
        self.buf.push(v);
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
            0u64           ... 252u64                => {
                self.write_uint8(v as u8);
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
        assert!(Unmarshalling::sanitize_string(str) == *str);

        for b in bytes.iter()
        {
            self.buf.push(*b);
        }

        for _ in range(str.len(),12)
        {
            self.buf.push(0x00);
        }
    }

    pub fn write_varstr(&mut self, str : &String)
    {
        let bytes : &[u8] = str.as_bytes();

        println!(".{}.{}.",Unmarshalling::sanitize_string(str),*str);

        assert!(Unmarshalling::sanitize_string(str) == *str);

        self.write_varint(bytes.len() as u64);

        for b in bytes.iter()
        {
            self.buf.push(*b);
        }
    }

    pub fn write_timestampu32(&mut self, time : time::Timespec)
    {
        self.write_uint32(time.sec as u32);
    }

    pub fn write_timestamp64(&mut self, time : time::Timespec)
    {
        self.write_int64(time.sec);
    }

    pub fn write_netaddr(&mut self, netaddr : &NetAddr)
    {
        if netaddr.time.is_some()
        {
            self.write_timestampu32(netaddr.time.unwrap());
        }

        self.write_uint64(netaddr.services as u64);

        match netaddr.addr
        {
            Some(addr) =>
            {
                match addr.ip
                {
                    Ipv4Addr(b3, b2, b1, b0) =>
                    {
                        self.write([0x00u8, ..10]);
                        self.write([0xffu8, ..2]);
                        self.write([b3,b2,b1,b0]);
                    },
                    Ipv6Addr(..) => unimplemented!() /* TODO ipv6 */
                };

                /* port is encoded in network order (big endian) */
                self.write(&[(addr.port>>8) as u8,
                             (addr.port&0xff) as u8]);
            }
            None =>
            {
                self.write([0u8, ..10]);
                self.write([0xffu8, ..2]);
                self.write([0u8, ..4]); /* ip */
                self.write_uint16(0);   /* port */
            }
        };
    }

    pub fn write_hash(&mut self, hash : &Vec<u8>)
    {
        assert!(hash.len() == 32);

        for i in range(0u,32).rev()
        {
            self.buf.push(hash[i]);
        }
    }

    pub fn write_invvect(&mut self, invvec : &InvVect)
    {
        self.write_varint(invvec.len() as u64);

        for entry in invvec.iter()
        {
            self.write_uint32(entry.typ as u32);
            self.write_hash(&entry.hash);
        }
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

pub struct Unmarshalling
{
    buf: Vec<u8>,
    pos: uint
}

#[allow(dead_code)]
impl Unmarshalling
{
    pub fn new(data : &Vec<u8>) -> Unmarshalling
    {
        Unmarshalling
        {
            buf: data.clone(),
            pos: 0
        }
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

    pub fn skip(&mut self, s : uint)
    {
        self.pos += s;
    }

    pub fn read_uint8(&mut self) -> u8
    {
        let v : u8;

        v = self.buf[self.pos];
        self.pos += 1;

        v
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

    pub fn read_int32(&mut self) -> i32
    {
        assert!(self.pos+4 <= self.buf.len());

        /* TODO: read negative values */
        self.read_uint32() as i32
    }

    pub fn read_int64(&mut self) -> i64
    {
        assert!(self.pos+8 <= self.buf.len());

        /* TODO: read negative values */
        self.read_uint64() as i64
    }

    pub fn read_bool(&mut self) -> bool
    {
        let b : u8;

        assert!(self.pos+1 <= self.buf.len());

        b = self.buf[self.pos];

        self.pos += 1;

        if b == 0u8 { false } else { true }
    }

    pub fn read_varint(&mut self) -> u64
    {
        let first : u8;

        assert!(self.pos+1 <= self.buf.len());

        first = self.buf[self.pos];

        self.pos += 1;

        match first
        {
            0u8 ... 252u8 => first as u64,
            253u8         => self.read_uint16() as u64,
            254u8         => self.read_uint32() as u64,
            255u8         => self.read_uint64(),
            _             => unreachable!()
        }
    }

    pub fn sanitize_string(str : &String) -> String
    {
        let mut r : String = String::new();

        for ch in str.as_slice().chars()
        {
            if VARSTR_SAFE_CHARS.contains_char(ch)
            {
                r.push(ch);
            }
        }

        r
    }

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

        /* TODO Should be all zeros after the first zero */

        Unmarshalling::sanitize_string(&str)
    }

    pub fn read_varstr(&mut self) -> String
    {
        let mut str : String = String::new();
        let len = self.read_varint() as uint;

        assert!(len <= VARSTR_MAX_LENGTH);
        assert!(self.pos+len <= self.buf.len());

        for _ in range(0,len)
        {
            str.push(self.buf[self.pos] as char);
            self.pos += 1;

            assert!(self.pos <= self.buf.len());
        }

        Unmarshalling::sanitize_string(&str)
    }

    pub fn read_timestampu32(&mut self) -> time::Timespec
    {
        assert!(self.pos+4 <= self.buf.len());

        time::Timespec::new(self.read_uint32() as i64,0)
    }

    pub fn read_timestamp64(&mut self) -> time::Timespec
    {
        assert!(self.pos+8 <= self.buf.len());

        time::Timespec::new(self.read_int64(),0)
    }

    pub fn read_netaddr(&mut self, with_time : bool) -> NetAddr
    {
        let time : Option<time::Timespec>;
        let services : ::config::Services;
        let addr : IpAddr;
        let port : u16;
        let mut socketaddr : Option<SocketAddr>;

        assert!(self.pos+if with_time { 30 } else { 24 } <= self.buf.len());

        time = if with_time { Some(self.read_timestampu32()) } else { None };

        services = self.read_uint64();

        for i in range(0u,10)
        {
            if self.buf[self.pos] != 0x00u8
            {
                println!("unimplemented: read_netaddr() IPv6"); /* TODO */

                self.skip(10-i+8);

                return NetAddr::new(time,services,None);
            }

            self.pos += 1;
        }

        assert!(self.buf[self.pos] == 0xffu8 && self.buf[self.pos+1] == 0xffu8);

        self.pos += 2; /* skip FF FF */

        addr = Ipv4Addr(self.buf[self.pos],
                        self.buf[self.pos+1],
                        self.buf[self.pos+2],
                        self.buf[self.pos+3]);

        self.pos += 4;

        /* port is encoded in network order (big endian) */
        port = ((self.buf[self.pos] as u16)<<8) | (self.buf[self.pos+1] as u16);

        self.pos += 2;

        socketaddr = None;

        if addr != Ipv4Addr(0,0,0,0)
        {
            socketaddr = Some(SocketAddr {
                ip:   addr,
                port: port,
            });
        }

        NetAddr::new(time,services,socketaddr)
    }

    pub fn read_hash(&mut self) -> Vec<u8>
    {
        let mut hash : Vec<u8> = Vec::new();

        assert!(self.pos+32 <= self.buf.len());

        for _ in range(0u,32)
        {
            hash.push(self.buf[self.pos]);
            self.pos += 1;
        }

        hash.reverse();

        hash
    }

    pub fn read_invvect(&mut self) -> InvVect
    {
        let mut invvec = InvVect::new();
        let count : u64;

        count = self.read_varint();

        assert!(count <= 50000);

        for _ in range(0,count)
        {
            let typ : ::datatype::invvect::InvEntryType;

            typ = match self.read_uint32() {
                0 => ::datatype::invvect::Error,
                1 => ::datatype::invvect::MsgTx,
                2 => ::datatype::invvect::MsgBlock,
                _ => fail!("invalid type of inventory entry")
            };

            invvec.add(::datatype::invvect::InvEntry {
                typ  : typ,
                hash : self.read_hash()
            });
        }

        invvec
    }

    pub fn consumed(&self) -> uint
    {
        self.pos
    }
}
