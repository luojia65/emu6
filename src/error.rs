use thiserror::Error;
use crate::mem64::MemError as Mem64Error;
use crate::riscv::FetchError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("error in memory module")]
    Mem64(#[from] Mem64Error),
    #[error("error in instruction fetch")]
    Fetch(#[from] FetchError),
}

pub type Result<T> = core::result::Result<T, Error>;
