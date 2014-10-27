extern crate time;

use std::io::net::ip::SocketAddr;

use message::Message;
use message::MsgVersion;
use message::MsgVerAck;
use message::MsgPing;
use message::MsgPong;
use message::MsgAddr;
use message::MsgInv;
use message::MsgGetData;
use message::MsgReject;
use message::MsgTx;
use message::MsgGetAddr;

use std::time::duration::Duration;

enum LogFlag
{
    LogFlagPeerError  = 1 <<  0,
    LogFlagMsgVersion = 1 <<  1,
    LogFlagMsgVerAck  = 1 <<  2,
    LogFlagMsgPing    = 1 <<  3,
    LogFlagMsgPong    = 1 <<  4,
    LogFlagMsgAddr    = 1 <<  5,
    LogFlagMsgInv     = 1 <<  6,
    LogFlagMsgGetData = 1 <<  7,
    LogFlagMsgReject  = 1 <<  8,
    LogFlagMsgTx      = 1 <<  9,
    LogFlagMsgGetAddr = 1 << 10,
    LogFlagLag        = 1 << 11,
    LogFlagAddrMng    = 1 << 12
}

static LOG_FLAGS : u64 =
    0
    | LogFlagPeerError as u64
//    | LogFlagMsgVersion as u64
//    | LogFlagMsgVerAck as u64
//    | LogFlagMsgPing as u64
//    | LogFlagMsgPong as u64
    | LogFlagMsgAddr as u64
//    | LogFlagMsgInv as u64
//    | LogFlagMsgGetData as u64
    | LogFlagMsgReject as u64
//    | LogFlagMsgTx as u64
    | LogFlagMsgGetAddr as u64
//    | LogFlagLag as u64
    | LogFlagAddrMng as u64
    ;

fn msg_to_command(msg : &Message) -> &str
{
    match *msg
    {
        MsgVersion(_) => "version",
        MsgVerAck(_)  => "verack",
        MsgPing(_)    => "ping",
        MsgPong(_)    => "pong",
        MsgAddr(_)    => "addr",
        MsgInv(_)     => "inv",
        MsgGetData(_) => "getdata",
        MsgReject(_)  => "reject",
        MsgTx(_)      => "tx",
        MsgGetAddr(_) => "getaddr",
    }
}

pub fn log_received_msg(addr : &SocketAddr, msg : &Message)
{
    match *msg
    {
        MsgVersion(_) => if LOG_FLAGS & LogFlagMsgVersion as u64 == 0 { return; },
        MsgVerAck(_)  => if LOG_FLAGS & LogFlagMsgVerAck as u64 == 0  { return; },
        MsgPing(_)    => if LOG_FLAGS & LogFlagMsgPing as u64 == 0    { return; },
        MsgPong(_)    => if LOG_FLAGS & LogFlagMsgPong as u64 == 0    { return; },
        MsgAddr(_)    => if LOG_FLAGS & LogFlagMsgAddr as u64 == 0    { return; },
        MsgInv(_)     => if LOG_FLAGS & LogFlagMsgInv as u64 == 0     { return; },
        MsgGetData(_) => if LOG_FLAGS & LogFlagMsgGetData as u64 == 0 { return; },
        MsgReject(_)  => if LOG_FLAGS & LogFlagMsgReject as u64 == 0  { return; },
        MsgTx(_)      => if LOG_FLAGS & LogFlagMsgTx as u64 == 0      { return; },
        MsgGetAddr(_) => if LOG_FLAGS & LogFlagMsgGetAddr as u64 == 0 { return; },
    }

    println!(">>> {}  {} command: {:9}",
             time::now().rfc822z(),addr,msg_to_command(msg));

    match *msg
    {
        MsgVersion(ref version) => println!("{:4}",version),
        MsgVerAck(ref verack)   => println!("{:4}",verack),
        MsgPing(ref ping)       => println!("{:4}",ping),
        MsgPong(ref pong)       => println!("{:4}",pong),
        MsgAddr(ref addrs)      => println!("{:4}",addrs),
        MsgInv(ref inv)         => println!("{:4}",inv),
        MsgGetData(ref getdata) => println!("{:4}",getdata),
        MsgReject(ref reject)   => println!("{:4}",reject),
        MsgTx(ref tx)           => println!("{:4}",tx),
        MsgGetAddr(ref getaddr) => println!("{:4}",getaddr),
    }
}

pub fn log_sent_msg(addr : &SocketAddr, msg : &Message)
{
    match *msg
    {
        MsgVersion(_) => if LOG_FLAGS & LogFlagMsgVersion as u64 == 0 { return; },
        MsgVerAck(_)  => if LOG_FLAGS & LogFlagMsgVerAck as u64 == 0  { return; },
        MsgPing(_)    => if LOG_FLAGS & LogFlagMsgPing as u64 == 0    { return; },
        MsgPong(_)    => if LOG_FLAGS & LogFlagMsgPong as u64 == 0    { return; },
        MsgAddr(_)    => if LOG_FLAGS & LogFlagMsgAddr as u64 == 0    { return; },
        MsgInv(_)     => if LOG_FLAGS & LogFlagMsgInv as u64 == 0     { return; },
        MsgGetData(_) => if LOG_FLAGS & LogFlagMsgGetData as u64 == 0 { return; },
        MsgReject(_)  => if LOG_FLAGS & LogFlagMsgReject as u64 == 0  { return; },
        MsgTx(_)      => if LOG_FLAGS & LogFlagMsgTx as u64 == 0      { return; },
        MsgGetAddr(_) => if LOG_FLAGS & LogFlagMsgGetAddr as u64 == 0 { return; },
    }

    println!("<<< {}  {} command: {:9}",
             time::now().rfc822z(),addr,msg_to_command(msg));

    match *msg
    {
        MsgVersion(ref version) => println!("{:4}",version),
        MsgVerAck(ref verack)   => println!("{:4}",verack),
        MsgPing(ref ping)       => println!("{:4}",ping),
        MsgPong(ref pong)       => println!("{:4}",pong),
        MsgAddr(ref addrs)      => println!("{:4}",addrs),
        MsgInv(ref inv)         => println!("{:4}",inv),
        MsgGetData(ref getdata) => println!("{:4}",getdata),
        MsgReject(ref reject)   => println!("{:4}",reject),
        MsgTx(ref tx)           => println!("{:4}",tx),
        MsgGetAddr(ref getaddr) => println!("{:4}",getaddr),
    }
}

pub fn log_lag(addr : &SocketAddr, lag : &Duration)
{
    if LOG_FLAGS & LogFlagLag as u64 != 0
    {
        println!("{}  Lag: {} ms",addr,lag.num_milliseconds());
    }
}

pub fn log_peer_error_fatal(addr : &SocketAddr, err : ::peer::PeerError)
{
    assert!(err.is_fatal());

    if LOG_FLAGS & LogFlagPeerError as u64 != 0
    {
        (write!(::std::io::stderr(),"{} Fatal Error: {}\n",addr,err)).unwrap();
    }
}

pub fn log_addr_mng_request(request : &::addrmng::AddrManagerRequest)
{
    if LOG_FLAGS & LogFlagAddrMng as u64 != 0
    {
        println!("Address Manager Request: {}",request);
    }
}

pub fn log_addr_mng_reply(reply : &::addrmng::AddrManagerReply)
{
    if LOG_FLAGS & LogFlagAddrMng as u64 != 0
    {
        println!("Address Manager Reply: {}",reply);
    }
}

pub fn log_addr_mng(str : &str)
{
    if LOG_FLAGS & LogFlagAddrMng as u64 != 0
    {
        println!("Address Manager: {}",str);
    }
}
