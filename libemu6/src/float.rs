// #[derive(Clone, Copy)]
// pub struct F32 {
//     bits: u32
// }

// impl F32 {
//     pub fn from_bits(bits: u32) -> F32 {
//         F32 { bits }
//     }

//     pub fn to_bits(self) -> u32 {
//         self.bits
//     }
// }

// impl F32 {
//     pub fn rounding_add(self, rhs: F32, rounding: Rounding) -> F32 {
//         todo!("soft float")
//     }
    
//     pub fn rounding_sub(self, rhs: F32, rounding: Rounding) -> F32 {
//         todo!("soft float")
//     }

//     pub fn rounding_mul(self, rhs: F32, rounding: Rounding) -> F32 {
//         todo!("soft float")
//     }
    
//     pub fn rounding_div(self, rhs: F32, rounding: Rounding) -> F32 {
//         todo!("soft float")
//     }
    
//     pub fn rounding_sqrt(self, rounding: Rounding) -> F32 {
//         todo!("soft float")
//     }
    
//     // must use IEEE 754-201x minimumNumber and maximumNumber according
//     // to instruction set manual

//     pub fn min(self, rhs: F32) -> F32 {
//         todo!("soft float")
//     }

//     pub fn max(self, rhs: F32) -> F32 {
//         todo!("soft float")
//     }

//     // returns a * b + c
//     pub fn rounding_madd(a: F32, b: F32, c: F32, rounding: Rounding) -> F32 {
//         todo!("soft float")
//     }

//     // returns a * b - c
//     pub fn rounding_msub(a: F32, b: F32, c: F32, rounding: Rounding) -> F32 {
//         todo!("soft float")
//     }

//     // returns -(a * b) + c
//     pub fn rounding_nmsub(a: F32, b: F32, c: F32, rounding: Rounding) -> F32 {
//         todo!("soft float")
//     }

//     // returns -(a * b) - c
//     pub fn rounding_nmadd(a: F32, b: F32, c: F32, rounding: Rounding) -> F32 {
//         todo!("soft float")
//     }
    
//     // todo: convert to integer
// }

// pub enum Rounding {
//     NearestToEven,
//     TowardsZero,
//     Down,
//     Up,
//     NearestToMaxMagitude,
// }
