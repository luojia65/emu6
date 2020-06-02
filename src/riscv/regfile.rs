use crate::size::{Usize, Isize};
use super::Xlen;

pub struct XReg {
    x: [Usize; 32],
}

impl XReg {
    pub fn new_zeroed(xlen: Xlen) -> XReg {
        let init = match xlen {
            Xlen::X32 => Usize::U32(0),
            Xlen::X64 => Usize::U64(0),
        };
        XReg { x: [init; 32] }
    }
}

impl XReg {
    pub fn r_usize(&self, idx: u8) -> Usize {
        self.x[idx as usize]
    }
    pub fn r_isize(&self, idx: u8) -> Isize {
        match self.r_usize(idx) {
            Usize::U32(data) => Isize::I32(i32::from_ne_bytes(u32::to_ne_bytes(data))),
            Usize::U64(data) => Isize::I64(i64::from_ne_bytes(u64::to_ne_bytes(data))),
        }
    }
    pub fn r_low8(&self, idx: u8) -> u8 {
        match self.x[idx as usize] {
            Usize::U32(data) => (data & 0xFF) as u8,
            Usize::U64(data) => (data & 0xFF) as u8,
        }
    }
    pub fn r_low16(&self, idx: u8) -> u16 {
        match self.x[idx as usize] {
            Usize::U32(data) => (data & 0xFFFF) as u16,
            Usize::U64(data) => (data & 0xFFFF) as u16,
        }
    }
    pub fn r_low32(&self, idx: u8) -> u32 {
        match self.x[idx as usize] {
            Usize::U32(data) => (data & 0xFFFFFFFF) as u32,
            Usize::U64(data) => (data & 0xFFFFFFFF) as u32,
        }
    }
    pub fn r_low64(&self, idx: u8) -> u64 {
        match self.x[idx as usize] {
            Usize::U32(_) => panic!("cannot read 64-bit value from 32-bit register"),
            Usize::U64(data) => data,
        }
    }
    pub fn w_usize(&mut self, idx: u8, val: Usize) {
        if idx == 0 { return }
        self.x[idx as usize] = val;
    }
    pub fn w_isize(&mut self, idx: u8, val: Isize) {
        if idx == 0 { return }
        self.x[idx as usize] = val.cast_to_usize();
    }
    pub fn w_zext8(&mut self, idx: u8, val: u8) {
        if idx == 0 { return }
        match &mut self.x[idx as usize] {
            Usize::U32(data) => *data = val as u32,
            Usize::U64(data) => *data = val as u64,
        }
    }
    pub fn w_zext16(&mut self, idx: u8, val: u16) {
        if idx == 0 { return }
        match &mut self.x[idx as usize] {
            Usize::U32(data) => *data = val as u32,
            Usize::U64(data) => *data = val as u64,
        }
    }
    pub fn w_zext32(&mut self, idx: u8, val: u32) {
        if idx == 0 { return }
        match &mut self.x[idx as usize] {
            Usize::U32(data) => *data = val,
            Usize::U64(data) => *data = val as u64,
        }
    }
    // useful for xlen==X128
    // pub fn w_zext64(&mut self, idx: u8, val: u64) {
    //     if idx == 0 { return }
    //     match &mut self.x[idx as usize] {
    //         Usize::U32(_) => panic!("cannot write 64-bit value into 32-bit register"),
    //         Usize::U64(data) => *data = val,
    //     }
    // }
    pub fn w_sext8(&mut self, idx: u8, val: u8) {
        if idx == 0 { return }
        match &mut self.x[idx as usize] {
            Usize::U32(data) => 
                *data = (val as u32) | if (val >> 7) != 0 { 0xFFFFFF00 } else { 0 },
            Usize::U64(data) => 
                *data = (val as u64) | if (val >> 7) != 0 { 0xFFFFFFFFFFFFFF00 } else { 0 },
        }
    }
    pub fn w_sext16(&mut self, idx: u8, val: u16) {
        if idx == 0 { return }
        match &mut self.x[idx as usize] {
            Usize::U32(data) => 
                *data = (val as u32) | if (val >> 15) != 0 { 0xFFFF0000 } else { 0 },
            Usize::U64(data) => 
                *data = (val as u64) | if (val >> 15) != 0 { 0xFFFFFFFFFFFF0000 } else { 0 },
        }
    }
    pub fn w_sext32(&mut self, idx: u8, val: u32) {
        if idx == 0 { return }
        match &mut self.x[idx as usize] {
            Usize::U32(data) => *data = val,
            Usize::U64(data) => 
                *data = (val as u64) | if (val >> 31) != 0 { 0xFFFFFFFF00000000 } else { 0 },
        }
    }
    pub fn w_sext64(&mut self, idx: u8, val: u64) {
        if idx == 0 { return }
        match &mut self.x[idx as usize] {
            Usize::U32(_) => panic!("cannot write 64-bit value into 32-bit registers"),
            Usize::U64(data) => *data = val,
        }
    }
}

impl core::fmt::Debug for XReg {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.x.iter()).finish()
    }
}

pub struct Csr {
    pub(crate) csr: [u64; 4096],
}
