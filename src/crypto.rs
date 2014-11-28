extern crate openssl;

use std::rand::Rng;
use std::rand::OsRng;

use self::openssl::crypto::hash::{SHA256,Hasher};

pub fn sha256(data : &[u8]) -> [u8, ..32]
{
    let mut hasher : Hasher = Hasher::new(SHA256);
    let mut hash : [u8, ..32] = [0u8, ..32];
    let digest;

    hasher.update(data);

    digest = hasher.finalize();

    for i in range(0,32)
    {
        hash[i] = digest[i];
    }

    hash
}

pub fn dsha256(data : &[u8]) -> [u8, ..32]
{
    sha256(&sha256(data))
}

pub fn hash_first_u32(data : &[u8]) -> u32
{
    let digest : [u8, ..32] = dsha256(data);
    let mut checksum : u32 = 0;

    /* TODO use marshalling when it uses slices */
    for i in range(0u,4)
    {
        checksum |= digest[i] as u32 << 8*i;
    }

    checksum
}

pub fn checksum(data : &[u8]) -> u32
{
    hash_first_u32(data)
}

pub fn to_hexstr(data : &[u8]) -> String
{
    let mut str : String = String::new();

    assert!(data.len() == 32);

    for b in data.iter()
    {
        str.push_str(format!("{:02x}",*b).as_slice());
    }

    str
}

pub fn rng() -> OsRng
{
    let rng = OsRng::new();

    assert!(rng.is_ok());

    rng.unwrap()
}

/* Return a random uint between low (inclusive) and max (inclusive).
 */
pub fn rand_interval(low : uint, max : uint) -> uint
{
    low+(rng().gen::<uint>()%(max-low+1))
}
