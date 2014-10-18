use std::fmt::Show;
use std::fmt::Formatter;

/* This makes it safe to change to subunits of satoshi in the future without
 * creating nasty bugs, because type system.
 */
pub enum Value
{
    Satoshi(u64)
}

impl Show for Value
{
    fn fmt(&self, f : &mut Formatter) -> Result<(), ::std::fmt::FormatError>
    {
        match *self
        {
            Satoshi(v) => write!(f,"{}.{:05}m฿",v/100_000,v%100_000)
        }
    }
}
