extern crate time;

use std::fmt::Show;
use std::fmt::Formatter;

use datatype::value::Value;
use datatype::script::Script;
use datatype::hash::Hash;

pub struct OutPoint
{
    hash  : Hash,
    index : u32      /* Index of the specific output in the transaction */
}

#[allow(dead_code)]
impl OutPoint
{
    pub fn new(hash : Hash, index : u32) -> OutPoint
    {
        OutPoint
        {
            hash:  hash,
            index: index
        }
    }

    pub fn get_hash(&self) -> &Hash
    {
        &self.hash
    }

    pub fn get_index(&self) -> u32
    {
        self.index
    }
}

impl Show for OutPoint
{
    fn fmt(&self, f : &mut Formatter) -> Result<(), ::std::fmt::Error>
    {
        let width = f.width().unwrap_or(0);
        let space = String::from_str(" ").repeat(width);

        write!(f,"{}{} idx: {}",space,self.hash,self.index)
    }
}

pub struct TxOut
{
    value  : Value,
    script : Script
}

#[allow(dead_code)]
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

    pub fn get_value(&self) -> &Value
    {
        &self.value
    }

    pub fn get_script(&self) -> &Script
    {
        &self.script
    }
}

impl Show for TxOut
{
    fn fmt(&self, f : &mut Formatter) -> Result<(), ::std::fmt::Error>
    {
        let width = f.width().unwrap_or(0);
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
    sequence   : u32       /* See http://bitcoin.stackexchange.com/q/2025/323 */
}

#[allow(dead_code)]
impl TxIn
{
    pub fn new(prev_out   : OutPoint,
               sig_script : Script,
               sequence   : u32) -> TxIn
    {
        TxIn
        {
            prev_out:   prev_out,
            sig_script: sig_script,
            sequence:   sequence
        }
    }

    pub fn get_prev_out(&self) -> &OutPoint
    {
        &self.prev_out
    }

    pub fn get_script(&self) -> &Script
    {
        &self.sig_script
    }

    pub fn get_sequence(&self) -> u32
    {
        self.sequence
    }
}

impl Show for TxIn
{
    fn fmt(&self, f : &mut Formatter) -> Result<(), ::std::fmt::Error>
    {
        let width = f.width().unwrap_or(0);
        let space = String::from_str(" ").repeat(width);

        try!(write!(f,"{}PrevOut  : {}\n",space,self.prev_out));
        try!(write!(f,"{}SigScript: {}\n",space,self.sig_script));
        try!(write!(f,"{}Sequence : {}",space,self.sequence));

        Ok(())
    }
}

#[deriving(Show)]
pub enum TxLock
{
    LockLocked,               /* Always locked */
    LockUnlocked,             /* Always unlocked */
    LockBlock(u32),
    LockTime(time::Timespec)
}

impl TxLock
{
    pub fn from_u32(v : u32) -> TxLock
    {
        match v
        {
            0                        => TxLock::LockLocked,
            1         ... 500000000  => TxLock::LockBlock(v),
            500000001 ... 0xfffffffe => TxLock::LockTime(time::Timespec { sec: v as i64,
                                                                          nsec: 0}),
            0xffffffff               => TxLock::LockUnlocked,
            _                        => unreachable!()
        }
    }
}

/* TODO impl Show for TemporalLock */

pub struct Transaction
{
    version : u32,
    txs_in  : Vec<TxIn>,
    txs_out : Vec<TxOut>,
    lock    : TxLock
}

#[allow(dead_code)]
impl Transaction
{
    pub fn new(version : u32,
               txs_in  : Vec<TxIn>,
               txs_out : Vec<TxOut>,
               lock    : TxLock) -> Transaction
    {
        Transaction
        {
            version: version,
            txs_in:  txs_in,
            txs_out: txs_out,
            lock:    lock
        }
    }

    pub fn get_hash(&self) -> Hash
    {
        Hash::new([0u8, ..32]) // TODO
    }

    pub fn get_version(&self) -> u32
    {
        self.version
    }

    pub fn get_in_txs(&self) -> &Vec<TxIn>
    {
        &self.txs_in
    }

    pub fn get_out_txs(&self) -> &Vec<TxOut>
    {
        &self.txs_out
    }

    pub fn get_lock(&self) -> TxLock
    {
        self.lock
    }
}

impl Show for Transaction
{
    fn fmt(&self, f : &mut Formatter) -> Result<(), ::std::fmt::Error>
    {
        let width = f.width().unwrap_or(0);
        let space = String::from_str(" ").repeat(width);


        try!(write!(f,"{}Hash    : {}\n",space,self.get_hash()));
        try!(write!(f,"{}Version : {}\n",space,self.version));
        try!(write!(f,"{}LockTime: {}\n",space,self.lock));

        try!(write!(f,"{}TxsIn\n",space));

        for tx_in in self.txs_in.iter()
        {
            // TODO this should be "{:2+space}"
            try!(write!(f,"{:8}\n",tx_in));
        }

        try!(write!(f,"{}TxsOut\n",space));

        for i in range(0,self.txs_out.len())
        {
            // TODO this should be "{:2+space}"
            try!(write!(f,"{:8}{}",self.txs_out[i],
                        if i == self.txs_out.len()-1 { "" } else { "\n" }));
        }

        Ok(())
    }
}
