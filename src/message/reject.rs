use std::fmt::Show;
use std::fmt::Formatter;

use message::header::Header;

#[deriving(Show)]
enum RejectType
{
    RejectMalformed       = 0x01,
    RejectInvalid         = 0x10,
    RejectObsolete        = 0x11,
    RejectDuplicate       = 0x12,
    RejectNonstandard     = 0x40,
    RejectDust            = 0x41,
    RejectInsufficientFee = 0x42,
    RejectCheckpoint      = 0x43
}

impl RejectType
{
    pub fn from_u8(v : u8) -> Option<RejectType>
    {
        match v
        {
            0x01 => Some(RejectType::RejectMalformed),
            0x10 => Some(RejectType::RejectInvalid),
            0x11 => Some(RejectType::RejectObsolete),
            0x12 => Some(RejectType::RejectDuplicate),
            0x40 => Some(RejectType::RejectNonstandard),
            0x41 => Some(RejectType::RejectDust),
            0x42 => Some(RejectType::RejectInsufficientFee),
            0x43 => Some(RejectType::RejectCheckpoint),
            _    => None
        }
    }
}

pub struct Reject
{
    msg    : String, /* message that cause this reject */
    typ    : RejectType,
    reason : String
}

#[allow(dead_code)]
impl Reject
{
    pub fn new(msg : String, typ : RejectType, reason : String) -> Reject
    {
        Reject
        {
            msg:    msg,
            typ:    typ,
            reason: reason
        }
    }

    pub fn unserialize(data : &Vec<u8>) -> Reject
    {
        let mut unmarshalling = ::marshalling::Unmarshalling::new(data);
        let msg : String;
        let typ : u8;
        let reason : String;

        msg = unmarshalling.read_varstr();
        typ = unmarshalling.read_uint8();
        reason = unmarshalling.read_varstr();

        assert!(RejectType::from_u8(typ).is_some());

        Reject
        {
            msg:    msg,
            typ:    RejectType::from_u8(typ).unwrap(),
            reason: reason
        }
    }

    pub fn serialize(&self) -> Vec<u8>
    {
        let mut msg = ::marshalling::Marshalling::new();
        let header : Header;

        msg.write_varstr(&self.msg);
        msg.write_uint8(self.typ as u8);
        msg.write_varstr(&self.reason);

        header = Header::new(::config::NETWORK,
                             "reject".to_string(),
                             msg.len() as u32,
                             ::crypto::checksum(msg.get().as_slice()));

        header.serialize() + msg.get()
    }
}

impl Show for Reject
{
    fn fmt(&self, f : &mut Formatter) -> Result<(), ::std::fmt::Error>
    {
        let width = f.width().unwrap_or(0);
        let space = String::from_str(" ").repeat(width);

        write!(f,"{}Reject {} {} \"{}\"", space, self.msg, self.typ, self.reason)
    }
}
