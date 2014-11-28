use std::fmt::Show;
use std::fmt::Formatter;

/* This makes it safe to change to subunits of satoshi in the future without
 * creating nasty bugs, because type system.
 */
#[allow(dead_code)]
pub enum Value
{
    Satoshi(u64)
}

impl Show for Value
{
    fn fmt(&self, f : &mut Formatter) -> Result<(), ::std::fmt::Error>
    {
        match *self
        {
            Value::Satoshi(v) => write!(f,"{}.{:05} m฿",v/100_000,v%100_000)
        }
    }
}
