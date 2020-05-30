use core::ops::Range;
use core::ptr::copy_nonoverlapping;

#[derive(Debug)]
pub struct Physical<'a> {
    sections: Vec<Section<'a>>,
}

impl<'a> Physical<'a> {
    pub fn new() -> Physical<'a> {
        Physical { sections: Vec::new() }
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
            if section.config.range.contains(&start) || 
                section.config.range.contains(&end) {
                return false;
            }
        }
        true
    } 
}

impl<'a> Physical<'a> {
    pub fn read_u8(&self, addr: u64) -> Result<u8> {
        if let Some(section) = self.choose_section(addr) {
            section.read_u8(addr)
        } else {
            Err(Error::NoMemory { addr })
        }
    }

    pub fn read_u16(&self, addr: u64) -> Result<u16> {
        if let Some(section) = self.choose_section(addr) {
            section.read_u16(addr)
        } else {
            Err(Error::NoMemory { addr })
        }
    }

    pub fn read_u32(&self, addr: u64) -> Result<u32> {
        if let Some(section) = self.choose_section(addr) {
            section.read_u32(addr)
        } else {
            Err(Error::NoMemory { addr })
        }
    }

    pub fn read_u64(&self, addr: u64) -> Result<u64> {
        if let Some(section) = self.choose_section(addr) {
            section.read_u64(addr)
        } else {
            Err(Error::NoMemory { addr })
        }
    }

    fn choose_section(&self, addr: u64) -> Option<&Section> {
        for section in &self.sections {
            if section.config.range.contains(&addr) {
                return Some(&section)
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
        Section { config, inner: SectionInner::Owned(Vec::new()) }
    }

    fn new_slice(config: Config, slice: &[u8]) -> Section {
        if config.protect.contains(Protect::WRITE) {
            panic!("Cannot construct writeable buffer from read-only slices")
        }
        Section { config, inner: SectionInner::Borrowed(slice) }
    }

    fn new_slice_mut(config: Config, slice: &mut [u8]) -> Section {
        Section { config, inner: SectionInner::BorrowedMut(slice) }
    }

    fn new_owned(config: Config, owned: Vec<u8>) -> Section<'a> {
        Section { config, inner: SectionInner::Owned(owned) }
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

    fn get_underlying_buf_offset(&self, addr: u64) -> Result<u64> {
        if !self.config.range.contains(&addr) {
            return Err(Error::NoMemory { addr })
        }
        Ok(addr - self.config.range.start) // will not underflow
    }

    fn check_read(&self, addr: u64) -> Result<()> {
        if !self.config.protect.contains(Protect::READ) {
            return Err(Error::CannotRead { addr })
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
        use SectionInner::*;
        match self {
            Borrowed(slice) => slice[offset],
            BorrowedMut(slice) => slice[offset],
            Owned(vec) => vec[offset],
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
}

#[derive(Clone, Debug)]
pub struct Config {
    pub range: Range<u64>,
    pub align: u64,
    pub protect: Protect,
    pub endian: Endian,
}

#[derive(Clone, Copy, Debug)]
pub enum Endian {
    Big,
    Little
}

bitflags::bitflags! {
    pub struct Protect: u8 {
        const READ = 0b1;
        const WRITE = 0b10;
        const EXECUTE = 0b100;
    }
}

#[derive(Clone, Debug)]
pub enum Error {
    Misaligned { addr: u64, min_align: u64 },
    CannotRead { addr: u64 },
    CannotWrite { addr: u64 },
    CannotExecute { addr: u64 },
    NoMemory { addr: u64 },
}

pub type Result<T> = core::result::Result<T, Error>;
