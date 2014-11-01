extern crate sync;

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

use std::iter::AdditiveIterator;

use std::rand::Rng;

use std::comm::Handle;
use std::collections::HashSet;
use std::collections::HashMap;

use self::sync::comm::Receiver;
use self::sync::comm::SyncSender;
use self::sync::comm::Select;
use self::sync::comm::Empty;
use self::sync::comm::Disconnected;

use datatype::netaddr::NetAddr;

pub static ADDRMNG_CHANNEL_BUF_CAP : uint = 8;

static MAX_ADDRESSES : uint = 2500;
static BUCKETS : uint = 64;

static MAX_ADDRS_PER_PEER_PERCENT : f32 = 0.02;

static ANNOUNCE_ADDRS_MIN : uint = 10;
static ANNOUNCE_ADDRS_MAX : uint = 50;

pub type AddrManagerChannel
    = (SyncSender<AddrManagerRequest>, Receiver<AddrManagerReply>);

type PeerChannel
    = (SyncSender<AddrManagerReply>, Receiver<AddrManagerRequest>);

pub enum AddrManagerRequest
{
    AddrMngAddAddresses(IpAddr, Vec<NetAddr>),
    AddrMngAddPeerChannel(SyncSender<AddrManagerReply>,
                          Receiver<AddrManagerRequest>),
    AddrMngGetAddresses
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
            AddrMngGetAddresses =>
                write!(f,"Addresses request")
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

impl Eq for Address
{
}

pub struct AddrManager
{
    channels       : Vec<PeerChannel>,
    addresses      : Vec<HashSet<Address>>,
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
            addresses:      Vec::from_fn(BUCKETS, |_| HashSet::new()),
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

    /* We only take into account the /12 subnet.
     */
    fn get_bucket_idx(&self, addr : &NetAddr) -> uint
    {
        let mut data : Vec<u8> = Vec::new();

        data.push_all(&self.secret);

        match addr.addr
        {
            Some(SocketAddr { ip: Ipv6Addr(..), port: _ }) =>
                unimplemented!(), /* TODO */
            Some(SocketAddr { ip: Ipv4Addr(h0,h1,_,_), port: _ }) =>
            {
                data.push(h0);
                data.push(h1&0xf0);
            },
            _  => unreachable!()
        }

        data.push_all(&self.secret);

        (::crypto::hash_first_u32(data.as_slice()) as uint)%BUCKETS
    }

    fn bucket_cleanup(&mut self, bucket : uint)
    {
        let mut to_remove : Vec<Address> = Vec::new();

        // TODO make this not aweful
        for addr in self.addresses[bucket].iter()
        {
            if ::crypto::rand_interval(0,8) == 0
            {
                to_remove.push(*addr);
            }
        }

        for addr in to_remove.iter()
        {
            self.addresses.get_mut(bucket).remove(addr);
            self.dec_peer_addresses(&addr.peer);
        }

        println!("bucket cleanup: dropped {} addresses",to_remove.len());
    }

    fn add_address(&mut self, peer : &IpAddr, addr : NetAddr)
    {
        let bucket : uint = self.get_bucket_idx(&addr);
        let address : Address = Address::new(addr,peer.clone());
        let new_addr : bool;

        assert!(self.allow_peer_to_add(peer));

        println!("bucket: {}",bucket);

        new_addr = self.addresses.get_mut(bucket).insert(address);

        if new_addr
        {
            self.inc_peer_addresses(peer);
        }

        if self.addresses[bucket].len() > MAX_ADDRESSES/BUCKETS
        {
            self.bucket_cleanup(bucket);
        }

        ::logger::log_addr_mng(format!("addresses: {}",
                                       self.addresses.iter()
                                                     .map(|b| b.len())
                                                     .collect::<Vec<uint>>())
                                                     .as_slice());
        ::logger::log_addr_mng(format!("addresses size: {}",
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

        if num == 0
        {
            self.addrs_per_peer.insert(*peer,num);
        }
        else
        {
            self.addrs_per_peer.remove(peer);
        }
    }

    fn allow_peer_to_add(&self, peer : &IpAddr) -> bool
    {
        match self.addrs_per_peer.find(peer)
        {
            Some(num) =>
                *num <= (MAX_ADDRS_PER_PEER_PERCENT*(MAX_ADDRESSES as f32)) as uint,
            None      => true
        }
    }

    fn handle_add_addresses(&mut self, peer : IpAddr, addrs : Vec<NetAddr>)
    {
        for addr in addrs.iter().filter(|addr| addr.is_valid_addr())
        {
            if !self.allow_peer_to_add(&peer)
            {
                break;
            }

            self.add_address(&peer,*addr);
        }
    }

    fn handle_add_channel(&mut self, channel : PeerChannel)
    {
        self.channels.push(channel);
    }

    fn handle_get_addrs(&self, channelid : uint)
    {
        let num = ::crypto::rand_interval(ANNOUNCE_ADDRS_MIN,ANNOUNCE_ADDRS_MAX);
        let mut addrs : Vec<NetAddr> = Vec::with_capacity(num);

        for _ in range(0,5*num)
        {
            let bucket = ::crypto::rand_interval(0,BUCKETS-1);
            let amount = ::crypto::rand_interval(2,4); // TODO constants?
            let mut candidates : Vec<&Address>;

            if addrs.len() >= num
            {
                break;
            }

            candidates=self.addresses[bucket].iter().collect();

            ::crypto::rng().shuffle(candidates.as_mut_slice());

            for addr in candidates.iter().take(amount)
            {
                addrs.push(addr.netaddr.clone());
            }
        }

        /* Shuffle addrs to minimize the bucket-related information leaked */
        ::crypto::rng().shuffle(addrs.as_mut_slice());

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
            AddrMngGetAddresses        => self.handle_get_addrs(channelid),
            AddrMngAddPeerChannel(s,r) => self.handle_add_channel((s,r))
        }
    }

    fn wait(&self)
    {
        let sel : Select;
        let mut handlers : Vec<Handle<AddrManagerRequest>>;
        let cap;

        sel = Select::new();
        /* This really needed with_capacity().  See the unsafe block bellow.
         */
        handlers = Vec::with_capacity(self.channels.len());

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

        sel.wait();

        for handler in handlers.iter_mut()
        {
            unsafe { handler.remove(); }
        }
    }

    pub fn read_loop(&mut self)
    {
        loop
        {
            self.wait();

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
 *
 * How do we judge oldness? we cannot trust peers completly.
 */

/* TODO:
 *
 * * (2) Drop old addresses (with >= X hrs)
 *       * (1,2) To drop addresses select older
 * * (5) Keep a small pool of tried addresses and serve at least X of them.
 */
