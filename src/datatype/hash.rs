use std::fmt::Show;
use std::fmt::Formatter;

use std::clone::Clone;

pub struct Hash
{
    hash : [u8, ..32]
}

impl Hash
{
    pub fn new(hash : [u8, ..32]) -> Hash
    {
        Hash
        {
            hash: hash
        }
    }
}

impl Index<uint, u8> for Hash
{
    fn index(&self, index: &uint) -> &u8
    {
        &self.hash[*index]
    }
}

impl Show for Hash
{
    fn fmt(&self, f : &mut Formatter) -> Result<(), ::std::fmt::Error>
    {
        write!(f, "{}",::crypto::to_hexstr(&self.hash))
    }
}

impl Clone for Hash
{
    fn clone(&self) -> Hash
    {
        Hash::new(self.hash)
    }
}
