extern crate time;

use std::io::net::ip::SocketAddr;

use message::Message;

use std::time::duration::Duration;
use self::time::Timespec;

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

const LOG_FLAGS : u64 =
    0
    | LogFlag::LogFlagPeerError as u64
//    | LogFlag::LogFlagMsgVersion as u64
//    | LogFlag::LogFlagMsgVerAck as u64
//    | LogFlag::LogFlagMsgPing as u64
//    | LogFlag::LogFlagMsgPong as u64
    | LogFlag::LogFlagMsgAddr as u64
//    | LogFlag::LogFlagMsgInv as u64
//    | LogFlag::LogFlagMsgGetData as u64
    | LogFlag::LogFlagMsgReject as u64
//    | LogFlag::LogFlagMsgTx as u64
    | LogFlag::LogFlagMsgGetAddr as u64
//    | LogFlag::LogFlagLag as u64
    | LogFlag::LogFlagAddrMng as u64
    ;

fn msg_to_command(msg : &Message) -> &str
{
    match *msg
    {
        Message::MsgVersion(_) => "version",
        Message::MsgVerAck(_)  => "verack",
        Message::MsgPing(_)    => "ping",
        Message::MsgPong(_)    => "pong",
        Message::MsgAddr(_)    => "addr",
        Message::MsgInv(_)     => "Message::inv",
        Message::MsgGetData(_) => "getdata",
        Message::MsgReject(_)  => "reject",
        Message::MsgTx(_)      => "tx",
        Message::MsgGetAddr(_) => "getaddr",
    }
}

pub fn log_received_msg(addr : &SocketAddr, msg : &Message)
{
    match *msg
    {
        Message::MsgVersion(_) =>
            if LOG_FLAGS & LogFlag::LogFlagMsgVersion as u64 == 0 { return; },
        Message::MsgVerAck(_)  =>
            if LOG_FLAGS & LogFlag::LogFlagMsgVerAck as u64 == 0  { return; },
        Message::MsgPing(_)    =>
            if LOG_FLAGS & LogFlag::LogFlagMsgPing as u64 == 0    { return; },
        Message::MsgPong(_)    =>
            if LOG_FLAGS & LogFlag::LogFlagMsgPong as u64 == 0    { return; },
        Message::MsgAddr(_)    =>
            if LOG_FLAGS & LogFlag::LogFlagMsgAddr as u64 == 0    { return; },
        Message::MsgInv(_)     =>
            if LOG_FLAGS & LogFlag::LogFlagMsgInv as u64 == 0     { return; },
        Message::MsgGetData(_) =>
            if LOG_FLAGS & LogFlag::LogFlagMsgGetData as u64 == 0 { return; },
        Message::MsgReject(_)  =>
            if LOG_FLAGS & LogFlag::LogFlagMsgReject as u64 == 0  { return; },
        Message::MsgTx(_)      =>
            if LOG_FLAGS & LogFlag::LogFlagMsgTx as u64 == 0      { return; },
        Message::MsgGetAddr(_) =>
            if LOG_FLAGS & LogFlag::LogFlagMsgGetAddr as u64 == 0 { return; },
    }

    println!(">>> {}  {} command: {:9}",
             time::now().rfc822z(),addr,msg_to_command(msg));

    match *msg
    {
        Message::MsgVersion(ref version) => println!("{:4}",version),
        Message::MsgVerAck(ref verack)   => println!("{:4}",verack),
        Message::MsgPing(ref ping)       => println!("{:4}",ping),
        Message::MsgPong(ref pong)       => println!("{:4}",pong),
        Message::MsgAddr(ref addrs)      => println!("{:4}",addrs),
        Message::MsgInv(ref inv)         => println!("{:4}",inv),
        Message::MsgGetData(ref getdata) => println!("{:4}",getdata),
        Message::MsgReject(ref reject)   => println!("{:4}",reject),
        Message::MsgTx(ref tx)           => println!("{:4}",tx),
        Message::MsgGetAddr(ref getaddr) => println!("{:4}",getaddr),
    }
}

pub fn log_sent_msg(addr : &SocketAddr, msg : &Message)
{
    match *msg
    {
        Message::MsgVersion(_) =>
            if LOG_FLAGS & LogFlag::LogFlagMsgVersion as u64 == 0 { return; },
        Message::MsgVerAck(_)  =>
            if LOG_FLAGS & LogFlag::LogFlagMsgVerAck as u64 == 0  { return; },
        Message::MsgPing(_)    =>
            if LOG_FLAGS & LogFlag::LogFlagMsgPing as u64 == 0    { return; },
        Message::MsgPong(_)    =>
            if LOG_FLAGS & LogFlag::LogFlagMsgPong as u64 == 0    { return; },
        Message::MsgAddr(_)    =>
            if LOG_FLAGS & LogFlag::LogFlagMsgAddr as u64 == 0    { return; },
        Message::MsgInv(_)     =>
            if LOG_FLAGS & LogFlag::LogFlagMsgInv as u64 == 0     { return; },
        Message::MsgGetData(_) =>
            if LOG_FLAGS & LogFlag::LogFlagMsgGetData as u64 == 0 { return; },
        Message::MsgReject(_)  =>
            if LOG_FLAGS & LogFlag::LogFlagMsgReject as u64 == 0  { return; },
        Message::MsgTx(_)      =>
            if LOG_FLAGS & LogFlag::LogFlagMsgTx as u64 == 0      { return; },
        Message::MsgGetAddr(_) =>
            if LOG_FLAGS & LogFlag::LogFlagMsgGetAddr as u64 == 0 { return; },
    }

    println!("<<< {}  {} command: {:9}",
             time::now().rfc822z(),addr,msg_to_command(msg));

    match *msg
    {
        Message::MsgVersion(ref version) => println!("{:4}",version),
        Message::MsgVerAck(ref verack)   => println!("{:4}",verack),
        Message::MsgPing(ref ping)       => println!("{:4}",ping),
        Message::MsgPong(ref pong)       => println!("{:4}",pong),
        Message::MsgAddr(ref addrs)      => println!("{:4}",addrs),
        Message::MsgInv(ref inv)         => println!("{:4}",inv),
        Message::MsgGetData(ref getdata) => println!("{:4}",getdata),
        Message::MsgReject(ref reject)   => println!("{:4}",reject),
        Message::MsgTx(ref tx)           => println!("{:4}",tx),
        Message::MsgGetAddr(ref getaddr) => println!("{:4}",getaddr),
    }
}

pub fn log_lag(addr : &SocketAddr, lag : &Duration)
{
    if LOG_FLAGS & LogFlag::LogFlagLag as u64 != 0
    {
        println!("{}  Lag: {} ms",addr,lag.num_milliseconds());
    }
}

pub fn log_peer_error_fatal(addr : &SocketAddr, err : ::peer::PeerError)
{
    assert!(err.is_fatal());

    if LOG_FLAGS & LogFlag::LogFlagPeerError as u64 != 0
    {
        (write!(&mut ::std::io::stderr(),"{} Fatal Error: {}\n",addr,err)).unwrap();
    }
}

pub fn log_addr_mng_request(request : &::addrmng::AddrManagerRequest)
{
    if LOG_FLAGS & LogFlag::LogFlagAddrMng as u64 != 0
    {
        println!("Address Manager: Request: {}",request);
    }
}

pub fn log_addr_mng_reply(reply : &::addrmng::AddrManagerReply)
{
    if LOG_FLAGS & LogFlag::LogFlagAddrMng as u64 != 0
    {
        println!("Address Manager: Reply: {}",reply);
    }
}

pub fn log_addr_mng_disconnect()
{
    if LOG_FLAGS & LogFlag::LogFlagAddrMng as u64 != 0
    {
        println!("Address Manager: Channel disconnected");
    }
}

pub fn log_addr_mng_buckets<T : Iterator<uint>>(buckets : &mut T)
{
    if LOG_FLAGS & LogFlag::LogFlagAddrMng as u64 != 0
    {
        let mut count : uint = 0;

        for b in *buckets
        {
            if count%8 == 0
            {
                if count > 0 { println!(""); }
                print!("Address Manager: Buckets {}:",count/8);
            }

            print!(" {:2}",b);
            count += 1;
        }

        println!("");
    }
}

pub fn log_addr_mng_timestamp_update(addr : &SocketAddr,
                                     old : &Timespec,
                                     new : &Timespec)
{
    if LOG_FLAGS & LogFlag::LogFlagAddrMng as u64 != 0
    {
        println!("Address Manager: Updated address {} from {} to {}",
                 addr,old.sec,new.sec);
    }
}

pub fn log_addr_mng_address_count(count : uint)
{
    if LOG_FLAGS & LogFlag::LogFlagAddrMng as u64 != 0
    {
        println!("Address Manager: Address count: {}",count);
    }
}

pub fn log_addr_mng_cleanup(count_before : uint, count_after : uint)
{
    if LOG_FLAGS & LogFlag::LogFlagAddrMng as u64 != 0
    {
        println!("Address Manager: Cleanup: {} -> {}",count_before,count_after);
    }
}
