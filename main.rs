use std::io::TcpStream;
use std::io::net::ip::SocketAddr;
use std::io::net::ip::Ipv4Addr;
use std::io::IoError;

mod marshalling;
mod message;
mod config;

struct Peer
{
    addr   : SocketAddr,
    socket : Option<TcpStream>
}

impl Peer
{
    pub fn new(addr : SocketAddr) -> Peer
    {
        Peer {
            addr: addr,
            socket: None
        }
    }

    pub fn connect(&mut self) -> Result<(),IoError>
    {
        match TcpStream::connect("192.168.1.2", 8333)
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
        let version = message::Version::new(config::NAME.to_string(),
                                            config::version());

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

fn main()
{
    let mut orion : Peer = Peer::new(SocketAddr { ip: Ipv4Addr(192, 168, 1, 2),
                                                  port: 8333 });

    orion.connect();
    orion.send_version();
}
