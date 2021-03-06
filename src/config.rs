pub const NAME : &'static str = "rustybit";

pub const VERSION_MAJOR : u8 = 0;
pub const VERSION_MINOR : u8 = 0;
pub const VERSION_FIXES : u8 = 0;

pub const VERSION_SUFIX : Option<&'static str> = Some("dev");

pub enum Network
{
    MainNet = 0xD9B4BEF9
}

/* TODO This should be of type Network */
pub const NETWORK : u32 = Network::MainNet as u32;

pub const PROTOCOL_VERSION : u32 = 70002;

/* We reject peers with protocol versions smaller than this */
pub const PROTOCOL_VERSION_MIN : u32 = 70002;

pub enum Service
{
    NoService   = 0,
    NodeNetwork = 1
}

pub type Services = u64;

pub const SERVICES : Services = Service::NodeNetwork as Services;

pub const INITIAL_DISCOVERY_PEERS : uint = 30;

pub const DEFAULT_PORT : u16 = 8333;

pub fn version() -> String
{
    match VERSION_SUFIX
    {
        Some(ref suf) => format!("{}.{}.{}-{}",VERSION_MAJOR,VERSION_MINOR,
                                               VERSION_FIXES,suf),
        None          => format!("{}.{}.{}",VERSION_MAJOR,VERSION_MINOR,
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
    format!("/{}:{}/",NAME,version().replace("-","_"))
}
