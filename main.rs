extern crate time;

use std::io::TcpStream;

mod marshalling;

mod config
{
    pub static NAME : &'static str = "rustybit";

    pub static VERSION_MAJOR : u8 = 0;
    pub static VERSION_MINOR : u8 = 0;
    pub static VERSION_DEV   : u8 = 0;

    pub static PROTOCOL_VERSION : u32 = 70002;

    enum Services {
        NodeNetwork = 1 << 0,
    }

    pub static SERVICES: Services = NodeNetwork;

    pub fn version() -> String
    {
        format!("{}.{}-{}",VERSION_MAJOR,VERSION_MINOR,VERSION_DEV)
    }

    pub fn name_version() -> String
    {
        format!("{} {}",NAME,version())
    }
}

mod message
{
    static MAIN_NET: [u8, ..4] = [0xf9,0xbe,0xb4,0xd9];

    pub struct Version
    {
        name         : String,
        version      : String,
        time         : ::time::Tm
    }

    impl Version
    {
        pub fn new(name : String, version : String) -> Version
        {
            Version { name:    name,
                      version: version,
                      time:    ::time::now_utc() }
        }

        /* Format the version according to BIP0014
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
            msg.write_timestamp(&self.time);
            // recipient addr
            // sender addr
            // node id
            msg.write_varstr(&self.name_version_bip0014());
            // last block

            header.write(MAIN_NET);
            header.write_str12(&"version".to_string());
            header.write_uint32(msg.len() as u32);
            // checksum

            header.get() + msg.get()
        }
    }
}

fn send_version(socket : &mut TcpStream)
{
    let version = message::Version::new(config::NAME.to_string(),
                                        config::version());

    socket.write(version.serialize().as_slice());
}

fn main()
{
    let mut socket = TcpStream::connect("192.168.1.2", 8333).unwrap();

    send_version(&mut socket);

    let response = socket.read_to_end();

    println!("{}", response);
}
