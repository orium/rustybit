use std::io::net::ip::SocketAddr;
use std::io::TcpStream;
use std::io::IoError;

pub struct Peer
{
    addr   : SocketAddr,
    socket : Option<TcpStream>
}

static CONNECT_TIMEOUT : u64 = 5000; /* ms */

impl Peer
{
    pub fn new(addr : SocketAddr) -> Peer
    {
        Peer {
            addr:   addr,
            socket: None }
    }

    pub fn connect(&mut self) -> Result<(),IoError>
    {
        match TcpStream::connect_timeout(self.addr,CONNECT_TIMEOUT)
        {
            Ok(socket) =>
            {
                self.socket = Some(socket);
                Ok(())
            }
            Err(e)     => Err(e)
        }
    }

    pub fn send_version(&mut self) -> Result<(),()>
    {
        if self.socket.is_none()
        {
            return Err(());
        }

        let version = ::message::Version::new(::config::NAME.to_string(),
                                              ::config::version());

        match self.socket
        {
            Some(ref mut socket) =>
            {
                let r = socket.write(version.serialize().as_slice());

                if r.is_ok() { Ok(()) } else { Err(()) }
            },
            None                 => Err(())
        }
    }

    pub fn read_loop(&mut self) -> Result<(),()>
    {
        if self.socket.is_none()
        {
            return Err(());
        }

        let socket : &mut TcpStream = self.socket.get_mut_ref();

        loop
        {
            let data_hd : Vec<u8> = socket.read_exact(24).unwrap();
            let header : ::message::Header = ::message::Header::unserialize(&data_hd);
            let data_msg : Vec<u8> = socket.read_exact(header.get_payload_len()).unwrap();

            /* TODO check network
             * TODO check checksum
             */

            println!("{}  command: {:9} len: {}",
                     self.addr,
                     header.get_command(),
                     header.get_payload_len());
        };

        Ok(())
    }
}
