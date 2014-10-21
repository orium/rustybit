extern crate sync;

use std::comm::Handle;

use self::sync::comm::Receiver;
use self::sync::comm::Sender;
use self::sync::comm::Select;
use self::sync::comm::Empty;
use self::sync::comm::Disconnected;

use datatype::netaddr::NetAddr;

pub type AddressPoolChannel
    = (Sender<AddressPoolRequest>, Receiver<AddressPoolReply>);

type PeerChannel
    = (Sender<AddressPoolReply>, Receiver<AddressPoolRequest>);

pub enum AddressPoolRequest
{
    AddrPoolAddAddresses(Vec<NetAddr>),
    AddrPoolAddPeerChannel(Sender<AddressPoolReply>,
                           Receiver<AddressPoolRequest>)
}

#[deriving(Show)]
pub enum AddressPoolReply
{

}

pub struct AddressPoolManager
{
    channels : Vec<PeerChannel>
}

impl AddressPoolManager
{
    pub fn new(orchestrator : PeerChannel) -> AddressPoolManager
    {
        let mut channels = Vec::new();

        channels.push(orchestrator);

        AddressPoolManager
        {
            channels: channels
        }
    }

    fn handle_request(&mut self,
                      _channelid : uint,
                      request   : AddressPoolRequest)
    {
        match request
        {
            AddrPoolAddAddresses(addrs) =>
            {
                println!("Address Pool: Add addresses: {}",addrs);
            },
            AddrPoolAddPeerChannel(sender,receiver) =>
            {
                self.channels.push((sender,receiver));
                println!("Address Pool: Add peer channel");
            }
        }
    }

    fn wait(&self)
    {
        let sel : Select;
        let mut handlers : Vec<Handle<AddressPoolRequest>>;
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
                    Ok(request) =>
                    {
                        self.handle_request(i,request);
                    }
                    Err(Empty) => (),
                    Err(Disconnected) =>
                    {
                        println!("Address Pool: disconnect");

                        self.channels.remove(i);
                        break;
                    }
                }
            }
        }
    }
}
