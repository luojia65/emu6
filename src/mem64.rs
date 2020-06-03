use crate::error::Result;
use core::ops::Range;
use core::ptr::copy_nonoverlapping;
use thiserror::Error;

#[derive(Debug)]
pub struct Physical<'a> {
    sections: Vec<Section<'a>>,
}

impl<'a> Physical<'a> {
    pub fn new() -> Physical<'a> {
        Physical {
            sections: Vec::new(),
        }
    }

    pub fn push_zeroed(&mut self, config: Config) {
        if !self.check_overlap(&config) {
            panic!("Section region overlapped")
        }
        self.sections.push(Section::new_zeroed(config));
    }

    pub fn push_slice(&mut self, config: Config, slice: &'a [u8]) {
        if !self.check_overlap(&config) {
            panic!("Section region overlapped")
        }
        self.sections.push(Section::new_slice(config, slice));
    }

    pub fn push_slice_mut(&mut self, config: Config, slice: &'a mut [u8]) {
        if !self.check_overlap(&config) {
            panic!("Section region overlapped")
        }
        self.sections.push(Section::new_slice_mut(config, slice));
    }

    pub fn push_owned(&mut self, config: Config, owned: Vec<u8>) {
        if !self.check_overlap(&config) {
            panic!("Section region overlapped")
        }
        self.sections.push(Section::new_owned(config, owned));
    }

    fn check_overlap(&self, new_config: &Config) -> bool {
        let start = new_config.range.start;
        let end = new_config.range.end;
        for section in &self.sections {
            if section.config.range.contains(&start) || section.config.range.contains(&end) {
                return false;
            }
        }
        true
    }
}

impl<'a> Physical<'a> {
    pub fn read_u8(&self, addr: u64) -> Result<u8> {
        self.read_any(
            addr,
            |section, addr| section.read_u8(addr),
            Protect::READ,
            |addr| MemError::CannotRead { addr },
        )
    }

    pub fn read_u16(&self, addr: u64) -> Result<u16> {
        self.read_any(
            addr,
            |section, addr| section.read_u16(addr),
            Protect::READ,
            |addr| MemError::CannotRead { addr },
        )
    }

    pub fn read_u32(&self, addr: u64) -> Result<u32> {
        self.read_any(
            addr,
            |section, addr| section.read_u32(addr),
            Protect::READ,
            |addr| MemError::CannotRead { addr },
        )
    }

    pub fn read_u64(&self, addr: u64) -> Result<u64> {
        self.read_any(
            addr,
            |section, addr| section.read_u64(addr),
            Protect::READ,
            |addr| MemError::CannotRead { addr },
        )
    }

    pub fn fetch_ins_u16(&self, addr: u64) -> Result<u16> {
        self.read_any(
            addr,
            |section, addr| section.read_u16(addr),
            Protect::EXECUTE,
            |addr| MemError::CannotExecute { addr },
        )
    }

    fn read_any<T, F, E>(&self, addr: u64, f: F, token: Protect, e: E) -> Result<T>
    where
        F: Fn(&Section, u64) -> Result<T>,
        E: Fn(u64) -> MemError,
    {
        if let Some(section) = self.choose_section(addr) {
            if section.config.protect.contains(token) {
                f(section, addr)
            } else {
                Err(e(addr))?
            }
        } else {
            Err(MemError::NoMemory { addr })?
        }
    }

    pub fn write_u8(&mut self, addr: u64, n: u8) -> Result<()> {
        self.write_any(addr, |section, addr| section.write_u8(addr, n))
    }

    pub fn write_u16(&mut self, addr: u64, n: u16) -> Result<()> {
        self.write_any(addr, |section, addr| section.write_u16(addr, n))
    }

    pub fn write_u32(&mut self, addr: u64, n: u32) -> Result<()> {
        self.write_any(addr, |section, addr| section.write_u32(addr, n))
    }

    pub fn write_u64(&mut self, addr: u64, n: u64) -> Result<()> {
        self.write_any(addr, |section, addr| section.write_u64(addr, n))
    }

    fn write_any<F>(&mut self, addr: u64, f: F) -> Result<()>
    where
        F: Fn(&mut Section, u64) -> Result<()>,
    {
        for mut section in &mut self.sections {
            if section.config.range.contains(&addr) {
                if section.config.protect.contains(Protect::WRITE) {
                    return f(&mut section, addr);
                } else {
                    return Err(MemError::CannotWrite { addr })?;
                }
            }
        }
        Err(MemError::NoMemory { addr })?
    }

    fn choose_section(&self, addr: u64) -> Option<&Section> {
        for section in &self.sections {
            if section.config.range.contains(&addr) {
                return Some(&section);
            }
        }
        None
    }
}

#[derive(Debug)]
struct Section<'a> {
    config: Config,
    inner: SectionInner<'a>,
}

impl<'a> Section<'a> {
    fn new_zeroed(config: Config) -> Section<'a> {
        Section {
            config,
            inner: SectionInner::Owned(Vec::new()),
        }
    }

    fn new_slice(config: Config, slice: &[u8]) -> Section {
        if config.protect.contains(Protect::WRITE) {
            panic!("Cannot construct writeable buffer from read-only slices")
        }
        Section {
            config,
            inner: SectionInner::Borrowed(slice),
        }
    }

    fn new_slice_mut(config: Config, slice: &mut [u8]) -> Section {
        Section {
            config,
            inner: SectionInner::BorrowedMut(slice),
        }
    }

    fn new_owned(config: Config, owned: Vec<u8>) -> Section<'a> {
        Section {
            config,
            inner: SectionInner::Owned(owned),
        }
    }
}

impl<'a> Section<'a> {
    pub fn read_u8(&self, addr: u64) -> Result<u8> {
        self.check_read(addr)?;
        let offset = self.get_underlying_buf_offset(addr)?;
        Ok(self.inner.read_u8(offset as usize))
    }

    pub fn read_u16(&self, addr: u64) -> Result<u16> {
        self.check_read(addr)?;
        let offset = self.get_underlying_buf_offset(addr)?;
        Ok(self.inner.read_u16(offset as usize, self.config.endian))
    }

    pub fn read_u32(&self, addr: u64) -> Result<u32> {
        self.check_read(addr)?;
        let offset = self.get_underlying_buf_offset(addr)?;
        Ok(self.inner.read_u32(offset as usize, self.config.endian))
    }

    pub fn read_u64(&self, addr: u64) -> Result<u64> {
        self.check_read(addr)?;
        let offset = self.get_underlying_buf_offset(addr)?;
        Ok(self.inner.read_u64(offset as usize, self.config.endian))
    }

    pub fn write_u8(&mut self, addr: u64, n: u8) -> Result<()> {
        self.check_write(addr)?;
        let offset = self.get_underlying_buf_offset(addr)?;
        Ok(self.inner.write_u8(offset as usize, n))
    }

    pub fn write_u16(&mut self, addr: u64, n: u16) -> Result<()> {
        self.check_write(addr)?;
        let offset = self.get_underlying_buf_offset(addr)?;
        Ok(self.inner.write_u16(offset as usize, n, self.config.endian))
    }

    pub fn write_u32(&mut self, addr: u64, n: u32) -> Result<()> {
        self.check_write(addr)?;
        let offset = self.get_underlying_buf_offset(addr)?;
        Ok(self.inner.write_u32(offset as usize, n, self.config.endian))
    }

    pub fn write_u64(&mut self, addr: u64, n: u64) -> Result<()> {
        self.check_write(addr)?;
        let offset = self.get_underlying_buf_offset(addr)?;
        Ok(self.inner.write_u64(offset as usize, n, self.config.endian))
    }

    fn get_underlying_buf_offset(&self, addr: u64) -> Result<u64> {
        if !self.config.range.contains(&addr) {
            return Err(MemError::NoMemory { addr })?;
        }
        Ok(addr - self.config.range.start) // will not underflow
    }

    fn check_read(&self, addr: u64) -> Result<()> {
        if !self.config.protect.contains(Protect::READ) {
            return Err(MemError::CannotRead { addr })?;
        }
        Ok(())
    }

    fn check_write(&self, addr: u64) -> Result<()> {
        if !self.config.protect.contains(Protect::WRITE) {
            return Err(MemError::CannotWrite { addr })?;
        }
        Ok(())
    }
}

#[derive(Debug)]
enum SectionInner<'a> {
    Borrowed(&'a [u8]),
    BorrowedMut(&'a mut [u8]),
    Owned(Vec<u8>),
}

impl<'a> SectionInner<'a> {
    fn read_u8(&self, offset: usize) -> u8 {
        match self {
            SectionInner::Borrowed(slice) => slice[offset],
            SectionInner::BorrowedMut(slice) => slice[offset],
            SectionInner::Owned(vec) => vec[offset],
        }
    }

    fn read_u16(&self, offset: usize, endian: Endian) -> u16 {
        self.read_uint(offset, endian, 2) as u16
    }

    fn read_u32(&self, offset: usize, endian: Endian) -> u32 {
        self.read_uint(offset, endian, 4) as u32
    }

    fn read_u64(&self, offset: usize, endian: Endian) -> u64 {
        self.read_uint(offset, endian, 8)
    }

    fn read_uint(&self, offset: usize, endian: Endian, nbytes: usize) -> u64 {
        let buf_ptr = match self {
            SectionInner::Borrowed(slice) => slice.as_ptr(),
            SectionInner::BorrowedMut(slice) => slice.as_ptr(),
            SectionInner::Owned(vec) => vec.as_ptr(),
        };
        let buf_ptr = unsafe { buf_ptr.offset(offset as isize) };
        let mut out = 0u64;
        let ptr_out = &mut out as *mut u64 as *mut u8;
        unsafe {
            copy_nonoverlapping(buf_ptr, ptr_out, nbytes);
        }
        match endian {
            Endian::Big => out.to_be(),
            Endian::Little => out.to_le(),
        }
    }

    fn write_u8(&mut self, offset: usize, n: u8) {
        match self {
            SectionInner::Borrowed(_slice) => unreachable!(),
            SectionInner::BorrowedMut(slice) => slice[offset] = n,
            SectionInner::Owned(vec) => vec[offset] = n,
        }
    }

    fn write_u16(&mut self, offset: usize, n: u16, endian: Endian) {
        self.write_uint(offset, n as u64, endian, 2)
    }

    fn write_u32(&mut self, offset: usize, n: u32, endian: Endian) {
        self.write_uint(offset, n as u64, endian, 4)
    }

    fn write_u64(&mut self, offset: usize, n: u64, endian: Endian) {
        self.write_uint(offset, n, endian, 8)
    }

    fn write_uint(&mut self, offset: usize, n: u64, endian: Endian, nbytes: usize) {
        let buf_ptr = match self {
            SectionInner::Borrowed(slice) => slice.as_ptr(),
            SectionInner::BorrowedMut(slice) => slice.as_ptr(),
            SectionInner::Owned(vec) => vec.as_ptr(),
        };
        let buf_ptr = unsafe { buf_ptr.offset(offset as isize) as *mut u8 };
        let in_buf = match endian {
            Endian::Big => n.to_be(),
            Endian::Little => n.to_le(),
        };
        let in_ptr = &in_buf as *const u64 as *const u8;
        unsafe {
            copy_nonoverlapping(in_ptr, buf_ptr, nbytes);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    pub range: Range<u64>,
    pub protect: Protect,
    pub endian: Endian,
}

#[derive(Clone, Copy, Debug)]
pub enum Endian {
    Big,
    Little,
}

bitflags::bitflags! {
    pub struct Protect: u8 {
        const READ = 0b1;
        const WRITE = 0b10;
        const EXECUTE = 0b100;
    }
}

#[derive(Error, Clone, Debug)]
pub enum MemError {
    #[error("Memory address 0x{addr:016X} cannot be read")]
    CannotRead { addr: u64 },
    #[error("Memory address 0x{addr:016X} cannot be written")]
    CannotWrite { addr: u64 },
    #[error("Memory address 0x{addr:016X} cannot be executed")]
    CannotExecute { addr: u64 },
    #[error("No memory bound for address 0x{addr:016X}")]
    NoMemory { addr: u64 },
}
