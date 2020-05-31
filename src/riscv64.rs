const OPCODE_LOAD: u32 =     0b000_0011; 
const OPCODE_MISC_MEM: u32 = 0b000_1111;
const OPCODE_OP_IMM: u32 =   0b001_0011; 
const OPCODE_AUIPC: u32 =    0b001_0111; 
const OPCODE_OP_IMM32: u32 = 0b001_1011; 
const OPCODE_STORE: u32 =    0b010_0011; 
const OPCODE_OP: u32 =       0b011_0011; 
const OPCODE_LUI: u32 =      0b011_0111; 
const OPCODE_BRANCH: u32 =   0b110_0011; 
const OPCODE_JALR: u32 =     0b110_0111; 
const OPCODE_JAL: u32 =      0b110_1111;
const OPCODE_SYSTEM: u32 =   0b111_0011; 

const FUNCT3_LOAD_LB: u8 = 0b000;
const FUNCT3_LOAD_LH: u8 = 0b001;
const FUNCT3_LOAD_LW: u8 = 0b010;
const FUNCT3_LOAD_LD: u8 = 0b011;
const FUNCT3_LOAD_LBU: u8 = 0b100;
const FUNCT3_LOAD_LHU: u8 = 0b101;
const FUNCT3_LOAD_LWU: u8 = 0b110;

const FUNCT3_STORE_SB: u8 = 0b000;
const FUNCT3_STORE_SH: u8 = 0b001;
const FUNCT3_STORE_SW: u8 = 0b010;
const FUNCT3_STORE_SD: u8 = 0b011;

const FUNCT3_BRANCH_BEQ: u8 = 0b000;
const FUNCT3_BRANCH_BNE: u8 = 0b001;
const FUNCT3_BRANCH_BLT: u8 = 0b100;
const FUNCT3_BRANCH_BGE: u8 = 0b101;
const FUNCT3_BRANCH_BLTU: u8 = 0b110;
const FUNCT3_BRANCH_BGEU: u8 = 0b111;

const FUNCT3_ALU_ADD_SUB: u8 = 0b000;
const FUNCT3_ALU_SLL: u8   = 0b001;
const FUNCT3_ALU_SLT: u8   = 0b010;
const FUNCT3_ALU_SLTU: u8  = 0b011;
const FUNCT3_ALU_XOR: u8   = 0b100;
const FUNCT3_ALU_SRL_SRA: u8 = 0b101;
const FUNCT3_ALU_OR: u8    = 0b110;
const FUNCT3_ALU_AND: u8   = 0b111;

const FUNCT7_ALU_SRL: u8 = 0b000_0000;
const FUNCT7_ALU_SRA: u8 = 0b010_0000;

const FUNCT7_ALU_ADD: u8 = 0b000_0000;
const FUNCT7_ALU_SUB: u8 = 0b010_0000;

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
    let rd = ((ins >> 7) & 0b1_1111) as u8;
    let rs1 = ((ins >> 15) & 0b1_1111) as u8;
    let rs2 = ((ins >> 20) & 0b1_1111) as u8;
    let funct3 = ((ins >> 12) & 0b111) as u8;
    let funct7 = ((ins >> 25) & 0b111_1111) as u8;
    let imm_i = {
        let mut val = (ins >> 20) & 0b1111_1111_1111;
        if (ins >> 31) != 0 {
            val |= 0b1111_1111_1111_1111_1111_0000_0000_0000
        }
        i32::from_ne_bytes(u32::to_ne_bytes(val))
    };
    let imm_s = {
        let mut val = ((ins >> 7) & 0b11111) | 
            (((ins >> 25) & 0b1111111) << 5);
        if (ins >> 31) != 0 {
            val |= 0b1111_1111_1111_1111_1111_0000_0000_0000
        }
        i32::from_ne_bytes(u32::to_ne_bytes(val))
    };
    let imm_b = {
        let mut val = (((ins >> 7) & 0b1) << 11) |
            (((ins >> 8) & 0b1111) << 1) | 
            (((ins >> 25) & 0b111111) << 5) |
            (((ins >> 31) & 0b1) << 12);
        if ins >> 31 != 0 { 
            val |= 0b1111_1111_1111_1111_1111_0000_0000_0000
        }
        i32::from_ne_bytes(u32::to_ne_bytes(val))
    };
    let imm_u = ins & 0b1111_1111_1111_1111_1111_0000_0000_0000;
    let imm_j = {
        (if ins & 0b1000_0000_0000_0000_0000_0000_0000_0000 != 0 {
            0b1111_1111_1111_0000_0000_0000_0000_0000
        } else { 0 }) | 
        (((ins & 0b0111_1111_1110_0000_0000_0000_0000_0000) >> 21) << 1) | 
        (((ins & 0b0000_0000_0001_0000_0000_0000_0000_0000) >> 20) << 11) | 
        (((ins & 0b0000_0000_0000_1111_1111_0000_0000_0000) >> 12) << 12)
    };
    let u_type = UType { rd, imm_u }; 
    let j_type = JType { rd, imm_j };
    let b_type = BType { rs1, rs2, funct3, imm_b };
    let i_type = IType { rd, rs1, funct3, imm_i };
    let s_type = SType { rs1, rs2, funct3, imm_s };
    let r_type = RType { rd, rs1, rs2, funct3, funct7 };
    use {Instruction::*, self::RV32I::*, self::RV64I::*};
    match opcode {
        OPCODE_LUI => Lui(u_type).into(),
        OPCODE_AUIPC => Auipc(u_type).into(),
        OPCODE_JAL => Jal(j_type).into(),
        OPCODE_JALR => Jalr(i_type).into(),
        OPCODE_BRANCH => match funct3 {
            FUNCT3_BRANCH_BEQ => Beq(b_type).into(),
            FUNCT3_BRANCH_BNE => Bne(b_type).into(),
            FUNCT3_BRANCH_BLT => Blt(b_type).into(),
            FUNCT3_BRANCH_BGE => Bge(b_type).into(),
            FUNCT3_BRANCH_BLTU => Bltu(b_type).into(),
            FUNCT3_BRANCH_BGEU => Bgeu(b_type).into(),
            _ => panic!("EILL")
        },
        OPCODE_LOAD => match funct3 {
            FUNCT3_LOAD_LB => Lb(i_type).into(),
            FUNCT3_LOAD_LH => Lh(i_type).into(),
            FUNCT3_LOAD_LW => Lw(i_type).into(),
            FUNCT3_LOAD_LD => Ld(i_type).into(),
            FUNCT3_LOAD_LBU => Lbu(i_type).into(),
            FUNCT3_LOAD_LHU => Lhu(i_type).into(),
            FUNCT3_LOAD_LWU => Lwu(i_type).into(),
            _ => panic!("EILL")
        },
        OPCODE_STORE => match funct3 {
            FUNCT3_STORE_SB => Sb(s_type).into(),
            FUNCT3_STORE_SH => Sh(s_type).into(),
            FUNCT3_STORE_SW => Sw(s_type).into(),
            FUNCT3_STORE_SD => Sd(s_type).into(),
            _ => panic!("EILL")
        },
        OPCODE_OP_IMM => match funct3 {
            FUNCT3_ALU_ADD_SUB => Addi(i_type).into(),
            FUNCT3_ALU_SLT => Slti(i_type).into(),
            FUNCT3_ALU_SLTU => Sltiu(i_type).into(),
            FUNCT3_ALU_XOR => Xori(i_type).into(),
            FUNCT3_ALU_OR => Ori(i_type).into(),
            FUNCT3_ALU_AND => Andi(i_type).into(),
            FUNCT3_ALU_SLL => Slli(i_type).into(),
            FUNCT3_ALU_SRL_SRA => match funct7 {
                FUNCT7_ALU_SRL => Srli(i_type).into(),
                FUNCT7_ALU_SRA => Srai(i_type).into(),
                _ => panic!("EILL")
            },
            _ => unreachable!()
        },
        OPCODE_OP => match funct3 {
            FUNCT3_ALU_ADD_SUB => match funct7 {
                FUNCT7_ALU_ADD => Add(r_type).into(),
                FUNCT7_ALU_SUB => Sub(r_type).into(),
                _ => panic!("EILL")
            },
            FUNCT3_ALU_SLT => Slt(r_type).into(),
            FUNCT3_ALU_SLTU => Sltu(r_type).into(),
            FUNCT3_ALU_XOR => Xor(r_type).into(),
            FUNCT3_ALU_OR => Or(r_type).into(),
            FUNCT3_ALU_AND => And(r_type).into(),
            FUNCT3_ALU_SLL => Sll(r_type).into(),
            FUNCT3_ALU_SRL_SRA => match funct7 {
                FUNCT7_ALU_SRL => Srl(r_type).into(),
                FUNCT7_ALU_SRA => Sra(r_type).into(),
                _ => panic!("EILL")
            },
            _ => unreachable!()
        },
        _ => panic!("EILL")
    }
}

#[derive(Debug)]
pub enum Instruction {
    RV32I(RV32I),
    RV64I(RV64I),
}

impl From<RV32I> for Instruction {
    fn from(src: RV32I) -> Instruction {
        Instruction::RV32I(src)
    }
}

impl From<RV64I> for Instruction {
    fn from(src: RV64I) -> Instruction {
        Instruction::RV64I(src)
    }
}

#[derive(Debug)]
pub enum RV32I {
    Lui(UType),
    Auipc(UType),
    Jal(JType),
    Jalr(IType),

    Beq(BType),
    Bne(BType),
    Blt(BType),
    Bge(BType),
    Bltu(BType),
    Bgeu(BType),

    Lb(IType),
    Lh(IType),
    Lw(IType),
    Lbu(IType),
    Lhu(IType),
    Sb(SType),
    Sh(SType),
    Sw(SType),

    Addi(IType),
    Slti(IType),
    Sltiu(IType),
    Xori(IType),
    Ori(IType),
    Andi(IType),
    Slli(IType),
    Srli(IType),
    Srai(IType),

    Add(RType),
    Sub(RType),
    Sll(RType),
    Slt(RType),
    Sltu(RType),
    Xor(RType),
    Srl(RType),
    Sra(RType),
    Or(RType),
    And(RType),
}

#[derive(Debug)]
pub enum RV64I { 
    Lwu(IType),
    Ld(IType),
    Sd(SType),
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
    imm_i: i32,
}

#[derive(Debug)]
pub struct SType {
    rs1: u8,
    rs2: u8,
    funct3: u8,
    imm_s: i32,
}

#[derive(Debug)]
pub struct BType {
    rs1: u8,
    rs2: u8,
    funct3: u8,
    imm_b: i32,
}

#[derive(Debug)]
pub struct RType {
    rd: u8,
    rs1: u8,
    rs2: u8,
    funct3: u8,
    funct7: u8,
}
