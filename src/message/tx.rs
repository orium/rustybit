use std::fmt::Show;
use std::fmt::Formatter;

use datatype::transaction::Transaction;

use message::header::Header;

pub struct Tx
{
    tx : Transaction
}

#[allow(dead_code)]
impl Tx
{
    pub fn new(tx : Transaction) -> Tx
    {
        Tx
        {
            tx: tx
        }
    }

    pub fn serialize(&self) -> Vec<u8>
    {
        let mut msg = ::marshalling::Marshalling::new();
        let header : Header;

        msg.write_transaction(&self.tx);

        header = Header::new(::config::NETWORK,
                             "tx".to_string(),
                             msg.len() as u32,
                             ::crypto::checksum(msg.get().as_slice()));

        header.serialize() + msg.get()
    }

    pub fn unserialize(data : &Vec<u8>) -> Tx
    {
        let mut unmarshalling = ::marshalling::Unmarshalling::new(data);

        Tx::new(unmarshalling.read_transaction())
    }
}

impl Show for Tx
{
    fn fmt(&self, f : &mut Formatter) -> Result<(), ::std::fmt::Error>
    {
        let width = f.width().unwrap_or(0);
        let space = String::from_str(" ").repeat(width);

        try!(write!(f,"{}Transaction:\n", space));

        // TODO this should be "{:2+space}"
        write!(f,"{:6}", self.tx)
    }
}
