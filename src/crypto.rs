extern crate openssl;

use std::rand::OsRng;

use self::openssl::crypto::hash::{SHA256,Hasher};

pub fn sha256(data : &Vec<u8>) -> Vec<u8>
{
    let hasher : Hasher = Hasher::new(SHA256);

    hasher.update(data.as_slice());

    hasher.finalize()
}

pub fn dsha256(data : &Vec<u8>) -> Vec<u8>
{
    sha256(&sha256(data))
}

pub fn checksum(data : &Vec<u8>) -> u32
{
    let mut unmarshalling : ::marshalling::Unmarshalling;
    let digest : Vec<u8> = dsha256(data);

    unmarshalling = ::marshalling::Unmarshalling::new(&digest);

    unmarshalling.read_uint32()
}

pub fn hash_to_hexstr(hash : &Vec<u8>) -> String
{
    let mut str : String = String::new();

    assert!(hash.len() == 32);

    for b in hash.iter()
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
