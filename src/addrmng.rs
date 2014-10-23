extern crate sync;

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

#[deriving(Show)]
pub enum AddrManagerReply
{
    AddrMngAddresses(Vec<NetAddr>)
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

        sender.send(msg);
    }

    fn handle_add_addresses(&mut self, addrs : Vec<NetAddr>)
    {
        let num : uint;

        println!("XXX Address Manager: Add addresses: {}",addrs);

        num = ::crypto::rand_interval(ANNOUNCE_ADDRS_MIN,ANNOUNCE_ADDRS_MAX);

        for addr in addrs.iter().take(num)
        {
            self.addresses.insert(*addr);
        }

        println!("XXX Address Manager: addresses size: {}",self.addresses.len());

        if self.addresses.len() > 1000
        {
            self.addresses.clear();
        }
    }

    fn handle_add_channel(&mut self, channel : PeerChannel)
    {
        self.channels.push(channel);
        println!("XXX Address Manager: Add peer channel");
    }

    fn handle_get_addrs(&self, channelid : uint)
    {
        let mut addrs : Vec<NetAddr> = Vec::with_capacity(20);

        for addr in self.addresses.iter()
        {
            if addr.is_valid_addr()
            {
                addrs.push(*addr);
            }
        }

        println!("XXX Address Manager: Get addresses: {}",addrs);

        self.send(channelid,AddrMngAddresses(addrs));
    }

    fn handle_request(&mut self,
                      channelid : uint,
                      request   : AddrManagerRequest)
    {
        match request
        {
            AddrMngAddAddresses(addrs) => self.handle_add_addresses(addrs),
            AddrMngGetAddresses => self.handle_get_addrs(channelid),
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
                        println!("XXX Address Manager: disconnect");

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
 * * Only keep a limited number of addresses around.
 *      say 4K addresses
 * * Make sure no (localized) attacker can fill the entire table with his
 *   nodes/addresses.
 */
