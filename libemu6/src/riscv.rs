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
    
// #[repr(C)]
// pub struct VectorMemExtVTable {
//     /* for i in 0..nelems {
//            if mask[i * elem_width_bytes] == 1 {
//                *(val_out + i * elem_width_bytes) = 
//                     MEM[base_addr + i * elem_width_bytes]
//            }
//        } */
//     pub read_unit_stride: extern "C" fn(
//         this: *mut (), addr_len_bytes: u32, base_addr: *const u8, 
//         elem_width_bytes: u32, mask: *const u8, 
//         val_out: *mut u8, nelems: u32, endian: Endian, 
//     ) -> MemResult,
//     pub write_unit_stride: extern "C" fn(
//         this: *mut (), addr_len_bytes: u32, base_addr: *mut u8, 
//         elem_width_bytes: u32, mask: *const u8, 
//         val_in: *const u8, nelems: u32, endian: Endian, 
//     ) -> MemResult,
//     /*  for i in 0..nelems {
//             if mask[i * elem_width_bytes] == 1 {
//                 *(val_out + i * elem_width_bytes) = 
//                     MEM[base_addr + i * stride_bytes]
//             }
//         } */
//     pub read_strided: extern "C" fn(
//         this: *mut (), addr_len_bytes: u32, 
//         base_addr: *const u8, stride_bytes: *const u8, 
//         elem_width_bytes: u32, mask: *const u8, 
//         val_out: *mut u8, nelems: u32, endian: Endian, 
//     ) -> MemResult,
//     pub write_strided: extern "C" fn(
//         this: *mut (), addr_len_bytes: u32, 
//         base_addr: *mut u8, stride_bytes: *const u8, 
//         elem_width_bytes: u32, mask: *const u8, 
//         val_in: *const u8, nelems: u32, endian: Endian, 
//     ) -> MemResult,
//     /*  for i in 0..nelems {
//             if mask[i * elem_width_bytes] == 1 {
//                 *(val_out + i * elem_width_bytes) = 
//                     MEM[base_addr + index_array[i * addr_len_bytes]]
//             }
//         } */
//     pub read_indexed: extern "C" fn(
//         this: *mut (), addr_len_bytes: u32, 
//         base_addr: *const u8, index_array: *const u8,
//         elem_width_bytes: u32, mask: *const u8, 
//         val_out: *mut u8, nelems: u32, endian: Endian, 
//     ) -> MemResult,
//     pub write_indexed_unordered: extern "C" fn(
//         this: *mut (), addr_len_bytes: u32, 
//         base_addr: *mut u8, index_array: *const u8,
//         elem_width_bytes: u32, mask: *const u8, 
//         val_in: *const u8, nelems: u32, endian: Endian, 
//     ) -> MemResult,
//     pub write_indexed_ordered: extern "C" fn(
//         this: *mut (), addr_len_bytes: u32, 
//         base_addr: *mut u8, index_array: *const u8,
//         elem_width_bytes: u32, mask: *const u8, 
//         val_in: *const u8, nelems: u32, endian: Endian, 
//     ) -> MemResult,
// }
