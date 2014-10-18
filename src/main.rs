#![feature(macro_rules)]

extern crate getopts;

use std::io::net::ip::SocketAddr;

use getopts::optflag;
use peerdiscovery::discover_peers;

mod config;
mod datatype;
mod marshalling;
mod crypto;
mod msgbuffer;
mod message;
mod peer;
mod peerdiscovery;

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

    Some(Options
         {
             help:    matches.opt_present("h"),
             version: matches.opt_present("v")
         })
}

fn handle_peer(address : SocketAddr) -> Result<(),peer::PeerError>
{
    let mut peer : peer::Peer = peer::Peer::new(address);

    try!(peer.connect());
    try!(peer.send_version());

    peer.read_loop()
}

fn spawn_thread_handle_peer(address : SocketAddr)
{
    spawn(proc() {
        match handle_peer(address)
        {
            Err(err) =>
            {
                (write!(std::io::stderr(),"{} Error: {}\n",address,err)).unwrap();
            },
            _        => unreachable!()
        }
    });
}

fn main()
{
    let mut peers : Vec<SocketAddr>;
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

    peers = discover_peers(config::INITIAL_DISCOVERY_PEERS);

    /* For testing */
    peers.push(SocketAddr { ip: std::io::net::ip::Ipv4Addr(127,0,0,1),   port: 8333 });
    peers.push(SocketAddr { ip: std::io::net::ip::Ipv4Addr(192,168,1,2), port: 8333 });
    peers.reverse();

    for peer in peers.iter()
    {
        spawn_thread_handle_peer(*peer);
    }
}

/* TODO:
 *
 *  * There are asserts that need to be verified in runtime and handled
 *    gracefully instead of terminating the program
 *    (eg. Unmarshalling::read_strvar()).
 *  * Logger
 */
