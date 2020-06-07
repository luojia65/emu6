use crate::mem64::MemError as Mem64Error;
use crate::riscv::{ExecError, FetchError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("error in memory module")]
    Mem64(#[from] Mem64Error),
    #[error("error in instruction fetch")]
    Fetch(#[from] FetchError),
    #[error("error in instruction execution")]
    Exec(#[from] ExecError),
}

pub type Result<T> = core::result::Result<T, Error>;
