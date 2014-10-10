pub static NAME : &'static str = "rustybit";

pub static VERSION_MAJOR : u8 = 0;
pub static VERSION_MINOR : u8 = 0;
pub static VERSION_FIXES : u8 = 0;

pub static MAIN_NET : u32 = 0xD9B4BEF9;

pub static PROTOCOL_VERSION : u32 = 70002;

pub enum Services
{
    NodeNetwork = 1 << 0,
}

pub static SERVICES : Services = NodeNetwork;

pub fn version() -> String
{
    format!("{}.{}.{}",VERSION_MAJOR,VERSION_MINOR,VERSION_FIXES)
}

#[allow(dead_code)]
pub fn name_version() -> String
{
    format!("{} {}",NAME,version())
}