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
            socket: None
        }
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
}
