extern crate time;

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

    pub fn write_int64(&mut self, v : i64)
    {
        assert!(v >= 0);

        self.write_uint64(v as u64);
    }

    pub fn write_varint(&mut self, v : u64)
    {
        match v
        {
            0u64           .. 252u64             => {
                self.buf.push(v as u8);
            },
            253u64         .. 0xffffu64             => {
                self.buf.push(253u8);
                self.write_uint16(v as u16);
            },
            0x10000u64     .. 0xffffffffu64         => {
                self.buf.push(254u8);
                self.write_uint32(v as u32);
            },
            0x100000000u64 .. 0xffffffffffffffffu64 => {
                self.buf.push(255u8);
                self.write_uint64(v);
            },
            _ => fail!("This should never happen!")
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

    pub fn write_timestamp(&mut self, time : &time::Tm)
    {
        self.write_int64(time.to_timespec().sec);
    }

    // netaddr, (checksum?)

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
