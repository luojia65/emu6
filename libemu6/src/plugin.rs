use std::collections::HashMap;
use std::sync::Arc;
use core::ptr::NonNull;

#[derive(Clone)]
pub struct Plugin {
    features: Arc<HashMap<&'static str, NonNull<()>>>,
}

pub struct Builder {
    features: HashMap<&'static str, NonNull<()>>
}

impl Builder {
    pub fn new() -> Builder {
        Builder { features: HashMap::new() }
    }

    pub fn insert(mut self, feature: &'static str, content: NonNull<()>) -> Self {
        self.features.insert(feature, content);
        self
    }

    pub fn build(self) -> Plugin {
        Plugin { features: Arc::new(self.features) }
    }
}

#[non_exhaustive]
#[repr(u8)]
pub enum MemResult {
    Ok = 0,
    NoMemory = 1,
    CannotWrite = 2,
    CannotExecute = 3,
}

#[repr(u8)]
pub enum Endian {
    Big = 0,
    Little = 1,
}

pub const MEMORY_EXT: &'static str = "memory-ext";

#[repr(C)]
pub struct MemoryExtVTable {
    pub api_version: u32,
    pub memory_new: extern "C" fn() -> *mut (),
    pub memory_unref: extern "C" fn(this: *mut ()),
    pub read_nbytes: extern "C" fn(
        this: *mut (), addr: *const (), 
        val_out: *mut (), endian: Endian, nbytes: u8
    ) -> MemResult,
    pub write_nbytes: extern "C" fn(
        this: *mut (), addr: *const (), 
        val_in: *const (), endian: Endian, nbytes: u8
    ) -> MemResult,
}
