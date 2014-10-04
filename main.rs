use std::io::TcpStream;

mod marshalling;

mod config
{
    pub static NAME: &'static str = "rustybit";
    pub static VERSION: &'static str = "0.0";

    pub static PROTOCOL_VERSION: u32 = 70002;
}

mod message
{
    static MAIN_NET: [u8, ..4] = [0xf9,0xbe,0xb4,0xd9];

    pub struct Version
    {
        version: &'static str
    }

    fn serialize_uint32(v: u32) -> [u8, ..4]
    {
        let l0: u8=(v&0xff) as u8;
        let l1: u8=((v>>8)&0xff) as u8;
        let l2: u8=((v>>8*2)&0xff) as u8;
        let l3: u8=((v>>8*3)&0xff) as u8;

        [l0,l1,l2,l3]
    }

    fn serialize_uint64(v: u64) -> [u8, ..8]
    {
        let l0: u8=(v&0xff) as u8;
        let l1: u8=((v>>8)&0xff) as u8;
        let l2: u8=((v>>8*2)&0xff) as u8;
        let l3: u8=((v>>8*3)&0xff) as u8;
        let l4: u8=((v>>8*4)&0xff) as u8;
        let l5: u8=((v>>8*5)&0xff) as u8;
        let l6: u8=((v>>8*6)&0xff) as u8;
        let l7: u8=((v>>8*7)&0xff) as u8;

        [l0,l1,l2,l3,l4,l5,l6,l7]
    }

    impl Version
    {
        pub fn new(version: &'static str) -> Version
        {
            Version { version: version }
        }

        pub fn serialize(&self) // -> &[u8]
        {
            let VERSION : [u8, ..12] = [0x76,0x65,0x72,0x73,0x69,0x6F,0x6E,
                                        0x00,0x00,0x00,0x00,0x00];
/*/
            let mut header : Vec<u8> = Vec::new();
            let mut msg : Vec<u8> = Vec::new();

            // TODO make message / make header functions
            header.push_all(MAIN_NET);
            header.push_all(VERSION);

            header+msg
*/
            let mut header = ::marshalling::Marshalling::new();

            header.write(MAIN_NET);
            header.write(VERSION);
        }
    }
}

fn send_version(socket : &mut TcpStream)
{
    let version = message::Version::new(config::NAME);

    // socket.write(version.serialize().as_slice());
}

fn main()
{
    let mut socket = TcpStream::connect("192.168.1.2", 8333).unwrap();

    send_version(&mut socket);

    let response = socket.read_to_end();

    println!("{}", response);
}
