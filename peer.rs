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
            let data : Vec<u8> = socket.read_exact(24).unwrap();

            for i in range(4u,12)
            {
                if *data.get(i) == 0u8 { println!(""); break; }

                print!("{}",*data.get(i) as char);
            }

            let i0 = *data.get(12+4) as uint;
            let i1 = *data.get(12+4+1) as uint;
            let i2 = *data.get(12+4+2) as uint;
            let i3 = *data.get(12+4+3) as uint;

            socket.read_exact(i0|(i1<<8)|(i2<<16)|(i3<<24)).unwrap();
        };

        Ok(())
    }
}
