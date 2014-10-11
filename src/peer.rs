extern crate time;

use std::io::net::ip::SocketAddr;
use std::io::TcpStream;

use std::time::duration::Duration;

use message::Message;
use message::MsgVersion;
use message::Header;
use message::Version;

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

/* TODO:
 * define a error enum to use in result
 */

/* TODO inside Peer */
static ERR : Result<(),()> = Err(());

impl Peer
{
    pub fn new(addr : SocketAddr) -> Peer
    {
        Peer { addr:   addr,
               socket: None }
    }

    pub fn connect(&mut self) -> Result<(),()>
    {
        let CONNECT_TIMEOUT : Duration = Duration::milliseconds(5000);
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

    /* Returns Err(true) if fatal error
     */
    pub fn read_message(&mut self) -> Result<::message::Message,bool>
    {
        let ERR_FATAL = Err(true);
        let ERR_OK    = Err(false);
        let socket : &mut TcpStream = some_ref_or!(self.socket,ERR_FATAL);
        let data_hd : Vec<u8>;
        let data_msg : Vec<u8>;
        let header : Header;

        /* TODO 24 should be in Header::HEADER_SIZE */
        data_hd = try_or!(socket.read_exact(24),ERR_FATAL);
        header = Header::unserialize(&data_hd);

        /* TODO check max msg size */
        data_msg = try_or!(socket.read_exact(header.get_payload_len()),ERR_FATAL);

        /* TODO check network
         * TODO check checksum
         */

        println!("{}  {}  \tcommand: {:9} len: {}",
                 time::now().rfc822z(),
                 self.addr,
                 header.get_command(),
                 header.get_payload_len());

        ERR_OK
    }

    pub fn read_loop(&mut self) -> Result<(),()>
    {
        loop
        {
            let maybemsg = self.read_message();

            if maybemsg.is_err()
            {
                match maybemsg
                {
                    Err(true) => return Err(()),
                    Err(false) => continue,
                    _ => unreachable!()
                }
            }

            match maybemsg.unwrap()
            {
                MsgVersion(version) =>
                {
                    println!("got a version");
                }
            };
        };

        Ok(())
    }
}
