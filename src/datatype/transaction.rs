extern crate time;

use std::fmt::Show;
use std::fmt::Formatter;

use datatype::value::Value;
use datatype::script::Script;

pub struct OutPoint
{
    hash  : Vec<u8>,
    index : u32      /* Index of the specific output in the transaction */
}

impl OutPoint
{
    pub fn new(hash : Vec<u8>, index : u32) -> OutPoint
    {
        OutPoint
        {
            hash:  hash,
            index: index
        }
    }
}

impl Show for OutPoint
{
    fn fmt(&self, f : &mut Formatter) -> Result<(), ::std::fmt::FormatError>
    {
        let width = if f.width.is_some() { f.width.unwrap() } else { 0 };
        let space = String::from_str(" ").repeat(width);

        try!(write!(f,"{}Hash : {}\n",space,::crypto::hash_to_hexstr(&self.hash)));
        try!(write!(f,"{}Index: {}",space,self.index));

        Ok(())
    }
}

pub struct TxOut
{
    value  : Value,
    script : Script
}

impl TxOut
{
    pub fn new(value : Value, script : Script) -> TxOut
    {
        TxOut
        {
            value:  value,
            script: script
        }
    }
}

impl Show for TxOut
{
    fn fmt(&self, f : &mut Formatter) -> Result<(), ::std::fmt::FormatError>
    {
        let width = if f.width.is_some() { f.width.unwrap() } else { 0 };
        let space = String::from_str(" ").repeat(width);

        try!(write!(f,"{}Value : {}\n",space,self.value));
        try!(write!(f,"{}Script: {}",space,self.script));

        Ok(())
    }
}

pub struct TxIn
{
    prev_out   : OutPoint,
    sig_script : Script,
    sequence   : u32       /* http://bitcoin.stackexchange.com/q/2025/323 */
}

impl TxIn
{
    pub fn new(prev_out   : OutPoint,
               sig_script : Script,
               sequence   : u32) -> TxIn
    {
        TxIn
        {
            prev_out   : prev_out,
            sig_script : sig_script,
            sequence   : sequence
        }
    }
}

impl Show for TxIn
{
    fn fmt(&self, f : &mut Formatter) -> Result<(), ::std::fmt::FormatError>
    {
        let width = if f.width.is_some() { f.width.unwrap() } else { 0 };
        let space = String::from_str(" ").repeat(width);

        try!(write!(f,"{}PrevOut  : {}\n",space,self.prev_out));
        try!(write!(f,"{}SigScript: {}\n",space,self.sig_script));
        try!(write!(f,"{}Sequence : {}",space,self.sequence));

        Ok(())
    }
}

#[deriving(Show)]
pub enum TemporalLock
{
    LockBlock(u32),
    LockTime(time::Timespec),
}

pub struct Tx
{
    version   : u32,
    txs_in    : Vec<TxIn>,
    txs_out   : Vec<TxOut>,
    lock_time : TemporalLock
}

impl Tx
{
    pub fn new(version   : u32,
               txs_in    : Vec<TxIn>,
               txs_out   : Vec<TxOut>,
               lock_time : TemporalLock) -> Tx
    {
        Tx
        {
            version   : version,
            txs_in    : txs_in,
            txs_out   : txs_out,
            lock_time : lock_time
        }
    }
}

impl Show for Tx
{
    fn fmt(&self, f : &mut Formatter) -> Result<(), ::std::fmt::FormatError>
    {
        let width = if f.width.is_some() { f.width.unwrap() } else { 0 };
        let space = String::from_str(" ").repeat(width);

        try!(write!(f,"{}Version:  {}\n",space,self.version));

        try!(write!(f,"{}TxIn\n",space));

        for tx_in in self.txs_in.iter()
        {
            try!(write!(f,"{}{:4}\n",space,tx_in));
        }

        try!(write!(f,"{}TxOut\n",space));

        for tx_out in self.txs_out.iter()
        {
            try!(write!(f,"{}{:4}\n",space,tx_out));
        }

        try!(write!(f,"{}LockTime: {}",space,self.lock_time));

        Ok(())
    }
}
