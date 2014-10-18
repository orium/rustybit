pub mod header;
pub mod version;
pub mod verack;
pub mod ping;
pub mod pong;
pub mod addr;
pub mod inv;
pub mod getdata;
pub mod reject;
pub mod tx;

pub enum Message
{
    MsgVersion(version::Version),
    MsgVerAck(verack::VerAck),
    MsgPing(ping::Ping),
    MsgPong(pong::Pong),
    MsgAddr(addr::Addr),
    MsgInv(inv::Inv),
    MsgGetData(getdata::GetData),
    MsgReject(reject::Reject),
    MsgTx(tx::Tx)
}
