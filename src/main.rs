#![feature(macro_rules)]

extern crate getopts;

use std::io::net::ip::SocketAddr;
use std::io::net::ip::Ipv4Addr;

use getopts::optflag;

mod config;
mod datatype;
mod marshalling;
mod crypto;
mod msgbuffer;
mod message;
mod peer;

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
    let peers = [ SocketAddr { ip: Ipv4Addr(127,0,0,1),    port: 8333 },
                  SocketAddr { ip: Ipv4Addr(192,168,1,2),  port: 8333 },
                  SocketAddr { ip: Ipv4Addr(93,93,135,12), port: 8333 }, /* UK */
                  SocketAddr { ip: Ipv4Addr(93,93,135,12), port: 8333 }, /* UK */
                  SocketAddr { ip: Ipv4Addr(70,69,238,84), port: 8333 }, /* Canada */
                  SocketAddr { ip: Ipv4Addr(54,232,98,22), port: 8333 }, /* Brazil */
                  SocketAddr { ip: Ipv4Addr(5,9,7,180),    port: 8333 }, /* Germany */
                  SocketAddr { ip: Ipv4Addr(217,69,224,209), port: 8333 }, /* Germany */
                  SocketAddr { ip: Ipv4Addr(54,252,97,50), port: 8333 }, /* Australia */
                  SocketAddr { ip: Ipv4Addr(103,248,189,97), port: 8333 }, /* Australia */
                  SocketAddr { ip: Ipv4Addr(54,245,235,252), port: 8333 }, /* US */
                  SocketAddr { ip: Ipv4Addr(66,114,33,250), port: 8333 }, /* US */
                  SocketAddr { ip: Ipv4Addr(204,27,61,162), port: 8333 }, /* US */
                  SocketAddr { ip: Ipv4Addr(192,198,92,99), port: 8333 }, /* US */
                  SocketAddr { ip: Ipv4Addr(91,220,163,18), port: 8333 }, /* Ukraine */
                  SocketAddr { ip: Ipv4Addr(193,107,19,83), port: 8333 }, /* Russia */
                  SocketAddr { ip: Ipv4Addr(5,100,123,19), port: 8333 }, /* Russia */
                  SocketAddr { ip: Ipv4Addr(195,197,175,190), port: 8333 }, /* Finland */
                  SocketAddr { ip: Ipv4Addr(188,126,8,14), port: 8333 }, /* Bulgaria */
                  SocketAddr { ip: Ipv4Addr(77,234,129,233), port: 8333 }, /* Slovenia */
                  SocketAddr { ip: Ipv4Addr(176,241,243,15), port: 8333 }, /* Poland */
                  SocketAddr { ip: Ipv4Addr(149,210,133,244), port: 8333 }, /* Netherlands */
                  SocketAddr { ip: Ipv4Addr(54,246,85,246), port: 8333 }, /* Ireland */
                  SocketAddr { ip: Ipv4Addr(82,209,206,37), port: 8333 }, /* Belarus */
                 ];
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
