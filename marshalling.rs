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

    // date, netaddr, str, (checksum?)

    pub fn write(&mut self, d: &[u8])
    {
        self.buf.push_all(d);
    }

    pub fn write_uint32(&mut self, v: u32)
    {
        for i in range(0u,4)
        {
            self.buf.push(((v>>8*i)&0xff) as u8);
        }
    }

    pub fn write_uint64(&mut self, v: u64)
    {
        for i in range(0u,8)
        {
            self.buf.push(((v>>8*i)&0xff) as u8);
        }
    }

    pub fn get<'a>(&'a self) -> &'a [u8]
    {
        self.buf.as_slice()
    }
}

// TODO: Unmarchalling
