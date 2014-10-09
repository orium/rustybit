extern crate time;

static MAIN_NET: [u8, ..4] = [0xf9,0xbe,0xb4,0xd9];

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

    pub fn serialize(&self) -> Vec<u8>
    {
        let mut header = ::marshalling::Marshalling::new();
        let mut msg = ::marshalling::Marshalling::new();

        msg.write_uint32(::config::PROTOCOL_VERSION);
        msg.write_uint64(::config::SERVICES as u64);
        msg.write_timestamp(self.time);
        msg.write_netaddr(None,::config::SERVICES,None); /* recv addr */
        msg.write_netaddr(None,::config::SERVICES,None); /* send addr */
        header.write_uint64(self.nounce);
        msg.write_varstr(&self.name_version_bip0014());
        msg.write_uint32(324485); /* TODO last block */

        header.write(MAIN_NET);
        header.write_str12(&"version".to_string());
        header.write_uint32(msg.len() as u32);
        // checksum

        header.get() + msg.get()
    }
}