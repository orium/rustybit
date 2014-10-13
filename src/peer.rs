extern crate time;

use std::io::net::ip::SocketAddr;
use std::io::TcpStream;

use std::time::duration::Duration;

use message::Message;
use message::MsgVersion;
use message::MsgVersionAck;
use message::MsgPing;
use message::MsgPong;
use message::MsgAddresses;
use message::MsgInv;
use message::MsgGetData;

use message::header::Header;
use message::header::HEADER_SIZE;

use message::version::Version;
use message::versionack::VersionAck;
use message::ping::Ping;
use message::pong::Pong;
use message::addresses::Addresses;
use message::inv::Inv;
use message::getdata::GetData;

use datatype::invvect::InvVect;

macro_rules! try_or(
    ($e:expr, $err:expr) => (match $e { Ok(e) => e, Err(_) => return $err })
)

macro_rules! some_ref_or(
    ($e:expr, $err:expr) => (match $e { Some(ref mut e) => e, None => return $err })
)

#[deriving(Show)]
pub enum PeerError
{
    ReadEOF,
    ReadTimeout,
    ReadIncomplete,
    ReadIOError,
    ReadMsgPayloadTooBig,
    ReadMsgInvalidChecksum,
    ReadMsgUnknownMsg,
    WriteIOError,
    ConnectIOError,
    NotConnected,
    DoubleHandshake,
    UnsupportedProtoVersion,
}

pub struct Peer
{
    addr    : SocketAddr,
    socket  : Option<TcpStream>,
    version : Option<Version>
}

static PAYLOAD_MAX_SIZE : uint = 4*(1<<20); /* 4MB */
static CONNECT_TIMEOUT_MS : uint = 5000;

impl Peer
{
    pub fn new(addr : SocketAddr) -> Peer
    {
        Peer { addr:    addr,
               socket:  None,
               version: None }
    }

    pub fn connect(&mut self) -> Result<(),PeerError>
    {
        let timeout : Duration = Duration::milliseconds(CONNECT_TIMEOUT_MS as i64);
        let maybesocket = TcpStream::connect_timeout(self.addr,timeout);

        self.socket = Some(try_or!(maybesocket,Err(ConnectIOError)));

        Ok(())
    }

    pub fn send_version(&mut self) -> Result<(),PeerError>
    {
        let socket : &mut TcpStream = some_ref_or!(self.socket,Err(NotConnected));
        let version = Version::new(::config::name_version_bip0014(),0);

        println!("<<< {}  {:30} command: {:9}",
                 time::now().rfc822z(),
                 self.addr,
                 "version");

        try_or!(socket.write(version.serialize().as_slice()),Err(WriteIOError));

        println!("{:4}",version);

        Ok(())
    }

    fn send_versionack(&mut self) -> Result<(),PeerError>
    {
        let socket : &mut TcpStream = some_ref_or!(self.socket,Err(NotConnected));
        let verack = VersionAck::new();

        println!("<<< {}  {:30} command: {:9}",
                 time::now().rfc822z(),
                 self.addr,
                 "verack");

        try_or!(socket.write(verack.serialize().as_slice()),Err(WriteIOError));

        println!("{:4}",verack);

        Ok(())
    }

    fn send_pong(&mut self, nounce : u64) -> Result<(),PeerError>
    {
        let socket : &mut TcpStream = some_ref_or!(self.socket,Err(NotConnected));
        let pong = Pong::new(nounce);

        println!("<<< {}  {:30} command: {:9}",
                 time::now().rfc822z(),
                 self.addr,
                 "pong");

        try_or!(socket.write(pong.serialize().as_slice()),Err(WriteIOError));

        println!("{:4}",pong);

        Ok(())
    }

    fn send_getdata(&mut self, inv : &InvVect)  -> Result<(),PeerError>
    {
        let socket : &mut TcpStream = some_ref_or!(self.socket,Err(NotConnected));
        let getdata = GetData::from_inv(inv);

        println!("<<< {}  {:30} command: {:9}",
                 time::now().rfc822z(),
                 self.addr,
                 "getdata");

        try_or!(socket.write(getdata.serialize().as_slice()),Err(WriteIOError));

        println!("{:4}",getdata);

        Ok(())
    }

    pub fn read_loop(&mut self) -> Result<(),PeerError>
    {
        let mut buffer : MsgBuffer = MsgBuffer::new();

        loop
        {
            let maybemsg = buffer.read_message(some_ref_or!(self.socket,Err(NotConnected)));

            if maybemsg.is_err()
            {
                let err : PeerError = maybemsg.err().unwrap();

                match err
                {
                    ReadEOF              => return Err(err),
                    ReadIOError          => return Err(err),
                    ReadMsgPayloadTooBig => return Err(err),
                    _                    => continue
                }
            }

            // TODO: a function for each of these messages
            match maybemsg.unwrap()
            {
                MsgVersion(version) =>
                {
                    println!("{:4}",version);

                    /* Do not allow a peer send a version msg twice */
                    if self.version.is_some()
                    {
                        return Err(DoubleHandshake);
                    }

                    if version.get_protocol_version() < ::config::PROTOCOL_VERSION_MIN
                    {
                        return Err(UnsupportedProtoVersion);
                    }

                    self.version = Some(version);

                    try!(self.send_versionack());
                },
                MsgVersionAck(verack) =>
                {
                    println!("{:4}",verack);
                },
                MsgPing(ping) =>
                {
                    println!("{:4}",ping);

                    try!(self.send_pong(ping.get_nounce()));
                }
                MsgPong(pong) =>
                {
                    println!("{:4}",pong);
                }
                MsgAddresses(addrs) =>
                {
                    println!("{:4}",addrs);
                }
                MsgInv(inv) =>
                {
                    println!("{:4}",inv);

                    try!(self.send_getdata(inv.get_invvect()));
                }
                MsgGetData(getdata) =>
                {
                    println!("{:4}",getdata);
                }
            };
        }
    }
}

static READ_TIMEOUT_MS : uint = 500;

struct MsgBuffer
{
    buf : Vec<u8>
}

impl MsgBuffer
{
    pub fn new() -> MsgBuffer
    {
        MsgBuffer
        {
            buf: Vec::with_capacity(PAYLOAD_MAX_SIZE+HEADER_SIZE)
        }
    }

    fn drop(&mut self, n : uint)
    {
        let len = self.buf.len();

        assert!(self.buf.len() >= n);

        for src in range(n,len)
        {
            let dst = src-n;

            *self.buf.get_mut(dst) = self.buf[src];
        }

        self.buf.truncate(len-n);
    }

    fn read_ensure_size(&mut self, size : uint, socket : &mut TcpStream)
                        -> Result<(),PeerError>
    {
        if self.buf.len() < size
        {
            let result;

            socket.set_read_timeout(Some(READ_TIMEOUT_MS as u64));
            result = socket.push(size-self.buf.len(),&mut self.buf);

            if result.is_err()
            {
                match result.err().unwrap().kind
                {
                    ::std::io::EndOfFile => return Err(ReadEOF),
                    ::std::io::TimedOut  => return Err(ReadTimeout),
                    _                    => return Err(ReadIOError)
                }
            }

            assert!(self.buf.len() <= size);

            /* We fail to read the entire thing */
            if self.buf.len() < size
            {
                return Err(ReadIncomplete);
            }

            assert!(self.buf.len() == size);
        }

        assert!(self.buf.len() >= size);

        Ok(())
    }

    pub fn read_message(&mut self, socket : &mut TcpStream)
                        -> Result<Message,PeerError>
    {
        let header : Header;
        let msg : Result<Message,PeerError>;

        /* We should never have to expand */
        assert!(self.buf.capacity() == PAYLOAD_MAX_SIZE+HEADER_SIZE);

        /* Read enoght to have a header */
        try!(self.read_ensure_size(HEADER_SIZE,socket));

        assert!(self.buf.len() >= HEADER_SIZE);

        header = Header::unserialize(&self.buf);

        if header.get_payload_len() > PAYLOAD_MAX_SIZE
        {
            println!("message payload length too big");

            return Err(ReadMsgPayloadTooBig);
        }

        /* Read enoght to have the message payload */
        try!(self.read_ensure_size(HEADER_SIZE+header.get_payload_len(),socket));

        assert!(self.buf.len() == HEADER_SIZE+header.get_payload_len());

        /* We can now safely drop the header, since we have a complete message */
        self.drop(HEADER_SIZE);

        assert!(self.buf.len() == header.get_payload_len());

        if ::crypto::checksum(&self.buf) != header.get_checksum()
        {
            println!("invalid checksum");

            self.buf.clear();

            return Err(ReadMsgInvalidChecksum);
        }

        println!(">>> {}  {:30} \tcommand: {:9}",
                 time::now().rfc822z(),
                 socket.peer_name().unwrap(),
                 header.get_command());

        msg = match header.get_command().as_slice()
        {
            "version" =>
            {
                let version : Version;

                version = Version::unserialize(&self.buf);

                Ok(MsgVersion(version))
            },
            "verack" =>
            {
                let verack : VersionAck;

                verack = VersionAck::unserialize(&self.buf);

                Ok(MsgVersionAck(verack))
            },
            "ping" =>
            {
                let ping : Ping;

                ping = Ping::unserialize(&self.buf);

                Ok(MsgPing(ping))
            },
            "pong" =>
            {
                let pong : Pong;

                pong = Pong::unserialize(&self.buf);

                Ok(MsgPong(pong))
            },
            "addr" =>
            {
                let addr : Addresses;

                addr = Addresses::unserialize(&self.buf);

                Ok(MsgAddresses(addr))
            },
            "inv" =>
            {
                let inv : Inv;

                inv = Inv::unserialize(&self.buf);

                Ok(MsgInv(inv))
            },
            "getdata" =>
            {
                let getdata : GetData;

                getdata = GetData::unserialize(&self.buf);

                Ok(MsgGetData(getdata))
            },
            _ => Err(ReadMsgUnknownMsg)
        };

        self.buf.clear();

        /* TODO check network */

        msg
    }
}

/* Progress:
 *
 *          recv | send
 * ______________________
 * version    v  |   v
 * verack     v  |   v
 * ping       v  |   
 * pong       v  |   v
 * addr       v  |   
 * inv        v  |   
 * getdata       |   v
 *
 * TODO: we should ping
 */
