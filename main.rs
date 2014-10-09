use std::io::net::ip::SocketAddr;
use std::io::net::ip::Ipv4Addr;

mod config;
mod marshalling;
mod message;
mod peer;

fn main()
{
    let address : SocketAddr = SocketAddr { ip: Ipv4Addr(192, 168, 1, 2),
                                            port: 8333 };
    let mut orion : peer::Peer = peer::Peer::new(address);

    orion.connect();
    orion.send_version();
}
