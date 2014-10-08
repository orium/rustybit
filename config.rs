pub static NAME : &'static str = "rustybit";

pub static VERSION_MAJOR : u8 = 0;
pub static VERSION_MINOR : u8 = 0;
pub static VERSION_DEV   : u8 = 0;

pub static PROTOCOL_VERSION : u32 = 70002;

enum Services {
    NodeNetwork = 1 << 0,
}

pub static SERVICES: Services = NodeNetwork;

pub fn version() -> String
{
    format!("{}.{}-{}",VERSION_MAJOR,VERSION_MINOR,VERSION_DEV)
}

pub fn name_version() -> String
{
    format!("{} {}",NAME,version())
}
