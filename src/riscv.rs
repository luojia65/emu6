pub mod fetch;
pub mod exec;
pub mod regfile;
pub mod imm;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Xlen {
    X32,
    X64,
}

