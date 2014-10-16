extern crate openssl;

use std::path::Path;
use std::io::File;

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

pub fn rand() -> u64
{
    /* TODO: For some reason the OsRnd does not seem to work, so we do this here.
     */
    let mut reader;
    let v;

    reader = File::open(&Path::new("/dev/urandom"));

    assert!(reader.is_ok());

    v = reader.ok().unwrap().read_le_u64();

    assert!(v.is_ok());

    v.ok().unwrap()
}

