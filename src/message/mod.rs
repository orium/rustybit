pub mod header;
pub mod version;
pub mod versionack;
pub mod ping;
pub mod pong;
pub mod addresses;
pub mod inv;
pub mod getdata;

pub enum Message
{
    MsgVersion(version::Version),
    MsgVersionAck(versionack::VersionAck),
    MsgPing(ping::Ping),
    MsgPong(pong::Pong),
    MsgAddresses(addresses::Addresses),
    MsgInv(inv::Inv),
    MsgGetData(getdata::GetData),
}
