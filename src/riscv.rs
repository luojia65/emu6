mod fetch;
mod exec;
mod regfile;
mod imm;

pub use exec::Execute;
pub use fetch::{Fetch, Instruction, FetchError};

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Xlen {
    X32,
    X64,
    X128,
}
