use std::fmt::Show;
use std::fmt::Formatter;

/* TODO */
pub struct Script;

impl Script
{
    pub fn new() -> Script
    {
        Script
    }
}

impl Show for Script
{
    fn fmt(&self, _f : &mut Formatter) -> Result<(), ::std::fmt::Error>
    {
        Ok(()) // XXX TODO
    }
}
