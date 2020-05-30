const OPCODE_LOAD: u32 =     0b000_0011; 
const OPCODE_MISC_MEM: u32 = 0b000_1111;
const OPCODE_OP_IMM: u32 =   0b001_0011; 
const OPCODE_AUIPC: u32 =    0b001_0111; 
const OPCODE_STORE: u32 =    0b010_0011; 
const OPCODE_OP: u32 =       0b011_0011; 
const OPCODE_LUI: u32 =      0b011_0111; 
const OPCODE_BRANCH: u32 =   0b110_0011; 
const OPCODE_JALR: u32 =     0b110_0111; 
const OPCODE_JAL: u32 =      0b110_1111;
const OPCODE_SYSTEM: u32 =   0b111_0011; 

use crate::mmu64::Physical;
use crate::mmu64::Result;

pub struct IntRegister {
    registers: [u64; 32],
}

pub struct Csr {
    inner: [u64; 4096],
}

pub struct Fetch<T> {
    pub inner: T,
    pub pc: u64
}

impl<'a, T: AsRef<Physical<'a>>> Fetch<T> {
    pub fn next_instruction(&mut self) -> Result<Instruction> {
        let ins = self.next_u16()?;
        if ins & 0b11 != 0b11 {
            println!("16 bit");
        }
        if ins & 0b11100 != 0b11100 {
            let ins = (ins as u32) + ((self.next_u16()? as u32) << 16);
            println!("32 bit {:08X}", ins);
            return Ok(resolve_u32(ins));
        }
        todo!()
    }

    fn next_u16(&mut self) -> Result<u16> {
        let ans = self.inner.as_ref().fetch_ins_u16(self.pc);
        self.pc += 2;
        ans
    }
}

fn resolve_u32(ins: u32) -> Instruction {
    let opcode = ins & 0b111_1111;
    let imm_i = (ins >> 20) & 0b1111_1111_1111;
    let imm_s = ((ins >> 7) & 0b11111) | ((ins >> 25) & 0b1111111);
    let imm_u = ins & 0b1111_1111_1111_1111_1111_0000_0000_0000;
    let imm_j = {
        (if ins & 0b1000_0000_0000_0000_0000_0000_0000_0000 != 0 {
            0b1111_1111_1111_0000_0000_0000_0000_0000
        } else { 0 }) | 
        (((ins & 0b0111_1111_1110_0000_0000_0000_0000_0000) >> 21) << 1) | 
        (((ins & 0b0000_0000_0001_0000_0000_0000_0000_0000) >> 20) << 11) | 
        (((ins & 0b0000_0000_0000_1111_1111_0000_0000_0000) >> 12) << 12)
    };
    let rd = ((ins >> 7) & 0b1_1111) as u8;
    let rs1 = ((ins >> 15) & 0b1_1111) as u8;
    let rs2 = ((ins >> 20) & 0b1_1111) as u8;
    let funct3 = ((ins >> 12) & 0b111) as u8;
    let funct7 = ((ins >> 25) & 0b111_1111) as u8;
    use Instruction::*;
    match opcode {
        OPCODE_LUI => Lui(UType { rd, imm_u }),
        OPCODE_AUIPC => Auipc(UType { rd, imm_u }),
        OPCODE_JAL => Jal(JType { rd, imm_j }),
        OPCODE_JALR => Jalr(IType { rd, rs1, funct3, imm_i }),
        _ => panic!("EILL")
    }
}

#[derive(Debug)]
pub enum Instruction {
    Lui(UType),
    Auipc(UType),
    Jal(JType),
    Jalr(IType),
}

#[derive(Debug)]
pub struct UType {
    rd: u8,
    imm_u: u32,
}

#[derive(Debug)]
pub struct JType {
    rd: u8,
    imm_j: u32,
}

#[derive(Debug)]
pub struct IType {
    rd: u8,
    rs1: u8,
    funct3: u8,
    imm_i: u32,
}
