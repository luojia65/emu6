use core::ops::Range;
use crate::size::Usize;
use crate::plugin::MemoryExtVTable;

pub struct MemorySet<'a> {
    sections: Vec<Section<'a>>,
}

#[derive(Debug)]
struct Section<'a> {
    config: Config,
    inner: SectionInner<'a>,
}

#[derive(Debug)]
enum SectionInner<'a> {
    Borrowed(&'a [u8]),
    BorrowedMut(&'a mut [u8]),
    Owned(Vec<u8>),
    Extension(),
}

struct Extension {
    vtable: MemoryExtVTable,
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

