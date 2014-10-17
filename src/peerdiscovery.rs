use std::rand::Rng;

use std::io::net::ip::SocketAddr;
use std::io::net::ip::Ipv4Addr;

use std::iter::AdditiveIterator;

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

fn discover_getaddr_bitnodes_io() -> Vec<SocketAddr>
{
    Vec::new()
}

pub fn discover_peers(count : uint) -> Vec<SocketAddr>
{
    let discovery_methods = [ discover_hardcoded,
                              discover_getaddr_bitnodes_io ];
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
            None => ()
        }
    }

    peers
}
