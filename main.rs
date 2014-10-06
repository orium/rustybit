extern crate time;

use std::io::TcpStream;

mod marshalling;

mod config
{
    pub static NAME: &'static str = "rustybit";
    pub static VERSION: &'static str = "0.0";

    pub static PROTOCOL_VERSION: u32 = 70002;

    enum Services {
        NodeNetwork = 1 << 0,
    }

    pub static SERVICES: Services = NodeNetwork;
}

mod message
{
    static MAIN_NET: [u8, ..4] = [0xf9,0xbe,0xb4,0xd9];

    pub struct Version
    {
        version: &'static str,
        time:    ::time::Tm
    }

    impl Version
    {
        pub fn new(version: &'static str) -> Version
        {
            Version { version: version,
                      time: ::time::now_utc() }
        }

        pub fn serialize(&self) -> Vec<u8>
        {
            let mut header = ::marshalling::Marshalling::new();
            let mut msg = ::marshalling::Marshalling::new();

            header.write(MAIN_NET);
            header.write_str12("version");
            // payload size
            // checksum

            msg.write_uint32(::config::PROTOCOL_VERSION);
            msg.write_uint64(::config::SERVICES as u64);
            msg.write_int64(self.time.to_timespec().sec);
            // recipient addr
            // sender addr
            // node id
            msg.write_varstr(self.version);
            // last block

            header.get() + msg.get()
        }
    }
}

fn send_version(socket : &mut TcpStream)
{
    let version = message::Version::new(config::NAME);

    socket.write(version.serialize().as_slice());
}

fn main()
{
    let mut socket = TcpStream::connect("192.168.1.2", 8333).unwrap();

    send_version(&mut socket);

    let response = socket.read_to_end();

    println!("{}", response);
}
