pub static NAME : &'static str = "rustybit";

pub static VERSION_MAJOR : u8 = 0;
pub static VERSION_MINOR : u8 = 0;
pub static VERSION_FIXES : u8 = 0;

pub static VERSION_SUFIX : Option<&'static str> = Some("dev");

pub enum Network
{
    MainNet = 0xD9B4BEF9
}

pub static NETWORK : u32 = MainNet as u32; /* This should be of type Network */

pub static PROTOCOL_VERSION : u32 = 70002;

/* We reject peers with protocol versions smaller than this */
pub static PROTOCOL_VERSION_MIN : u32 = 70002;

pub enum Service
{
    NoService   = 0,
    NodeNetwork = 1 << 0,
}

pub type Services = u64;

pub static SERVICES : Services = NodeNetwork as Services;

pub fn version() -> String
{
    match VERSION_SUFIX
    {
        Some(ref suf) => format!("{}.{}.{}-{}",VERSION_MAJOR,VERSION_MINOR,
                                           VERSION_FIXES,suf),
        None      => format!("{}.{}.{}",VERSION_MAJOR,VERSION_MINOR,
                                        VERSION_FIXES)
    }
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
