extern crate time;

use std::io::net::ip::SocketAddr;
use std::io::TcpStream;

use std::time::duration::Duration;

use message::Message;
use message::MsgVersion;
use message::MsgVersionAck;
use message::Header;
use message::Version;
use message::VersionAck;

macro_rules! try_or(
    ($e:expr, $err:expr) => (match $e { Ok(e) => e, Err(_) => return $err })
)

macro_rules! some_ref_or(
    ($e:expr, $err:expr) => (match $e { Some(ref mut e) => e, None => return $err })
)

pub struct Peer
{
    addr    : SocketAddr,
    socket  : Option<TcpStream>,
    version : Option<Version>
}

/* TODO:
 * define a error enum to use in result
 */

/* TODO inside Peer */
static ERR : Result<(),()> = Err(());
static PAYLOAD_MAX_SIZE : uint = 4*(1<<20); /* 4MB */

impl Peer
{
    pub fn new(addr : SocketAddr) -> Peer
    {
        Peer { addr:    addr,
               socket:  None,
               version: None }
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
        let version = Version::new(::config::name_version_bip0014(),0);

        try_or!(socket.write(version.serialize().as_slice()),ERR);

        Ok(())
    }

    fn send_versionack(&mut self) -> Result<(),()>
    {
        let socket : &mut TcpStream = some_ref_or!(self.socket,ERR);
        let verack = VersionAck::new();

        try_or!(socket.write(verack.serialize().as_slice()),ERR);

        Ok(())
    }

    /* Returns Err(true) if fatal error
     */
    pub fn read_message(&mut self) -> Result<Message,bool>
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

        if header.get_payload_len() >= PAYLOAD_MAX_SIZE
        {
            println!("message payload length too big");
            return ERR_OK;
        }

        data_msg = try_or!(socket.read_exact(header.get_payload_len()),ERR_FATAL);

        /* TODO check network
         */

        if ::crypto::checksum(&data_msg) != header.get_checksum()
        {
            println!("invalid checksum");
            return ERR_OK;
        }

        println!("{}\n",header);

        println!("{}  {}  \tcommand: {:9} len: {}",
                 time::now().rfc822z(),
                 self.addr,
                 header.get_command(),
                 header.get_payload_len());

        match header.get_command().as_slice()
        {
            "version" =>
            {
                let version : Version;

                version = Version::unserialize(&data_msg);

                Ok(MsgVersion(version))
            },
            "verack" =>
            {
                let verack : VersionAck;

                verack = VersionAck::unserialize(&data_msg);

                Ok(MsgVersionAck(verack))
            },
            _ => ERR_OK
        }
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
                MsgVersion(version)   =>
                {
                    println!("{}",version);

                    self.version = Some(version);

                    try_or!(self.send_versionack(),Err(()));
                },
                MsgVersionAck(verack) =>
                {
                    println!("{}",verack);
                }
            };
        };

        Ok(())
    }
}
