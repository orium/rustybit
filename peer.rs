use std::io::net::ip::SocketAddr;
use std::io::TcpStream;

use message::Header;

macro_rules! try_or(
    ($e:expr, $err:expr) => (match $e { Ok(e) => e, Err(_) => return $err })
)

macro_rules! some_ref_or(
    ($e:expr, $err:expr) => (match $e { Some(ref mut e) => e, None => return $err })
)

pub struct Peer
{
    addr   : SocketAddr,
    socket : Option<TcpStream>
}

/* TODO inside Peer */
static CONNECT_TIMEOUT : u64 = 5000; /* ms */
static ERR : Result<(),()> = Err(());

impl Peer
{
    pub fn new(addr : SocketAddr) -> Peer
    {
        Peer {
            addr:   addr,
            socket: None }
    }

    pub fn connect(&mut self) -> Result<(),()>
    {
        let maybesocket = TcpStream::connect_timeout(self.addr,CONNECT_TIMEOUT);

        self.socket = Some(try_or!(maybesocket,ERR));

        Ok(())
    }

    pub fn send_version(&mut self) -> Result<(),()>
    {
        let socket : &mut TcpStream = some_ref_or!(self.socket,ERR);
        let version = ::message::Version::new(::config::NAME.to_string(),
                                              ::config::version(),
                                              0);

        try_or!(socket.write(version.serialize().as_slice()),ERR);

        Ok(())
    }

    pub fn read_loop(&mut self) -> Result<(),()>
    {
        let socket : &mut TcpStream = some_ref_or!(self.socket,ERR);
        let mut inbytes : uint = 0;
        let mut msg_count : uint = 0;

        loop
        {
            let data_hd : Vec<u8>;
            let data_msg : Vec<u8>;
            let header : Header;

            /* TODO 24 should be in Header::HEADER_SIZE */
            data_hd = try_or!(socket.read_exact(24),ERR);
            header = Header::unserialize(&data_hd);

            /* TODO check max msg size */
            data_msg = try_or!(socket.read_exact(header.get_payload_len()),ERR);


            /* TODO check network
             * TODO check checksum
             */

            println!("{}  command: {:9} len: {}",
                     self.addr,
                     header.get_command(),
                     header.get_payload_len());

            msg_count += 1;
            inbytes += 24+header.get_payload_len(); /* use Header::HEADER_SIZE */

            if msg_count%100 == 0
            {
                println!("{} in: {} KB",self.addr,inbytes/1024);
            }
        };

        Ok(())
    }
}
