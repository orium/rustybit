#![feature(macro_rules)]

extern crate sync;
extern crate getopts;

use std::io::net::ip::SocketAddr;
use std::io::timer::sleep;
use std::time::duration::Duration;

use getopts::optflag;
use peerdiscovery::discover_peers;

use sync::comm::channel;

use addresspool::AddressPoolChannel;
use addresspool::AddressPoolManager;
use addresspool::AddressPoolRequest;
use addresspool::AddressPoolReply;
use addresspool::AddrPoolAddPeerChannel;

mod config;
mod datatype;
mod marshalling;
mod crypto;
mod msgbuffer;
mod message;
mod peer;
mod peerdiscovery;
mod addresspool;

struct Options
{
    help    : bool,
    version : bool
}

pub static OPT_DESC_HELP : &'static str
    = "Display this help and exit";
pub static OPT_DESC_VERSION : &'static str
    = "Output version information and exit";

#[allow(unused_must_use)]
fn print_usage(out : &mut std::io::LineBufferedWriter<std::io::stdio::StdWriter>)
{
    let program : &String = &std::os::args()[0];

    print_version(out);

    write!(out,"Usage: {} [OPTIONS]\n", program);
    write!(out,"\n");
    write!(out,"Options:\n");
    write!(out,"  -h, --help     {}\n",OPT_DESC_HELP);
    write!(out,"  -v, --version  {}\n",OPT_DESC_VERSION);
}

#[allow(unused_must_use)]
fn print_version(out : &mut std::io::LineBufferedWriter<std::io::stdio::StdWriter>)
{
    write!(out, "{}\n",config::name_version());
}

fn parse_options() -> Option<Options>
{
    let opts = [ optflag("h", "help",    OPT_DESC_HELP),
                 optflag("v", "version", OPT_DESC_VERSION) ];
    let matches : getopts::Matches;

    matches = match getopts::getopts(std::os::args().as_slice(), opts) {
        Ok(m) => m,
        Err(e) => {
            (write!(std::io::stderr(),"error: {}\n", e)).unwrap();
            return None;
        }
    };

    if matches.free.len() > 1
    {
        print_usage(&mut std::io::stderr());
        return None;
    }

    Some(Options { help:    matches.opt_present("h"),
                   version: matches.opt_present("v") })
}

fn run_peer(address           : SocketAddr,
            addr_pool_channel : AddressPoolChannel) -> Result<(),peer::PeerError>
{
    let mut peer : peer::Peer = peer::Peer::new(address,addr_pool_channel);

    try!(peer.connect());
    try!(peer.send_version());

    peer.read_loop()
}

fn spawn_thread_run_peer(address           : SocketAddr,
                         addr_pool_channel : AddressPoolChannel)
{
    spawn(proc() {
        match run_peer(address,addr_pool_channel)
        {
            Err(err) =>
            {
                (write!(std::io::stderr(),"{} Error: {}\n",address,err)).unwrap();
            },
            _        => unreachable!()
        }
    });
}

fn spawn_thread_run_address_pool_manager(sender   : Sender<AddressPoolReply>,
                                         receiver : Receiver<AddressPoolRequest>)
{
    spawn(proc() {
        let mut addr_pool_mng : AddressPoolManager;

        addr_pool_mng = AddressPoolManager::new((sender,receiver));

        addr_pool_mng.read_loop();
    });
}

fn run_peers()
{
    let mut addrs : Vec<SocketAddr>;
    let (send_our, recv_poolmsg) = channel();
    let (send_poolmsg, _recv_our) = channel();

    addrs = discover_peers(config::INITIAL_DISCOVERY_PEERS);

    /* For testing */
    addrs.push(SocketAddr { ip: std::io::net::ip::Ipv4Addr(127,0,0,1),   port: 8333 });
    addrs.push(SocketAddr { ip: std::io::net::ip::Ipv4Addr(192,168,1,2), port: 8333 });
    addrs.reverse();

    spawn_thread_run_address_pool_manager(send_poolmsg,recv_poolmsg);

    for addrs in addrs.iter()
    {
        let (send_c, recv_s) = channel();
        let (send_s, recv_c) = channel();

        send_our.send(AddrPoolAddPeerChannel(send_s,recv_s));

        spawn_thread_run_peer(*addrs,(send_c,recv_c));
    }

    /* Loop while we occasionally add more peers */
    loop
    {
        sleep(Duration::seconds(60)); // XXX TODO
    }
}

fn main()
{
    let options : Options;

    options = match parse_options() {
        Some(opt) => opt,
        None      =>
        {
            std::os::set_exit_status(-1);
            return;
        }
    };

    if options.help
    {
        print_usage(&mut std::io::stdout());
        return;
    }
    else if options.version
    {
        print_version(&mut std::io::stdout());
        return;
    }

    run_peers();
}

/* TODO:
 *
 *  * There are asserts that need to be verified in runtime and handled
 *    gracefully instead of terminating the task
 *    (eg. Unmarshalling::read_strvar()).
 *  * Logger
 *  * Carefuly audit block consensus to be 100% equal to the core implementation
 *     * Test block acceptence: https://github.com/TheBlueMatt/test-scripts
 *     * There will be an official concesus library. When that's ready, use it.
 */
