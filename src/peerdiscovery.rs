extern crate serialize;

/* TODO: rust-http is obsolete but the replacement (teepee) is not ready yet.
 *       When teepee is usable replace rust-http with teepee.
 */
extern crate http;
extern crate url;

use std::rand::Rng;

use std::io::net::ip::SocketAddr;
use std::io::net::ip::Ipv4Addr;
use std::io::net::ip::Ipv6Addr;

use std::iter::AdditiveIterator;

use self::serialize::json;

use self::url::Url;
use self::http::client::RequestWriter;
use self::http::method::Get;
use self::http::client::ResponseReader;

use std::io::net::addrinfo::get_host_addresses;

macro_rules! try_err_nil(
    ($e:expr) => (match $e { Ok(e) => e, Err(_) => return Err(()) }))

macro_rules! unwrap_err_nil(
    ($e:expr) => (match $e { Some(e) => e, None => return Err(()) }))

macro_rules! try_emp_vec(
    ($e:expr) => (match $e { Ok(e) => e, Err(_) => return Vec::new() }))

macro_rules! unwrap_emp_vec(
    ($e:expr) => (match $e { Some(e) => e, None => return Vec::new() }))

fn discover_hardcoded() -> Vec<SocketAddr>
{
    vec![ SocketAddr { ip: Ipv4Addr(93,93,135,12), port: 8333 }, /* UK */
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
          SocketAddr { ip: Ipv4Addr(82,209,206,37), port: 8333 }, /* Belarus */ ]
}

/* TODO: it might make sense to have a timeout in the http GET.
 */
fn json_from_url(url_str : String) -> Result<json::Json,()>
{
    let url : Url;
    let request : RequestWriter;
    let mut response : ResponseReader<_>;
    let body : Vec<u8>;
    let body_str : &str;

    url = Url::parse(url_str.as_slice()).ok().unwrap();
    request = RequestWriter::new(Get, url).unwrap();

    response = try_err_nil!(request.read_response());

    body = try_err_nil!(response.read_to_end());

    body_str = unwrap_err_nil!(::std::str::from_utf8(body.as_slice()));

    Ok(try_err_nil!(json::Builder::new(body_str.chars()).build()))
}

fn get_last_snapshot_getaddr_bitnodes_io() -> Result<String,()>
{
    let snapshot_index = "https://getaddr.bitnodes.io/api/v1/snapshots/";
    let root : json::Json;
    let results : &json::Json;
    let snapshots : &json::JsonList;
    let lastest_snapshots_url : &json::Json;

    root = try!(json_from_url(snapshot_index.to_string()));
    results = unwrap_err_nil!(root.find(&"results".to_string()));

    snapshots = unwrap_err_nil!(results.as_list());

    if snapshots.len() < 1
    {
        return Err(());
    }

    lastest_snapshots_url = unwrap_err_nil!(snapshots[0].find(&"url".to_string()));

    Ok(unwrap_err_nil!(lastest_snapshots_url.as_string()).to_string())
}

fn discover_getaddr_bitnodes_io() -> Vec<SocketAddr>
{
    let snapshot_url : String;
    let root : json::Json;
    let nodes : &json::Json;
    let mut peers = Vec::new();

    snapshot_url = try_emp_vec!(get_last_snapshot_getaddr_bitnodes_io());

    root = try_emp_vec!(json_from_url(snapshot_url));

    nodes = unwrap_emp_vec!(root.find(&"nodes".to_string()));

    for (addr, prop) in unwrap_emp_vec!(nodes.as_object()).iter()
    {
        let proto_ver;
        let sock_addr : SocketAddr;

        proto_ver = unwrap_emp_vec!(unwrap_emp_vec!(prop.as_list())[0].as_u64());

        if (proto_ver as u32) < ::config::PROTOCOL_VERSION_MIN
        {
            continue;
        }

        sock_addr = match from_str::<SocketAddr>(addr.as_slice()) {
            Some(addr) => addr,
            None    => continue
        };

        match sock_addr.ip
        {
            Ipv4Addr(..) => peers.push(sock_addr),
            Ipv6Addr(..) => () /* TODO check if configuration allows ipv6 addresses */
        }
    }

    peers
}

/* TODO: it might make sense to have a timeout in the http GET.
 */
fn discover_dns_lookup_seeds() -> Vec<SocketAddr>
{
    let hostnames = [ "seed.bitcoin.sipa.be",
                       "dnsseed.bluematt.me",
                       "dnsseed.bitcoin.dashjr.org",
                       "seed.bitcoinstats.com",
                       "seed.bitnodes.io",
                       "bitseed.xf2.org" ];
    let mut peers : Vec<SocketAddr> = Vec::new();

    for hostname in hostnames.iter()
    {
        let result = get_host_addresses(*hostname);

        if result.is_err()
        {
            continue;
        }

        for addr in result.ok().unwrap().iter()
        {
            match *addr
            {
                Ipv4Addr(..) =>
                {
                    let sock_addr = SocketAddr { ip: *addr,
                                                 port: ::config::DEFAULT_PORT };

                    peers.push(sock_addr);
                },
                Ipv6Addr(..) => () /* TODO check if configuration allows ipv6 addresses */
            }
        }
    }

    peers
}

pub fn discover_peers(count : uint) -> Vec<SocketAddr>
{
    let discovery_methods = [ discover_hardcoded,
                              discover_getaddr_bitnodes_io,
                              discover_dns_lookup_seeds ];
    let mut peers_by_method : Vec<Vec<SocketAddr>>;
    let mut peers : Vec<SocketAddr> = Vec::with_capacity(count);

    peers_by_method = discovery_methods.iter().map(|m| (*m)()).collect();

    for pm in peers_by_method.iter_mut()
    {
        ::crypto::rng().shuffle(pm.as_mut_slice());
    }

    assert!(peers_by_method.iter().map(|v| v.len()).sum() >= count);

    /* We take peers randomly from various peer discovery methods.  We make sure
     * that peers are uniformly distributed from each discovery method, since
     * different methods may obtain different numbers of peers, thus we garantee
     * fairness between methods.
     */
    while peers.len() < count
    {
        let method = (::crypto::rng().gen::<uint>())%peers_by_method.len();

        match peers_by_method.get_mut(method).pop()
        {
            Some(addr) => peers.push(addr),
            None       => ()
        }
    }

    peers
}
