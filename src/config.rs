pub static NAME : &'static str = "rustybit";

pub static VERSION_MAJOR : u8 = 0;
pub static VERSION_MINOR : u8 = 0;
pub static VERSION_FIXES : u8 = 0;

pub enum Network
{
    MainNet = 0xD9B4BEF9
}

pub static NETWORK : u32 = MainNet as u32; /* This should be of type Network */

pub static PROTOCOL_VERSION : u32 = 70002;

pub enum Service
{
    None        = 0,
    NodeNetwork = 1 << 0,
}

pub type Services = u64;

pub static SERVICES : Services = NodeNetwork as Services;

pub fn version() -> String
{
    format!("{}.{}.{}",VERSION_MAJOR,VERSION_MINOR,VERSION_FIXES)
}

#[allow(dead_code)]
pub fn name_version() -> String
{
    format!("{} {}",NAME,version())
}

/* Format the version according to BIP0014.
 * https://en.bitcoin.it/wiki/BIP_0014
 */
pub fn name_version_bip0014() -> String
{
    format!("/{}:{}/",NAME,version())
}
