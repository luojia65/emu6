use crate::mmu64::Physical;
use crate::mmu64::Result;

pub struct IntRegister {
    registers: [u64; 32],
}

pub struct Csr {
    inner: [u64; 4096],
}

pub struct Fetch<T> {
    inner: T
}

impl<'a, T: AsRef<Physical<'a>>> Fetch<T> {
    pub fn next_instruction(&mut self, cur: u64) -> Result<Instruction> {
        let a = self.inner.as_ref().fetch_ins_u16(cur)?;
        println!("!!{}", a);
        todo!()
    }
}

pub enum Instruction {

}
