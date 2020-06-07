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
