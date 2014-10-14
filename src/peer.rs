extern crate time;

use std::io::net::ip::SocketAddr;
use std::io::TcpStream;

use self::time::Timespec;
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
    ($e:expr, $err:expr) => (match $e { Ok(e) => e, Err(_) => return $err }))

macro_rules! some_ref_or(
    ($e:expr, $err:expr) => (match $e { Some(ref mut e) => e, None => return $err }))

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

impl PeerError
{
    pub fn is_fatal(&self) -> bool
    {
        match *self
        {
            ReadEOF                 => true,
            ReadIOError             => true,
            ReadMsgPayloadTooBig    => true,
            WriteIOError            => true,
            ConnectIOError          => true,
            NotConnected            => true,
            DoubleHandshake         => true,
            UnsupportedProtoVersion => true,
            ReadMsgInvalidChecksum  => true,
            _                       => false
        }
    }
}

static PERIODIC_PERIOD_S : uint = 5;
static PING_PERIOD_S : uint = 300;

pub struct Peer
{
    addr      : SocketAddr,
    socket    : Option<TcpStream>,
    version   : Option<Version>,
    last_ping : Option<Timespec>
}

static PAYLOAD_MAX_SIZE : uint = 4*(1<<20); /* 4MB */
static CONNECT_TIMEOUT_MS : uint = 5000;

impl Peer
{
    pub fn new(addr : SocketAddr) -> Peer
    {
        Peer
        {
            addr:      addr,
            socket:    None,
            version:   None,
            last_ping: None
        }
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

    fn send_ping(&mut self) -> Result<(),PeerError>
    {
        let socket : &mut TcpStream = some_ref_or!(self.socket,Err(NotConnected));
        let now : Timespec = time::now_utc().to_timespec();
        let ping : Ping;

        assert!(self.last_ping == None);

        ping = Ping::new(((now.sec as u64)<<10) | ((now.nsec as u64)/1_000_000));

        println!("<<< {}  {:30} command: {:9}",
                 time::now().rfc822z(),
                 self.addr,
                 "ping");

        try_or!(socket.write(ping.serialize().as_slice()),Err(WriteIOError));

        println!("{:4}",ping);

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

    fn send_getdata(&mut self, inv : &InvVect) -> Result<(),PeerError>
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

    fn handle_version(&mut self, version : Version) -> Result<(),PeerError>
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

        Ok(())
    }

    fn handle_versionack(&mut self, verack : VersionAck) -> Result<(),PeerError>
    {
        println!("{:4}",verack);

        Ok(())
    }

    fn handle_ping(&mut self, ping : Ping) -> Result<(),PeerError>
    {
        println!("{:4}",ping);

        try!(self.send_pong(ping.get_nounce()));

        Ok(())
    }

    fn handle_pong(&mut self, pong : Pong) -> Result<(),PeerError>
    {
        let now : Timespec = time::now_utc().to_timespec();
        let nounce : u64 = pong.get_nounce();
        let then : Timespec;
        let lag : Duration;
        self.last_ping = None;

        then = Timespec::new((nounce>>10) as i64,
                             ((nounce&(0x400-1))*1_000_000) as i32);

        lag = now-then;

        println!("{:4}",pong);

        println!("{}  Lag: {} ms",self.addr,lag.num_milliseconds());

        Ok(())
    }

    fn handle_addresses(&mut self, addrs : Addresses) -> Result<(),PeerError>
    {
        println!("{:4}",addrs);

        Ok(())
    }

    fn handle_inv(&mut self, inv : Inv) -> Result<(),PeerError>
    {
        println!("{:4}",inv);

        try!(self.send_getdata(inv.get_invvect()));

        Ok(())
    }

    fn handle_getdata(&mut self, getdata : GetData) -> Result<(),PeerError>
    {
        println!("{:4}",getdata);

        Ok(())
    }

    fn periodic_sendping(&mut self) -> Result<(),PeerError>
    {
        if self.last_ping.is_none()
        {
            return self.send_ping();
        }

        Ok(())
    }

    /* Warning: This ignores non fatal errors, i.e. it returns Ok with non-fatal
     *          errors
     */
    fn periodic(&mut self, periodics : &mut Vec<Periodic>) -> Result<(),PeerError>
    {
        for p in periodics.iter_mut()
        {
            if p.is_time()
            {
                let result;

                result = match p.token()
                {
                    PeriodicPing => self.periodic_sendping(),
                };

                match result
                {
                    Err(err) => if err.is_fatal() { return Err(err) },
                    _        => ()
                }

                p.done();
            }
        }

        Ok(())
    }

    pub fn read_loop(&mut self) -> Result<(),PeerError>
    {
        let mut buffer : MsgBuffer = MsgBuffer::new();
        let mut last_periodic : Timespec = time::now_utc().to_timespec();
        let mut periodics : Vec<Periodic> = Vec::new();

        periodics.push(Periodic::new(Duration::seconds(PING_PERIOD_S as i64),
                                     PeriodicPing));

        loop
        {
            let maybemsg : Result<Message,PeerError>;
            let result : Result<(),PeerError>;

            if time::now_utc().to_timespec()
                > last_periodic+Duration::seconds(PERIODIC_PERIOD_S as i64)
            {
                match self.periodic(&mut periodics)
                {
                    Err(err) => if err.is_fatal() { return Err(err) },
                    _        => ()
                }

                last_periodic = time::now_utc().to_timespec();
            }

            maybemsg = buffer.read_message(some_ref_or!(self.socket,Err(NotConnected)));

            if maybemsg.is_err()
            {
                let err : PeerError = maybemsg.err().unwrap();

                if err.is_fatal()
                {
                    return Err(err);
                }

                continue;
            }

            result = match maybemsg.unwrap()
            {
                MsgVersion(version)   => self.handle_version(version),
                MsgVersionAck(verack) => self.handle_versionack(verack),
                MsgPing(ping)         => self.handle_ping(ping),
                MsgPong(pong)         => self.handle_pong(pong),
                MsgAddresses(addrs)   => self.handle_addresses(addrs),
                MsgInv(inv)           => self.handle_inv(inv),
                MsgGetData(getdata)   => self.handle_getdata(getdata),
            };

            match result
            {
                Err(err) => if err.is_fatal() { return Err(err) },
                _        => ()
            }
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

        if header.get_payload_size() > PAYLOAD_MAX_SIZE
        {
            println!("message payload length too big");

            return Err(ReadMsgPayloadTooBig);
        }

        /* Read enoght to have the message payload */
        try!(self.read_ensure_size(HEADER_SIZE+header.get_payload_size(),socket));

        assert!(self.buf.len() == HEADER_SIZE+header.get_payload_size());

        /* We can now safely drop the header, since we have a complete message */
        self.drop(HEADER_SIZE);

        assert!(self.buf.len() == header.get_payload_size());

        if ::crypto::checksum(&self.buf) != header.get_checksum()
        {
            println!("invalid checksum");

            self.buf.clear();

            return Err(ReadMsgInvalidChecksum);
        }

        println!(">>> {}  {} \tcommand: {:9}",
                 time::now().rfc822z(),
                 socket.peer_name().unwrap(),
                 header.get_command());

        msg = match header.get_command().as_slice()
        {
            "version" =>
            {
                let version : Version;

                version = Version::unserialize(&self.buf,
                                               header.get_payload_size());

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

enum PeriodicToken
{
    PeriodicPing
}

/* TODO: Remove token and instead store a closure with the call to run.
 *       Method done() will become go() which calls the closure before
 *       updating self.last.  I tried this but I could not get the lifetimes
 *       working (and the documentation is laking in this regard).
 */
struct Periodic
{
    interval : Duration,
    last     : Timespec,
    token    : PeriodicToken
}

impl Periodic
{
    pub fn new<'a>(interval : Duration,
                   token : PeriodicToken) -> Periodic
    {
        Periodic
        {
            interval: interval,
            last:     time::now_utc().to_timespec(),
            token:    token
        }
    }

    pub fn is_time(&self) -> bool
    {
        let now = time::now_utc().to_timespec();

        now > self.last+self.interval
    }

    pub fn token(&self) -> PeriodicToken
    {
        self.token
    }

    pub fn done(&mut self)
    {
        self.last = time::now_utc().to_timespec();
    }
}

/* Progress:
 *
 *          recv | send
 * ______________________
 * version    v  |   v
 * verack     v  |   v
 * ping       v  |   v
 * pong       v  |   v
 * addr       v  |   
 * inv        v  |   
 * getdata       |   v
 */
