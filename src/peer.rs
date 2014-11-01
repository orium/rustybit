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
use message::MsgGetAddr;

use message::version::Version;
use message::verack::VerAck;
use message::ping::Ping;
use message::pong::Pong;
use message::addr::Addr;
use message::inv::Inv;
use message::getdata::GetData;
use message::reject::Reject;
use message::tx::Tx;
use message::getaddr::GetAddr;

use datatype::invvect::InvVect;
use datatype::netaddr::NetAddr;

use msgbuffer::MsgBuffer;

use addrmng::AddrManagerChannel;
use addrmng::AddrManagerRequest;
use addrmng::AddrManagerReply;
use addrmng::AddrMngAddAddresses;
use addrmng::AddrMngGetSomeAddresses;
use addrmng::AddrMngGetManyAddresses;
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
    PingTimeout
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
static PERIOD_ANNOUNCE_ADDRS_S : uint = 15*60;
static PERIOD_REQUEST_ADDRS_S : uint = 30*60;

static TIMEOUT_S : uint = 10*60;

pub struct Peer
{
    addr            : SocketAddr,
    socket          : Option<TcpStream>,
    version         : Option<Version>,
    /* last time we sent (and we are waiting for the pong) */
    last_ping       : Option<Timespec>,
    /* last time we received an addr msg */
    last_addr       : Option<Timespec>,
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
            last_addr:       None,
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

        try!(self.send(&version.serialize()));

        ::logger::log_sent_msg(&self.addr,&MsgVersion(version));

        Ok(())
    }

    fn send_verack(&mut self) -> Result<(),PeerError>
    {
        let verack = VerAck::new();

        try!(self.send(&verack.serialize()));

        ::logger::log_sent_msg(&self.addr,&MsgVerAck(verack));

        Ok(())
    }

    fn send_ping(&mut self) -> Result<(),PeerError>
    {
        let now : Timespec = time::now_utc().to_timespec();
        let ping : Ping;

        assert!(self.last_ping == None);

        ping = Ping::new(((now.sec as u64)<<10) | ((now.nsec as u64)/1_000_000));

        try!(self.send(&ping.serialize()));

        self.last_ping = Some(now);

        ::logger::log_sent_msg(&self.addr,&MsgPing(ping));

        Ok(())
    }

    fn send_pong(&mut self, nounce : u64) -> Result<(),PeerError>
    {
        let pong = Pong::new(nounce);

        try!(self.send(&pong.serialize()));

        ::logger::log_sent_msg(&self.addr,&MsgPong(pong));

        Ok(())
    }

    fn send_getdata(&mut self, inv : &InvVect) -> Result<(),PeerError>
    {
        let getdata = GetData::from_inv(inv);

        try!(self.send(&getdata.serialize()));

        ::logger::log_sent_msg(&self.addr,&MsgGetData(getdata));

        Ok(())
    }

    fn send_addr(&mut self, addrs : &Vec<NetAddr>) -> Result<(),PeerError>
    {
        let addr = Addr::from_addrs(addrs);

        try!(self.send(&addr.serialize()));

        ::logger::log_sent_msg(&self.addr,&MsgAddr(addr));

        Ok(())
    }

    fn send_getaddr(&mut self) -> Result<(),PeerError>
    {
        let getaddr = GetAddr::new();

        try!(self.send(&getaddr.serialize()));

        ::logger::log_sent_msg(&self.addr,&MsgGetAddr(getaddr));

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
            self.addr_mng_send(AddrMngAddAddresses(self.addr.ip,singleton_addrs));
        }
    }

    fn handle_version(&mut self, version : Version) -> Result<(),PeerError>
    {
        /* Do not allow a peer send a version msg twice */
        if self.version.is_some()
        {
            return Err(DoubleHandshake);
        }

        if version.get_protocol_version() < ::config::PROTOCOL_VERSION_MIN
        {
            return Err(UnsupportedProtoVersion);
        }

        self.version = Some(version.clone());

        try!(self.send_verack());

        self.addr_mng_add_self();

        ::logger::log_received_msg(&self.addr,&MsgVersion(version));

        Ok(())
    }

    fn handle_verack(&mut self, verack : VerAck) -> Result<(),PeerError>
    {
        ::logger::log_received_msg(&self.addr,&MsgVerAck(verack));

        Ok(())
    }

    fn handle_ping(&mut self, ping : Ping) -> Result<(),PeerError>
    {
        try!(self.send_pong(ping.get_nounce()));

        ::logger::log_received_msg(&self.addr,&MsgPing(ping));

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

        ::logger::log_received_msg(&self.addr,&MsgPong(pong));

        ::logger::log_lag(&self.addr,&lag);

        Ok(())
    }

    fn handle_addr(&mut self, addr : Addr) -> Result<(),PeerError>
    {
        let now : Timespec = time::now_utc().to_timespec();

        self.addr_mng_send(AddrMngAddAddresses(self.addr.ip,
                                               addr.get_addresses().clone()));

        self.last_addr = Some(now);

        ::logger::log_received_msg(&self.addr,&MsgAddr(addr));

        Ok(())
    }

    fn handle_inv(&mut self, inv : Inv) -> Result<(),PeerError>
    {
        try!(self.send_getdata(inv.get_invvect()));

        ::logger::log_received_msg(&self.addr,&MsgInv(inv));

        Ok(())
    }

    fn handle_getdata(&mut self, getdata : GetData) -> Result<(),PeerError>
    {
        ::logger::log_received_msg(&self.addr,&MsgGetData(getdata));

        Ok(())
    }

    fn handle_reject(&mut self, reject : Reject) -> Result<(),PeerError>
    {
        ::logger::log_received_msg(&self.addr,&MsgReject(reject));

        Ok(())
    }

    fn handle_tx(&mut self, tx : Tx) -> Result<(),PeerError>
    {
        ::logger::log_received_msg(&self.addr,&MsgTx(tx));

        Ok(())
    }

    fn handle_getaddr(&mut self, getaddr : GetAddr) -> Result<(),PeerError>
    {
        try!(self.announce_addresses(true));

        ::logger::log_received_msg(&self.addr,&MsgGetAddr(getaddr));

        Ok(())
    }

    fn announce_addresses(&mut self, many : bool) -> Result<(),PeerError>
    {
        let request : AddrManagerRequest;
        let reply   : AddrManagerReply;

        request = if many { AddrMngGetManyAddresses }
                  else { AddrMngGetSomeAddresses };

        reply = self.addr_mng_send_recv(request);

        match reply
        {
            AddrMngAddresses(addrs) =>
            {
                assert!(addrs.len() <= ::message::addr::MSG_ADDR_MAX);

                self.send_addr(&addrs)
            },
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
        self.announce_addresses(false)
    }

    fn periodic_request_addrs(&mut self) -> Result<(),PeerError>
    {
        let now = time::now_utc().to_timespec();
        let then;

        if self.last_addr.is_none()
        {
            return self.send_getaddr();
        }

        then = self.last_addr.unwrap();

        if now > then+Duration::seconds(PERIOD_REQUEST_ADDRS_S as i64)
        {
            return self.send_getaddr();
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
                    PeriodicPing         => self.periodic_sendping(),
                    PeriodicTimeoutCheck => self.periodic_timeout_check(),
                    PeriodicAnnounceAddresses => self.periodic_announce_addrs(),
                    PeriodicRequestAddresses  => self.periodic_request_addrs()
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
        periodics.push(Periodic::new(Duration::seconds(PERIOD_REQUEST_ADDRS_S as i64),
                                     PeriodicRequestAddresses));

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
                MsgGetAddr(getaddr) => self.handle_getaddr(getaddr),
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
    PeriodicAnnounceAddresses,
    PeriodicRequestAddresses
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
    pub fn new(interval : Duration,
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
 * getaddr          F  |   F
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
