use core::ops::Range;
use core::mem::MaybeUninit;
use crate::size::Usize;
use crate::plugin::MemoryExtVTable;

pub struct MemorySet<'a> {
    sections: Vec<Section<'a>>,
}

impl<'a> MemorySet<'a> {
    pub fn new() -> MemorySet<'a> {
        MemorySet {
            sections: Vec::new(),
        }
    }

    pub fn push_zeroed(&mut self, config: Config) {
        if !self.check_overlap(&config.range) {
            panic!("Section region overlapped")
        }
        self.sections.push(Section::new_zeroed(config));
    }

    pub fn push_slice(&mut self, config: Config, slice: &'a [u8]) {
        if !self.check_overlap(&config.range) {
            panic!("Section region overlapped")
        }
        self.sections.push(Section::new_slice(config, slice));
    }

    pub fn push_slice_mut(&mut self, config: Config, slice: &'a mut [u8]) {
        if !self.check_overlap(&config.range) {
            panic!("Section region overlapped")
        }
        self.sections.push(Section::new_slice_mut(config, slice));
    }

    pub fn push_owned(&mut self, config: Config, owned: Vec<u8>) {
        if !self.check_overlap(&config.range) {
            panic!("Section region overlapped")
        }
        self.sections.push(Section::new_owned(config, owned));
    }

    pub fn push_extension(&mut self, vtable: Box<MemoryExtVTable>) {
        // todo: check overlap
        self.sections.push(Section::new_extension(vtable));
    }

    fn check_overlap(&self, new_range: &Range<Usize>) -> bool {
        let start = new_range.start;
        let end = new_range.end;
        for section in &self.sections {
            let existing_range = match section {
                Section::Buffer(config, _) => config.range.clone(),
                Section::Extension(extension) => extension.get_range(),
            };
            if existing_range.contains(&start) || existing_range.contains(&end) {
                return false;
            }
        }
        true
    }
}

struct Extension {
    vtable: Box<MemoryExtVTable>,
    instance: *mut (),
}

impl Extension {
    fn get_range(&self) -> Range<Usize> {
        let mut addr_len_bytes: MaybeUninit<u32> = MaybeUninit::uninit();
        let mut buf_from: MaybeUninit<[u8; 8]> = MaybeUninit::uninit();
        let mut buf_to: MaybeUninit<[u8; 8]> = MaybeUninit::uninit();
        (self.vtable.get_range)(
            self.instance, addr_len_bytes.as_mut_ptr(), 
            buf_from.as_mut_ptr() as _, buf_to.as_mut_ptr() as _
        );
        let addr_len_bytes: u32 = unsafe { addr_len_bytes.assume_init() };
        let addr_from = match addr_len_bytes {
            4 => (Usize::U32(u32::from_ne_bytes(unsafe { *(buf_from.as_ptr() as *const [u8; 4]) }))),
            8 => (Usize::U64(u64::from_ne_bytes(unsafe { buf_from.assume_init() }))),
            _ => panic!("invalid addr_len_bytes")
        };
        let addr_to = match addr_len_bytes {
            4 => (Usize::U32(u32::from_ne_bytes(unsafe { *(buf_from.as_ptr() as *const [u8; 4]) }))),
            8 => (Usize::U64(u64::from_ne_bytes(unsafe { buf_from.assume_init() }))),
            _ => panic!("invalid addr_len_bytes")
        };
        addr_from..addr_to
    }
}

impl Drop for Extension {
    fn drop(&mut self) {
        (self.vtable.memory_unref)(self.instance)
    }
}

// #[derive(Debug)]
enum Section<'a> {
    Buffer(Config, SectionInner<'a>),
    Extension(Extension),
}

impl<'a> Section<'a> {
    fn new_zeroed(config: Config) -> Section<'a> {
        Section::Buffer(config, SectionInner::Owned(Vec::new()))
    }

    fn new_slice(config: Config, slice: &[u8]) -> Section {
        if config.protect.contains(Protect::WRITE) {
            panic!("Cannot construct writeable buffer from read-only slices")
        }
        Section::Buffer(config, SectionInner::Borrowed(slice))
    }

    fn new_slice_mut(config: Config, slice: &mut [u8]) -> Section {
        Section::Buffer(config, SectionInner::BorrowedMut(slice))
    }

    fn new_owned(config: Config, owned: Vec<u8>) -> Section<'a> {
        Section::Buffer(config, SectionInner::Owned(owned))
    }

    fn new_extension(vtable: Box<MemoryExtVTable>) -> Section<'a> {
        let instance = (vtable.memory_new)();
        Section::Extension(Extension { vtable, instance })
    }
}

// #[derive(Debug)]
enum SectionInner<'a> {
    Borrowed(&'a [u8]),
    BorrowedMut(&'a mut [u8]),
    Owned(Vec<u8>),
    Extension(Extension),
}

#[derive(Clone, Debug)]
pub struct Config {
    pub range: Range<Usize>,
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
        const WRITE = 0b1;
        const EXECUTE = 0b10;
    }
}
