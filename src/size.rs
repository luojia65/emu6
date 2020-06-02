#[derive(Debug, Clone, Copy, Ord, Eq, PartialEq)]
pub enum Usize {
    U32(u32),
    U64(u64),
}

impl core::cmp::PartialOrd for Usize {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use Usize::*;
        match (self, other) {
            (U32(a), U32(b)) => a.partial_cmp(b),
            (U64(a), U64(b)) => a.partial_cmp(b),
            _ => None
        } 
    }
}

impl core::cmp::PartialEq<u32> for Usize {
    fn eq(&self, other: &u32) -> bool {
        use Usize::*;
        match self {
            U32(a) => *a == *other,
            U64(a) => *a == *other as u64,
        } 
    }
}

impl core::cmp::PartialOrd<u32> for Usize {
    fn partial_cmp(&self, other: &u32) -> Option<std::cmp::Ordering> {
        use Usize::*;
        match self {
            U32(a) => a.partial_cmp(other),
            U64(a) => a.partial_cmp(&(*other as u64)),
        } 
    }
}

#[derive(Debug, Clone, Copy, Ord, Eq, PartialEq)]
pub enum Isize {
    U32(i32),
    U64(i64),
}

impl core::cmp::PartialOrd for Isize {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use Isize::*;
        match (self, other) {
            (U32(a), U32(b)) => a.partial_cmp(b),
            (U64(a), U64(b)) => a.partial_cmp(b),
            _ => None
        } 
    }
}

impl core::cmp::PartialEq<i32> for Isize {
    fn eq(&self, other: &i32) -> bool {
        use Isize::*;
        match self {
            U32(a) => *a == *other,
            U64(a) => *a == *other as i64,
        } 
    }
}

impl core::cmp::PartialOrd<i32> for Isize {
    fn partial_cmp(&self, other: &i32) -> Option<std::cmp::Ordering> {
        use Isize::*;
        match self {
            U32(a) => a.partial_cmp(other),
            U64(a) => a.partial_cmp(&(*other as i64)),
        } 
    }
}

impl Usize {
    fn set_zext_u8(&mut self, a: u8) {
        match self {
            Usize::U32(data) => *data = a as u32,
            Usize::U64(data) => *data = a as u64,
        }
    }

    fn set_zext_u16(&mut self, a: u16) {
        match self {
            Usize::U32(data) => *data = a as u32,
            Usize::U64(data) => *data = a as u64,
        }
    }

    fn set_zext_u32(&mut self, a: u32) {
        match self {
            Usize::U32(data) => *data = a,
            Usize::U64(data) => *data = a as u64,
        }
    }

    fn set_sext_u8(&mut self, a: u8) {
        match self {
            Usize::U32(data) => 
                *data = (a as u32) | if (a >> 7) != 0 { 0xFFFFFF00 } else { 0 },
            Usize::U64(data) => 
                *data = (a as u64) | if (a >> 7) != 0 { 0xFFFFFFFFFFFFFF00 } else { 0 },
        }
    }

    fn set_sext_u16(&mut self, a: u16) {
        match self {
            Usize::U32(data) => 
                *data = (a as u32) | if (a >> 7) != 0 { 0xFFFF0000 } else { 0 },
            Usize::U64(data) => 
                *data = (a as u64) | if (a >> 7) != 0 { 0xFFFFFFFFFFFF0000 } else { 0 },
        }
    }

    fn set_sext_u32(&mut self, a: u32) {
        match self {
            Usize::U32(data) => *data = a,
            Usize::U64(data) => 
                *data = (a as u64) | if (a >> 7) != 0 { 0xFFFFFFFF00000000 } else { 0 },
        }
    }

    fn set_sext_u64(&mut self, a: u64) {
        match self {
            Usize::U32(_) => panic!("Write u64 on XLEN < u64"),
            Usize::U64(data) => *data = a,
        }
    }

    fn lowbit_u8(self) -> u8 {
        match self {
            Usize::U32(data) => (data & 0xFF) as u8,
            Usize::U64(data) => (data & 0xFF) as u8,
        }
    }

    fn lowbit_u16(self) -> u16 {
        match self {
            Usize::U32(data) => (data & 0xFFFF) as u16,
            Usize::U64(data) => (data & 0xFFFF) as u16,
        }
    }

    fn lowbit_u32(self) -> u32 {
        match self {
            Usize::U32(data) => data,
            Usize::U64(data) => (data & 0xFFFFFFFF) as u32,
        }
    }

    fn to_mem_addr(self) -> u64 {
        match self {
            Usize::U32(data) => data as u64,
            Usize::U64(data) => data,
        }
    }

    fn as_signed(self) -> Isize {
        match self {
            Usize::U32(data) => Isize::U32(i32::from_ne_bytes(u32::to_ne_bytes(data))),
            Usize::U64(data) => Isize::U64(i64::from_ne_bytes(u64::to_ne_bytes(data))),
        }
    }
}

impl core::ops::Add<u32> for Usize {
    type Output = Usize;
    fn add(self, rhs: u32) -> Self::Output {
        match self {
            Usize::U32(u32_val) => 
                Usize::U32(u32_val.wrapping_add(rhs)),
            Usize::U64(u64_val) => 
                Usize::U64(u64_val.wrapping_add(rhs as u64)),
        }
    }
}

impl core::ops::Add<i32> for Usize {
    type Output = Usize;
    fn add(self, rhs: i32) -> Self::Output {
        match self {
            Usize::U32(u32_val) => 
                Usize::U32(if rhs > 0 {
                    u32_val.wrapping_add(rhs as u32)
                } else { 
                    u32_val.wrapping_sub(rhs.abs() as u32)
                }),
            Usize::U64(u64_val) => 
                Usize::U64(if rhs > 0 {
                    u64_val.wrapping_add(rhs as u64)
                } else { 
                    u64_val.wrapping_sub(rhs.abs() as u64)
                }),
        }
    }
}

impl core::ops::AddAssign<i32> for Usize {
    fn add_assign(&mut self, rhs: i32) {
        match self {
            Usize::U32(u32_val) => 
                *u32_val = if rhs > 0 {
                    u32_val.wrapping_add(rhs as u32)
                } else { 
                    u32_val.wrapping_sub(rhs.abs() as u32)
                },
            Usize::U64(u64_val) => 
                *u64_val = if rhs > 0 {
                    u64_val.wrapping_add(rhs as u64)
                } else { 
                    u64_val.wrapping_sub(rhs.abs() as u64)
                },
        }
    }
}
