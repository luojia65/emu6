pub mod fetch;
pub mod exec;
pub mod regfile;

use crate::size::Usize;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Xlen {
    X32,
    X64,
}

