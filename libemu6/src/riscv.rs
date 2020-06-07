mod exec;
mod fetch;
mod imm;
mod regfile;

pub use exec::{ExecError, Execute};
pub use fetch::{Fetch, FetchError, Instruction};

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Xlen {
    X32,
    X64,
    X128,
}

use crate::size::Usize;
use core::result::Result;

#[non_exhaustive]
pub enum Error {
    NoMemory,
    CannotWrite,
    CannotExecute,
}

pub trait Exec {
    fn fetch_u16(&mut self, idx: Usize) -> Result<u16, Error>;
}

pub trait Read {
    fn read_u8(&self, addr: Usize) -> Result<u8, Error>;
    fn read_u16(&self, addr: Usize) -> Result<u16, Error>;
    fn read_u32(&self, addr: Usize) -> Result<u32, Error>;
    fn read_u64(&self, addr: Usize) -> Result<u64, Error>;
    fn read_u128(&self, addr: Usize) -> Result<u128, Error>;
}

pub trait Write {
    fn write_u8(&self, addr: Usize, val: u8) -> Result<(), Error>;
    fn write_u16(&self, addr: Usize, val: u16) -> Result<(), Error>;
    fn write_u32(&self, addr: Usize, val: u32) -> Result<(), Error>;
    fn write_u64(&self, addr: Usize, val: u64) -> Result<(), Error>;
    fn write_u128(&self, addr: Usize, val: u128) -> Result<(), Error>;
}
