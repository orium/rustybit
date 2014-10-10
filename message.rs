extern crate time;

static MAIN_NET : u32 = 0xD9B4BEF9;

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

    pub fn unserialize(data : &Vec<u8>) -> Header
    {
        let mut unmarshalling = ::marshalling::Unmarshalling::new(data);

        assert!(data.len() == 24);

        Header
        {
            network:  unmarshalling.read_uint32(),
            command:  unmarshalling.read_str12(),
            len:      unmarshalling.read_uint32(),
            checksum: unmarshalling.read_uint32()
        }
    }

    pub fn get_command<'a>(&'a self) -> &'a String
    {
        &self.command
    }

    pub fn get_payload_len(&self) -> uint
    {
        self.len as uint
    }

    pub fn checksum(data : &Vec<u8>) -> u32
    {
        0 // XXX
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

pub struct Version
{
    name    : String,
    version : String,
    time    : time::Tm,
    nounce  : u64
}

impl Version
{
    pub fn new(name : String, version : String) -> Version
    {
        // TODO: rnd should be a global variable. Is that possible in rust?
        // let mut rng : ::std::rand::OsRng = ::std::rand::OsRng::new().unwrap();

        Version { name:    name,
                  version: version,
                  time:    time::now_utc(),
                  nounce:  0xababeface // TODO rng.gen()
        }
    }

    /* Format the version according to BIP0014.
     * https://en.bitcoin.it/wiki/BIP_0014
     */
    fn name_version_bip0014(&self) -> String
    {
        format!("/{}:{}/",self.name,self.version)
    }

    // TODO: create a trait for serialization
    pub fn serialize(&self) -> Vec<u8>
    {
        let mut msg = ::marshalling::Marshalling::new();

        msg.write_uint32(::config::PROTOCOL_VERSION);
        msg.write_uint64(::config::SERVICES as u64);

        // XXX
        if false
            { msg.write_timestamp(self.time); }
        else
            { msg.write_int64(1412833399i64); }

        msg.write_netaddr(None,::config::SERVICES,None); /* recv addr */
        msg.write_netaddr(None,::config::SERVICES,None); /* send addr */
        msg.write_uint64(self.nounce);
        msg.write_varstr(&self.name_version_bip0014());
        msg.write_uint32(324485); /* TODO last block */
        msg.write_bool(true); /* relay transactions */

        let header : Header = Header::new(MAIN_NET,
                                          "version".to_string(),
                                          msg.len() as u32,
                                          2596763594 as u32);

        header.serialize() + msg.get()
    }
}
