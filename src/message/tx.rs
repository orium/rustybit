use std::fmt::Show;
use std::fmt::Formatter;

use datatype::transaction::Transaction;
use datatype::transaction::LockLocked;
use datatype::transaction::LockUnlocked;
use datatype::transaction::LockBlock;
use datatype::transaction::LockTime;

use datatype::transaction::TxIn;
use datatype::transaction::TxOut;
use datatype::transaction::TxLock;
use datatype::transaction::OutPoint;

use datatype::value::Value;
use datatype::value::Satoshi;

use datatype::script::Script;

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

        msg.write_uint32(self.tx.get_version());

        msg.write_varint(self.tx.get_in_txs().len() as u64);

        for in_tx in self.tx.get_in_txs().iter()
        {
            let prev_out : &OutPoint = in_tx.get_prev_out();

            msg.write_hash(prev_out.get_hash());
            msg.write_uint32(prev_out.get_index());

            msg.write_varint(0u64); /* XXX TODO */
            /* TODO XXX write script */
            msg.write_uint32(in_tx.get_sequence());
        }

        msg.write_varint(self.tx.get_out_txs().len() as u64);

        for out_tx in self.tx.get_out_txs().iter()
        {
            match *out_tx.get_value()
            {
                Satoshi(v) => msg.write_uint64(v)
            }

            msg.write_varint(0u64); /* XXX TODO */
            /* TODO XXX write script */
        }

        match self.tx.get_lock()
        {
            LockLocked       => msg.write_uint32(0),
            LockUnlocked     => msg.write_uint32(0xffffffff),
            LockBlock(block) => msg.write_uint32(block),
            LockTime(tm)     => msg.write_uint32(tm.sec as u32)
        }

        header = Header::new(::config::NETWORK,
                             "tx".to_string(),
                             msg.len() as u32,
                             ::crypto::checksum(&msg.get()));

        header.serialize() + msg.get()
    }

    pub fn unserialize(data : &Vec<u8>) -> Tx
    {
        let mut unmarshalling = ::marshalling::Unmarshalling::new(data);
        let version : u32;
        let mut txs_in : Vec<TxIn> = Vec::new();
        let mut txs_out : Vec<TxOut> = Vec::new();
        let lock : TxLock;

        version = unmarshalling.read_uint32();

        for _ in range(0,unmarshalling.read_varint())
        {
            let mut prev_out_hash : Vec<u8>;
            let prev_out : OutPoint;
            let script_len : u64;
            let sig_script : Script;
            let sequence : u32;

            prev_out_hash = unmarshalling.read_hash();

            prev_out = OutPoint::new(prev_out_hash,
                                     unmarshalling.read_uint32());

            script_len = unmarshalling.read_varint();

            /* TODO assert script_len < somethigh */

            unmarshalling.skip(script_len as uint); /* TODO XXX read script */
            sig_script = Script::new(); /* TODO XXX read script */

            sequence = unmarshalling.read_uint32();

            txs_in.push(TxIn::new(prev_out,sig_script,sequence));
        }

        // out
        for _ in range(0,unmarshalling.read_varint())
        {
            let value : Value;
            let script_len : u64;
            let script : Script;

            value = Satoshi(unmarshalling.read_uint64());

            script_len = unmarshalling.read_varint(); /* XXX read script */
            unmarshalling.skip(script_len as uint); /* XXX read script */
            script = Script::new(); /* XXX read script */

            txs_out.push(TxOut::new(value,script));
        }

        lock = TxLock::from_u32(unmarshalling.read_uint32());

        Tx::new(Transaction::new(version,txs_in,txs_out,lock))
    }
}

impl Show for Tx
{
    fn fmt(&self, f : &mut Formatter) -> Result<(), ::std::fmt::FormatError>
    {
        let width = if f.width.is_some() { f.width.unwrap() } else { 0 };
        let space = String::from_str(" ").repeat(width);

        try!(write!(f,"{}Transaction:\n", space));

        // TODO this should be "{:2+space}"
        write!(f,"{:6}", self.tx)
    }
}
