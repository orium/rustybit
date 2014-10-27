extern crate sync;

use std::fmt::Show;
use std::fmt::Formatter;

use std::comm::Handle;
use std::collections::HashSet;

use self::sync::comm::Receiver;
use self::sync::comm::SyncSender;
use self::sync::comm::Select;
use self::sync::comm::Empty;
use self::sync::comm::Disconnected;

use datatype::netaddr::NetAddr;

pub static ADDRMNG_CHANNEL_BUF_CAP : uint = 8;

static ANNOUNCE_ADDRS_MIN : uint = 20;
static ANNOUNCE_ADDRS_MAX : uint = 100;

pub type AddrManagerChannel
    = (SyncSender<AddrManagerRequest>, Receiver<AddrManagerReply>);

type PeerChannel
    = (SyncSender<AddrManagerReply>, Receiver<AddrManagerRequest>);

pub enum AddrManagerRequest
{
    AddrMngAddAddresses(Vec<NetAddr>),
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
            AddrMngAddAddresses(ref addrs) =>
                write!(f,"Add Addresses: {}",addrs),
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

pub struct AddrManager
{
    channels : Vec<PeerChannel>,
    addresses : HashSet<NetAddr>
}

impl AddrManager
{
    pub fn new(orchestrator : PeerChannel) -> AddrManager
    {
        let mut channels = Vec::new();

        channels.push(orchestrator);

        AddrManager
        {
            channels:  channels,
            addresses: HashSet::new()
        }
    }

    fn send(&self, channelid : uint, msg : AddrManagerReply)
    {
        let (ref sender, _) = self.channels[channelid];

        ::logger::log_addr_mng_reply(&msg);

        sender.send(msg);
    }

    fn handle_add_addresses(&mut self, addrs : Vec<NetAddr>)
    {
        for addr in addrs.iter()
        {
            self.addresses.insert(*addr);
        }

        ::logger::log_addr_mng(format!("addresses size: {}",
                                       self.addresses.len()).as_slice());

        if self.addresses.len() > 1000
        {
            let mut to_remove : Vec<NetAddr> = Vec::new();

            for addr in self.addresses.iter()
            {
                if ::crypto::rand_interval(0,1) == 0
                {
                    to_remove.push(*addr);
                }
            }

            for addr in to_remove.iter()
            {
                self.addresses.remove(addr);
            }

            ::logger::log_addr_mng(format!("addresses after cleanup: {}",
                                           self.addresses.len()).as_slice());
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

        for addr in self.addresses.iter().take(num)
        {
            if addr.is_valid_addr()
            {
                addrs.push(*addr);
            }
        }

        self.send(channelid,AddrMngAddresses(addrs));
    }

    fn handle_request(&mut self,
                      channelid : uint,
                      request   : AddrManagerRequest)
    {
        ::logger::log_addr_mng_request(&request);

        match request
        {
            AddrMngAddAddresses(addrs) => self.handle_add_addresses(addrs),
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

            /* We make sure no rellocation happens, otherwise we moved the handlers,
             * which cannot happen.  (It is required by the unsafe block above.)
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

/* TODO
 *
 * Goals:
 * 1. Only keep a limited number of addresses.
 * 2. Keep addresses fresh.
 * 3. Make sure no single attacker can fill the entire table.
 * 4. Make sure no attacker can fill the entire table with nodes/addresses
 *    he controls (assume nodes are geographically close).
 * 5. Always serve some tried addresses.
 *
 * Policy:
 *
 * * (2) Drop old addresses (with >= 3h)
 * * (3) Any peer can only add X addrs per hour
 * * (1) Have a pool of, at most, 4K addresses
 *       * (1,2) To drop addresses select older
 * * (5) Keep a small pool of tried addresses and serve at least X of them.
 * * (4) See bitcoin core
 *
 * How do we judge oldness? we cannot trust peers completly.
 */
