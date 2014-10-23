extern crate time;

use std::io::net::ip::SocketAddr;
use std::io::TcpStream;

use self::time::Timespec;
use std::time::duration::Duration;

use message::Message;
use message::MsgVersion;
use message::MsgVerAck;
use message::MsgPing;
use message::MsgPong;
use message::MsgAddr;
use message::MsgInv;
use message::MsgGetData;
use message::MsgReject;
use message::MsgTx;

use message::version::Version;
use message::verack::VerAck;
use message::ping::Ping;
use message::pong::Pong;
use message::addr::Addr;
use message::inv::Inv;
use message::getdata::GetData;
use message::reject::Reject;
use message::tx::Tx;

use datatype::invvect::InvVect;
use datatype::netaddr::NetAddr;

use msgbuffer::MsgBuffer;

use addrmng::AddrManagerChannel;
use addrmng::AddrManagerRequest;
use addrmng::AddrManagerReply;
use addrmng::AddrMngAddAddresses;
use addrmng::AddrMngGetAddresses;
use addrmng::AddrMngAddresses;

macro_rules! some_ref_or(
    ($e:expr, $err:expr) => (match $e { Some(ref mut e) => e, None => return $err }))

/* TODO: Add information, i.e. ReadIOError(ioerror); PingTimeout(timeout)
 */
#[deriving(Show)]
pub enum PeerError
{
    ReadEOF,
    ReadTimeout,
    ReadIncomplete,
    ReadIOError,
    ReadMsgPayloadTooBig,
    ReadMsgInvalidChecksum,
    ReadMsgUnknownCommand,
    ReadMsgWrongNetwork,
    WriteIOError,
    WriteTimeout,
    ConnectError,
    NotConnected,
    DoubleHandshake,
    UnsupportedProtoVersion,
    PingTimeout,
    _AddressMngError
}

impl PeerError
{
    pub fn is_fatal(&self) -> bool
    {
        match *self
        {
            ReadTimeout           => false,
            ReadIncomplete        => false,
            ReadMsgUnknownCommand => false,
            _                     => true
        }
    }
}

static PERIODIC_PERIOD_S : uint = 5;

static PERIOD_PING_S : uint = 2*60;
static PERIOD_TIMEOUT_CHECK_S : uint = 10;
static PERIOD_ANNOUNCE_ADDRS_S : uint = 5*60;

static TIMEOUT_S : uint = 10*60;

pub struct Peer
{
    addr            : SocketAddr,
    socket          : Option<TcpStream>,
    version         : Option<Version>,
    last_ping       : Option<Timespec>,
    addrmng_channel : AddrManagerChannel
}

static TIMEOUT_CONNECT_MS : uint = 10000;
static TIMEOUT_WRITE_MS : uint = 5*60*1000;

impl Peer
{
    pub fn new(addr            : SocketAddr,
               addrmng_channel : AddrManagerChannel) -> Peer
    {
        Peer
        {
            addr:            addr,
            socket:          None,
            version:         None,
            last_ping:       None,
            addrmng_channel: addrmng_channel
        }
    }

    pub fn connect(&mut self) -> Result<(),PeerError>
    {
        let timeout : Duration = Duration::milliseconds(TIMEOUT_CONNECT_MS as i64);
        let maybesocket = TcpStream::connect_timeout(self.addr,timeout);

        if maybesocket.is_err()
        {
            return Err(ConnectError);
        }

        self.socket = Some(maybesocket.unwrap());

        Ok(())
    }

    fn send(&mut self, msg : &Vec<u8>) -> Result<(),PeerError>
    {
        let socket : &mut TcpStream = some_ref_or!(self.socket,Err(NotConnected));
        let result;

        socket.set_write_timeout(Some(TIMEOUT_WRITE_MS as u64));

        result = socket.write(msg.as_slice());

        if result.is_err()
        {
            match result.err().unwrap().kind
            {
                ::std::io::TimedOut  => return Err(WriteTimeout),
                _                    => return Err(WriteIOError)
            }
        }

        Ok(())
    }

    pub fn send_version(&mut self) -> Result<(),PeerError>
    {
        let version = Version::new(::config::name_version_bip0014(),0);

        println!("<<< {}  {:30} command: {:9}",
                 time::now().rfc822z(),
                 self.addr,
                 "version");

        try!(self.send(&version.serialize()));

        println!("{:4}",version);

        Ok(())
    }

    fn send_verack(&mut self) -> Result<(),PeerError>
    {
        let verack = VerAck::new();

        println!("<<< {}  {:30} command: {:9}",
                 time::now().rfc822z(),
                 self.addr,
                 "verack");

        try!(self.send(&verack.serialize()));

        println!("{:4}",verack);

        Ok(())
    }

    fn send_ping(&mut self) -> Result<(),PeerError>
    {
        let now : Timespec = time::now_utc().to_timespec();
        let ping : Ping;

        assert!(self.last_ping == None);

        ping = Ping::new(((now.sec as u64)<<10) | ((now.nsec as u64)/1_000_000));

        println!("<<< {}  {:30} command: {:9}",
                 time::now().rfc822z(),
                 self.addr,
                 "ping");

        try!(self.send(&ping.serialize()));

        self.last_ping = Some(time::now_utc().to_timespec());

        println!("{:4}",ping);

        Ok(())
    }

    fn send_pong(&mut self, nounce : u64) -> Result<(),PeerError>
    {
        let pong = Pong::new(nounce);

        println!("<<< {}  {:30} command: {:9}",
                 time::now().rfc822z(),
                 self.addr,
                 "pong");

        try!(self.send(&pong.serialize()));

        println!("{:4}",pong);

        Ok(())
    }

    fn send_getdata(&mut self, inv : &InvVect) -> Result<(),PeerError>
    {
        let getdata = GetData::from_inv(inv);

        println!("<<< {}  {:30} command: {:9}",
                 time::now().rfc822z(),
                 self.addr,
                 "getdata");

        try!(self.send(&getdata.serialize()));

        println!("{:4}",getdata);

        Ok(())
    }

    fn send_addr(&mut self, addrs : &Vec<NetAddr>) -> Result<(),PeerError>
    {
        let addr = Addr::from_addrs(addrs);

        println!("<<< {}  {:30} command: {:9}",
                 time::now().rfc822z(),
                 self.addr,
                 "addr");

        try!(self.send(&addr.serialize()));

        println!("{:4}",addr);

        Ok(())
    }

    fn addr_mng_send(&self, request : AddrManagerRequest)
    {
        let (ref sender, _) = self.addrmng_channel;

        sender.send(request);
    }

    fn addr_mng_send_recv(&self, request : AddrManagerRequest) -> AddrManagerReply
    {
        let (_, ref receiver) = self.addrmng_channel;

        self.addr_mng_send(request);

        receiver.recv()
    }

    fn addr_mng_add_self(&self)
    {
        let mut singleton_addrs : Vec<NetAddr> = Vec::with_capacity(1);
        let addr : NetAddr;

        assert!(self.version.is_some());

        addr = self.version.as_ref().unwrap().get_addr_send().clone();

        singleton_addrs.push(addr);

        if addr.is_valid_addr()
        {
            self.addr_mng_send(AddrMngAddAddresses(singleton_addrs));
        }
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

        try!(self.send_verack());

        self.addr_mng_add_self();

        Ok(())
    }

    fn handle_verack(&mut self, verack : VerAck) -> Result<(),PeerError>
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

    fn handle_addr(&mut self, addrs : Addr) -> Result<(),PeerError>
    {
        println!("{:4}",addrs);

        self.addr_mng_send(AddrMngAddAddresses(addrs.get_addresses().clone()));

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

    fn handle_reject(&mut self, reject : Reject) -> Result<(),PeerError>
    {
        println!("{:4}",reject);

        Ok(())
    }

    fn handle_tx(&mut self, tx : Tx) -> Result<(),PeerError>
    {
        println!("{:4}",tx);

        Ok(())
    }

    fn announce_addresses(&mut self) -> Result<(),PeerError>
    {
        let reply = self.addr_mng_send_recv(AddrMngGetAddresses);

        match reply
        {
            AddrMngAddresses(addrs) =>
            {
                println!("XXX Peer: got {}",addrs);

                assert!(addrs.len() <= ::message::addr::MSG_ADDR_MAX);

                self.send_addr(&addrs)
            },
            // _ => Err(AddressMngError)
        }
    }

    fn periodic_sendping(&mut self) -> Result<(),PeerError>
    {
        if self.last_ping.is_none()
        {
            try!(self.send_ping());
        }

        Ok(())
    }

    fn periodic_timeout_check(&mut self) -> Result<(),PeerError>
    {
        if self.last_ping.is_some()
        {
            let now : Timespec = time::now_utc().to_timespec();
            let last : Timespec = self.last_ping.unwrap();

            if now > last+Duration::seconds(TIMEOUT_S as i64)
            {
                return Err(PingTimeout);
            }
        }

        Ok(())
    }

    fn periodic_announce_addrs(&mut self) -> Result<(),PeerError>
    {
        self.announce_addresses()
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
                    PeriodicPing         => self.periodic_sendping(),
                    PeriodicTimeoutCheck => self.periodic_timeout_check(),
                    PeriodicAnnounceAddresses => self.periodic_announce_addrs()
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

    fn init_periodics() -> Vec<Periodic>
    {
        let mut periodics : Vec<Periodic> = Vec::new();

        periodics.push(Periodic::new(Duration::seconds(PERIOD_PING_S as i64),
                                     PeriodicPing));
        periodics.push(Periodic::new(Duration::seconds(PERIOD_TIMEOUT_CHECK_S as i64),
                                     PeriodicTimeoutCheck));
        periodics.push(Periodic::new(Duration::seconds(PERIOD_ANNOUNCE_ADDRS_S as i64),
                                     PeriodicAnnounceAddresses));

        periodics
    }

    pub fn read_loop(&mut self) -> Result<(),PeerError>
    {
        let mut buffer : MsgBuffer = MsgBuffer::new();
        let mut last_periodic : Timespec = time::now_utc().to_timespec();
        let mut periodics : Vec<Periodic>;

        periodics = Peer::init_periodics();

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
                MsgVersion(version) => self.handle_version(version),
                MsgVerAck(verack)   => self.handle_verack(verack),
                MsgPing(ping)       => self.handle_ping(ping),
                MsgPong(pong)       => self.handle_pong(pong),
                MsgAddr(addrs)      => self.handle_addr(addrs),
                MsgInv(inv)         => self.handle_inv(inv),
                MsgGetData(getdata) => self.handle_getdata(getdata),
                MsgReject(reject)   => self.handle_reject(reject),
                MsgTx(tx)           => self.handle_tx(tx),
            };

            match result
            {
                Err(err) => if err.is_fatal() { return Err(err) },
                _        => ()
            }
        }
    }
}

enum PeriodicToken
{
    PeriodicPing,
    PeriodicTimeoutCheck,
    PeriodicAnnounceAddresses
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
 *                recv | send
 * ___________________________
 * version          F  |   F
 * verack           F  |   F
 * ping             F  |   F
 * pong             F  |   F
 * addr             F  |   F
 * getaddr             |
 * inv              P  |
 * getdata             |   P
 * reject           P  |
 * tx               P  |
 * block               |
 * notfound            |
 * getblocks           |
 * getheaders          |
 * headers             |
 *
 *
 * Later:
 *     filterload
 *     filteradd
 *     filterclear
 *     merkleblock
 *     mempool
 *     alert
 */
