#![feature(macro_rules)]

use std::io::net::ip::SocketAddr;
use std::io::net::ip::Ipv4Addr;

mod config;
mod marshalling;
mod message;
mod peer;

macro_rules! try_proc(
    ($e:expr) => (if $e.is_err() { return; })
)

fn spawn_thread_handle_peer(address : SocketAddr)
{
    spawn(proc() {
        let mut peer : peer::Peer = peer::Peer::new(address);

        try_proc!(peer.connect());
        try_proc!(peer.send_version());

        try_proc!(peer.read_loop());
    });
}

fn main()
{
    let peers = [SocketAddr { ip: Ipv4Addr(192,168,1,2),  port: 8333 },
                 //SocketAddr { ip: Ipv4Addr(93,93,135,12), port: 8333 }
                 ];

    for i in range(0,peers.len())
    {
        spawn_thread_handle_peer(peers[i]);
    }
}
