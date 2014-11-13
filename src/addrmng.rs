extern crate sync;
extern crate time;

use std::fmt::Show;
use std::fmt::Formatter;

use std::hash::Hash;
use std::hash::sip::SipState;
use std::cmp::Eq;
use std::cmp::PartialEq;

use std::io::net::ip::SocketAddr;
use std::io::net::ip::IpAddr;
use std::io::net::ip::Ipv4Addr;
use std::io::net::ip::Ipv6Addr;
use std::io::Timer;
use std::time::duration::Duration;
use std::iter::AdditiveIterator;
use std::rand::Rng;
use std::comm::Handle;
use std::collections::HashMap;

use self::time::Timespec;

use self::sync::comm::Receiver;
use self::sync::comm::SyncSender;
use self::sync::comm::Select;
use self::sync::comm::Empty;
use self::sync::comm::Disconnected;

use datatype::netaddr::NetAddr;
use crypto::rand_interval;

pub static ADDRMNG_CHANNEL_BUF_CAP : uint = 8;

static MAX_ADDRESSES : uint = 2500;
static BUCKETS : uint = 64;

static MAX_ADDRS_PER_PEER : uint = (0.02*(MAX_ADDRESSES as f32)) as uint;
static MAX_ADDRS_PER_BUCKET : uint = MAX_ADDRESSES/BUCKETS;

static ANNOUNCE_SOME_ADDRS_MIN : uint = 5;
static ANNOUNCE_SOME_ADDRS_MAX : uint = 25;

static ANNOUNCE_MANY_ADDRS_MIN : uint = 200;
static ANNOUNCE_MANY_ADDRS_MAX : uint = 500;

static PERIODIC_CLEANUP_M : uint = 20;
static OLD_ADDRESS_AGE_M : uint = 3*60;

pub type AddrManagerChannel
    = (SyncSender<AddrManagerRequest>, Receiver<AddrManagerReply>);

type PeerChannel
    = (SyncSender<AddrManagerReply>, Receiver<AddrManagerRequest>);

pub enum AddrManagerRequest
{
    AddrMngAddAddresses(IpAddr, Vec<NetAddr>),
    AddrMngAddPeerChannel(SyncSender<AddrManagerReply>,
                          Receiver<AddrManagerRequest>),
    AddrMngGetSomeAddresses,
    AddrMngGetManyAddresses
}

pub enum AddrManagerReply
{
    AddrMngAddresses(Vec<NetAddr>)
}

impl Show for AddrManagerRequest
{
    fn fmt(&self, f : &mut Formatter) -> Result<(), ::std::fmt::FormatError>
    {
        match *self
        {
            AddrMngAddAddresses(ref peer, ref addrs) =>
                write!(f,"{}: Adds Addresses: {}",peer,addrs),
            AddrMngAddPeerChannel(_,_) =>
                write!(f,"New channel"),
            AddrMngGetSomeAddresses =>
                write!(f,"Some addresses request"),
            AddrMngGetManyAddresses =>
                write!(f,"Many addresses request")
        }
    }
}

impl Show for AddrManagerReply
{
    fn fmt(&self, f : &mut Formatter) -> Result<(), ::std::fmt::FormatError>
    {
        match *self
        {
            AddrMngAddresses(ref addrs) => write!(f,"Addresses: {}",addrs)
        }
    }
}

#[deriving(Clone)]
struct Address
{
    pub peer    : IpAddr,
    pub netaddr : NetAddr
}

impl Address
{
    pub fn new(netaddr : NetAddr, peer : IpAddr) -> Address
    {
        Address
        {
            peer:    peer,
            netaddr: netaddr
        }
    }

    pub fn is_old(&self) -> bool
    {
        let now : Timespec = time::now_utc().to_timespec();
        let age : Duration;

        assert!(self.netaddr.time.is_some());

        age = now-self.netaddr.time.unwrap();

        age > Duration::minutes(OLD_ADDRESS_AGE_M as i64)
    }
}

impl Hash for Address
{
    fn hash(&self, state: &mut SipState)
    {
        self.netaddr.addr.hash(state);
    }
}

impl PartialEq for Address
{
    fn eq(&self, other: &Address) -> bool
    {
        self.netaddr.addr == other.netaddr.addr
    }
}

impl Eq for Address {}

pub struct AddrManager
{
    channels       : Vec<PeerChannel>,
    addresses      : Vec<HashMap<SocketAddr,Address>>,
    addrs_per_peer : HashMap<IpAddr,uint>,
    secret         : [u8, ..256]
}

impl AddrManager
{
    pub fn new(orchestrator : PeerChannel) -> AddrManager
    {
        let mut channels = Vec::with_capacity(512);
        let mut secret : [u8, ..256] = [0u8, ..256];

        channels.push(orchestrator);

        ::crypto::rng().fill_bytes(&mut secret);

        AddrManager
        {
            channels:       channels,
            addresses:      Vec::from_fn(BUCKETS, |_| HashMap::new()),
            addrs_per_peer: HashMap::with_capacity(512),
            secret:         secret
        }
    }

    fn send(&self, channelid : uint, msg : AddrManagerReply)
    {
        let (ref sender, _) = self.channels[channelid];

        ::logger::log_addr_mng_reply(&msg);

        sender.send(msg);
    }

    #[allow(dead_code)]
    fn is_consistent(&self) -> bool
    {
        for (addr, num) in self.addrs_per_peer.iter()
        {
            let mut num_count : uint = 0;

            for bucket in self.addresses.iter()
            {
                num_count += bucket.iter()
                    .filter(|&(_,a)| a.peer == *addr).count();
            }

            if num_count != *num
            {
                println!("Error: Inconsistency: should be {}, is {}",num_count,num);
                return false;
            }
        }

        true
    }

    /* We only take into account the /12 subnet.
     */
    fn get_bucket_idx(&self, socketaddr : &SocketAddr) -> uint
    {
        let mut data : Vec<u8> = Vec::new();

        data.push_all(&self.secret);

        match *socketaddr
        {
            SocketAddr { ip: Ipv6Addr(..), port: _ } =>
                unimplemented!(), /* TODO */
            SocketAddr { ip: Ipv4Addr(h0,h1,_,_), port: _ } =>
            {
                data.push(h0);
                data.push(h1&0xf0);
            }
        }

        data.push_all(&self.secret);

        (::crypto::hash_first_u32(data.as_slice()) as uint)%BUCKETS
    }

    fn bucket_cleanup(&mut self, bucket : uint)
    {
        let mut to_remove : Vec<Address> = Vec::new();

        for (_, address) in self.addresses[bucket].iter()
        {
            if address.is_old()
            {
                to_remove.push(*address);
            }
        }

        for address in to_remove.iter()
        {
            let socketaddr : &SocketAddr = &address.netaddr.addr.unwrap();

            self.addresses.get_mut(bucket).remove(socketaddr);
            self.dec_peer_addresses(&address.peer);
        }

        println!("bucket cleanup: dropped {} addresses",to_remove.len());
    }

    fn get_known_address_time(&self, socketaddr : &SocketAddr) -> Option<Timespec>
    {
        let bucket : uint = self.get_bucket_idx(socketaddr);
        let known_addr : Option<&Address>;

        known_addr = self.addresses[bucket].find(socketaddr);

        known_addr.map(|address| address.netaddr.time.unwrap())
    }

    fn update_address_timestamp(&mut self, address : Address)
    {
        let socketaddr : &SocketAddr = &address.netaddr.addr.unwrap();
        let bucket : uint = self.get_bucket_idx(socketaddr);
        let old : Address;
        let new_addr : bool;

        /* We might be changing the peer of the address */
        old = self.addresses[bucket].find(socketaddr).unwrap().clone();
        self.dec_peer_addresses(&old.peer);

        assert!(old.netaddr.time.unwrap() < address.netaddr.time.unwrap());

        new_addr = self.addresses.get_mut(bucket).insert(*socketaddr,address);
        self.inc_peer_addresses(&address.peer);

        ::logger::log_addr_mng(format!("updated address {} from {} to {}",
                                       socketaddr,
                                       old.netaddr.time.unwrap().sec,
                                       address.netaddr.time.unwrap().sec)
                               .as_slice());

        assert!(!new_addr);
    }

    fn add_address(&mut self, address : Address)
    {
        let socketaddr : &SocketAddr = &address.netaddr.addr.unwrap();
        let bucket : uint = self.get_bucket_idx(socketaddr);
        let known_addr_time : Option<time::Timespec>;
        let new_addr : bool;

        known_addr_time = self.get_known_address_time(socketaddr);

        /* Refresh known address timestamp */
        if known_addr_time.is_some()
        {
            if known_addr_time.unwrap() < address.netaddr.time.unwrap()
            {
                self.update_address_timestamp(address);
            }

            return;
        }

        if !self.allow_peer_to_add(&address.peer)
            || self.addresses[bucket].len() > MAX_ADDRS_PER_BUCKET
        {
            return;
        }

        println!("bucket: {}",bucket);

        assert!(known_addr_time.is_none());
        assert!(!address.is_old());

        new_addr = self.addresses.get_mut(bucket).insert(*socketaddr,address);
        self.inc_peer_addresses(&address.peer);

        assert!(new_addr);

        ::logger::log_addr_mng(format!("addresses: {}",
                                       self.addresses.iter()
                                                     .map(|b| b.len())
                                                     .collect::<Vec<uint>>())
                                                     .as_slice());
        ::logger::log_addr_mng(format!("number of addresses: {}",
                                       self.addresses.iter()
                                                     .map(|b| b.len())
                                                     .sum()).as_slice());
    }

    fn inc_peer_addresses(&mut self, peer : &IpAddr)
    {
        let num : uint;

        if !self.addrs_per_peer.contains_key(peer)
        {
            self.addrs_per_peer.insert(*peer,0);
        }

        num = *self.addrs_per_peer.find(peer).unwrap() + 1;

        self.addrs_per_peer.insert(*peer,num);
    }

    fn dec_peer_addresses(&mut self, peer : &IpAddr)
    {
        let num : uint;

        assert!(self.addrs_per_peer.contains_key(peer));

        num = *self.addrs_per_peer.find(peer).unwrap() - 1;

        if num > 0
        {
            self.addrs_per_peer.insert(*peer,num);
        }
        else
        {
            assert!(num == 0);
            self.addrs_per_peer.remove(peer);
        }
    }

    fn allow_peer_to_add(&self, peer : &IpAddr) -> bool
    {
        match self.addrs_per_peer.find(peer)
        {
            Some(num) => *num <= MAX_ADDRS_PER_PEER,
            None      => true
        }
    }

    fn handle_add_addresses(&mut self, peer : IpAddr, addrs : Vec<NetAddr>)
    {
        /* TODO How about netaddr.services?
         */
        for addr in addrs.iter().filter(|addr| addr.is_valid_addr())
        {
            let address : Address = Address::new(*addr,peer.clone());

            assert!(addr.time.is_some());

            if !address.is_old()
            {
                self.add_address(address);
            }
        }
    }

    fn handle_add_channel(&mut self, channel : PeerChannel)
    {
        self.channels.push(channel);
    }

    fn get_addrs(&self, num : uint) -> Vec<NetAddr>
    {
        let mut addrs : Vec<NetAddr> = Vec::with_capacity(num);

        for _ in range(0,5*num)
        {
            let bucket = rand_interval(0,BUCKETS-1);
            let left   = addrs.len()-num;
            let amount = ::std::cmp::min(left,rand_interval(2,4));
            let mut candidates : Vec<&Address>;

            if addrs.len() >= num
            {
                break;
            }

            assert!(amount > 0);

            candidates = self.addresses[bucket].values().collect();

            ::crypto::rng().shuffle(candidates.as_mut_slice());

            for addr in candidates.iter().take(amount)
            {
                addrs.push(addr.netaddr.clone());
            }
        }

        /* Shuffle addrs to minimize the bucket-related information leaked */
        ::crypto::rng().shuffle(addrs.as_mut_slice());

        addrs
    }

    fn handle_get_some_addrs(&self, channelid : uint)
    {
        let num;
        let addrs;

        num = rand_interval(ANNOUNCE_SOME_ADDRS_MIN,ANNOUNCE_SOME_ADDRS_MAX);
        addrs = self.get_addrs(num);

        self.send(channelid,AddrMngAddresses(addrs));
    }

    fn handle_get_many_addrs(&self, channelid : uint)
    {
        let num;
        let addrs;

        num = rand_interval(ANNOUNCE_MANY_ADDRS_MIN,ANNOUNCE_MANY_ADDRS_MAX);
        addrs = self.get_addrs(num);

        self.send(channelid,AddrMngAddresses(addrs));
    }

    fn handle_request(&mut self,
                      channelid : uint,
                      request   : AddrManagerRequest)
    {
        ::logger::log_addr_mng_request(&request);

        match request
        {
            AddrMngAddAddresses(peer,addrs) =>
                self.handle_add_addresses(peer,addrs),
            AddrMngGetSomeAddresses    => self.handle_get_some_addrs(channelid),
            AddrMngGetManyAddresses    => self.handle_get_many_addrs(channelid),
            AddrMngAddPeerChannel(s,r) => self.handle_add_channel((s,r))
        }
    }

    fn periodic_cleanup(&mut self)
    {
        ::logger::log_addr_mng(format!("Before periodic cleanup addresses: {}",
                                       self.addresses.iter()
                                       .map(|b| b.len())
                                       .sum())
                               .as_slice());

        for i in range(0,BUCKETS)
        {
            self.bucket_cleanup(i);
        }

        ::logger::log_addr_mng(format!("After periodic cleanup addresses: {}",
                                       self.addresses.iter()
                                       .map(|b| b.len())
                                       .sum())
                               .as_slice());

    }

    fn wait(&self, periodic : &Receiver<()>)
    {
        let sel : Select;
        let mut handlers : Vec<Handle<AddrManagerRequest>>;
        let mut periodic_handler : Handle<()>;
        let cap : uint;

        sel = Select::new();
        /* This really needed with_capacity().  See the unsafe block bellow.
         */
        handlers = Vec::with_capacity(self.channels.len()+1);

        cap = handlers.capacity();

        /* handlers indices must match self.clients */
        for channel in self.channels.iter()
        {
            let (_, ref receiver) = *channel;

            handlers.push(sel.handle(receiver));

            /* We make sure no rellocation happens, otherwise we moved the
             * handlers, which cannot happen.  (It is required by the unsafe
             * block above.)
             */
            assert_eq!(cap,handlers.capacity());

            /* Handlers cannot be moved after they are added to select.
             */
            unsafe { handlers.last_mut().unwrap().add(); }
        }

        periodic_handler = sel.handle(periodic);
        unsafe { periodic_handler.add(); }

        sel.wait();

        for handler in handlers.iter_mut()
        {
            unsafe { handler.remove(); }
        }
    }

    pub fn read_loop(&mut self)
    {
        let mut cleanup_timer : Timer = Timer::new().unwrap();
        let cleanup_periodic : Receiver<()>;

        cleanup_periodic = cleanup_timer.periodic(Duration::minutes(PERIODIC_CLEANUP_M as i64));

        loop
        {
            self.wait(&cleanup_periodic);

            if cleanup_periodic.try_recv().is_ok()
            {
                self.periodic_cleanup();
            }

            for i in range(0,self.channels.len())
            {
                let maybereq = self.channels[i].ref1().try_recv();

                match maybereq
                {
                    Ok(request)       => self.handle_request(i,request),
                    Err(Empty)        => (),
                    Err(Disconnected) =>
                    {
                        ::logger::log_addr_mng("disconnected");

                        self.channels.remove(i);
                        break;
                    }
                }
            }
        }
    }
}

/* Goals:
 *
 * 1. Only keep a limited number of addresses.
 * 2. Keep addresses fresh.
 * 3. Make sure no single attacker can fill the entire table.
 * 4. Make sure no attacker can fill the entire table with nodes/addresses
 *    he controls (assume nodes are geographically close).
 * 5. Always serve some tried addresses.
 * 6. Serve addresses that are geographically spread
 *
 * Policy:
 *
 * * (2) Drop old addresses (with >= X hrs)
 * * (3) Any peer cannot have more than X% of the addrs added by him
 * * (1) Have a pool of, at most, 1K addresses
 *       * (1,2) To drop addresses select older
 * * (5) Keep a small pool of tried addresses and serve at least X of them.
 * * (4) Use /12 subnet to determine one of the buckets.
 *       Use cryptographic key so an attacker doesn't know how to fill a
 *       specific bucket.
 * * (6) When serving ips avoid serving more that X addrs from each bucket.
 */

/* TODO:
 *
 * * (5) Keep a small pool of tried addresses and serve at least X of them.
 *
 * How do we judge oldness? we cannot trust peers completly.
 */
