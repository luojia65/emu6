const OPCODE_LOAD: u32 =     0b000_0011; 
const OPCODE_MISC_MEM: u32 = 0b000_1111;
const OPCODE_OP_IMM: u32 =   0b001_0011; 
const OPCODE_AUIPC: u32 =    0b001_0111; 
const OPCODE_OP_IMM32: u32 = 0b001_1011; 
const OPCODE_STORE: u32 =    0b010_0011; 
const OPCODE_OP: u32 =       0b011_0011; 
const OPCODE_LUI: u32 =      0b011_0111; 
const OPCODE_OP_32: u32 =    0b011_1011; 
const OPCODE_BRANCH: u32 =   0b110_0011; 
const OPCODE_JALR: u32 =     0b110_0111; 
const OPCODE_JAL: u32 =      0b110_1111;
const OPCODE_SYSTEM: u32 =   0b111_0011; 

const OPCODE_C0: u16 =  0b00;
const OPCODE_C1: u16 =  0b01;
const OPCODE_C2: u16 =  0b10;

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

const FUNCT3_OP_ADD_SUB: u8 = 0b000;
const FUNCT3_OP_SLL: u8   = 0b001;
const FUNCT3_OP_SLT: u8   = 0b010;
const FUNCT3_OP_SLTU: u8  = 0b011;
const FUNCT3_OP_XOR: u8   = 0b100;
const FUNCT3_OP_SRL_SRA: u8 = 0b101;
const FUNCT3_OP_OR: u8    = 0b110;
const FUNCT3_OP_AND: u8   = 0b111;

const FUNCT7_OP_SRL: u8 = 0b000_0000;
const FUNCT7_OP_SRA: u8 = 0b010_0000;

const FUNCT7_OP_ADD: u8 = 0b000_0000;
const FUNCT7_OP_SUB: u8 = 0b010_0000;

const FUNCT3_SYSTEM_PRIV: u8   = 0b000;
const FUNCT3_SYSTEM_CSRRW: u8  = 0b001;
const FUNCT3_SYSTEM_CSRRS: u8  = 0b010;
const FUNCT3_SYSTEM_CSRRC: u8  = 0b011;
const FUNCT3_SYSTEM_CSRRWI: u8 = 0b101;
const FUNCT3_SYSTEM_CSRRSI: u8 = 0b110;
const FUNCT3_SYSTEM_CSRRCI: u8 = 0b111;

const FUNCT12_SYSTEM_ECALL: u32  = 0b000;
const FUNCT12_SYSTEM_EBREAK: u32 = 0b001;

const FUNCT3_MISC_MEM_FENCE: u8 = 0b000;

use crate::error::Result;
use crate::size::Usize;
use super::Xlen;
use super::imm::{Imm, Uimm};
use crate::mem64::Physical;
use thiserror::Error;

fn pc_to_mem_addr(pc: Usize) -> u64 {
    match pc {
        Usize::U32(a) => a as u64,
        Usize::U64(a) => a,
    }
}

pub struct Fetch<'a> {
    mem: &'a Physical<'a>,
    xlen: Xlen
}

impl<'a> Fetch<'a> {
    pub fn new(mem: &'a Physical<'a>, xlen: Xlen) -> Self {
        Fetch { mem, xlen }
    }

    pub fn next_instruction(&mut self, mut pc: Usize) -> Result<(Instruction, Usize)> {
        let addr = pc;
        let ins = self.next_u16(&mut pc)?;
        if ins & 0b11 != 0b11 {
            return Ok((resolve_u16(ins, self.xlen).map_err(|_| 
                FetchError::IllegalInstruction16 { addr, ins })?, pc));
        }
        if ins & 0b11100 != 0b11100 {
            let ins = (ins as u32) + ((self.next_u16(&mut pc)? as u32) << 16);
            return Ok((resolve_u32(ins, self.xlen).map_err(|_| 
                FetchError::IllegalInstruction32 { addr, ins })?, pc));
        }
        Err(FetchError::InstructionLength { addr })?
    }

    fn next_u16(&mut self, pc: &mut Usize) -> Result<u16> {
        let addr = pc_to_mem_addr(*pc);
        let ans = self.mem.fetch_ins_u16(addr);
        *pc += 2;
        ans
    }
}

#[derive(Error, Clone, Debug)]
pub enum FetchError {
    #[error("Illegal 16-bit instruction 0x{ins:04X} at address: 0x{addr:X} ")]
    IllegalInstruction16 { addr: Usize, ins: u16 },
    #[error("Illegal 32-bit instruction 0x{ins:08X} at address: 0x{addr:X} ")]
    IllegalInstruction32 { addr: Usize, ins: u32 },
    #[error("Illegal instruction at address: 0x{addr:X}; length over 32-bit is not supported")]
    InstructionLength { addr: Usize },
}

fn resolve_u16(ins: u16, xlen: Xlen) -> core::result::Result<Instruction, ()> {
    use self::RVC::*;
    let opcode = ins & 0b11;
    let funct3 = ((ins >> 13) & 0b111) as u8; // keep 0b111 to be explict (actually do not need to & 0b111)
    match (opcode, funct3) {
        
        _ => Err(())?
    }
    todo!()
}

fn resolve_u32(ins: u32, xlen: Xlen) -> core::result::Result<Instruction, ()> {
    use {self::RV32I::*, self::RV64I::*, self::RVZicsr::*};
    let opcode = ins & 0b111_1111;
    let rd = ((ins >> 7) & 0b1_1111) as u8;
    let rs1 = ((ins >> 15) & 0b1_1111) as u8;
    let rs2 = ((ins >> 20) & 0b1_1111) as u8;
    let funct3 = ((ins >> 12) & 0b111) as u8;
    let funct7 = ((ins >> 25) & 0b111_1111) as u8;
    let funct12 = (ins >> 20) & 0b1111_1111_1111;
    let imm_i = {
        let val = (ins >> 20) & 0b1111_1111_1111;
        Imm::new(val, 12)
    };
    let imm_s = {
        let val = ((ins >> 7) & 0b11111) | 
            (((ins >> 25) & 0b1111111) << 5);
        Imm::new(val, 12)
    };
    let imm_b = {
        let val = (((ins >> 7) & 0b1) << 11) |
            (((ins >> 8) & 0b1111) << 1) | 
            (((ins >> 25) & 0b111111) << 5) |
            (((ins >> 31) & 0b1) << 12);
        Imm::new(val, 12)
    };
    let imm_u = Uimm::new(ins & 0xFFFFF000, 32);
    let imm_j = {
        let val = 
            (((ins & 0b1000_0000_0000_0000_0000_0000_0000_0000) >> 31) << 20) | 
            (((ins & 0b0111_1111_1110_0000_0000_0000_0000_0000) >> 21) << 1) | 
            (((ins & 0b0000_0000_0001_0000_0000_0000_0000_0000) >> 20) << 11) | 
            (((ins & 0b0000_0000_0000_1111_1111_0000_0000_0000) >> 12) << 12);
        Imm::new(val, 12)
    };
    let csr = ((ins >> 20) & 0xFFF) as u16;
    let u_type = UType { rd, imm: imm_u }; 
    let j_type = JType { rd, imm: imm_j };
    let b_type = BType { rs1, rs2, funct3, imm: imm_b };
    let i_type = IType { rd, rs1, funct3, imm: imm_i };
    let s_type = SType { rs1, rs2, funct3, imm: imm_s };
    let r_type = RType { rd, rs1, rs2, funct3, funct7 };
    let csr_type = CsrType { rd, rs1uimm: rs1, funct3, csr };
    let ans = match opcode {
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
            _ => Err(())?
        },
        OPCODE_LOAD => match funct3 {
            FUNCT3_LOAD_LB => Lb(i_type).into(),
            FUNCT3_LOAD_LH => Lh(i_type).into(),
            FUNCT3_LOAD_LW => Lw(i_type).into(),
            FUNCT3_LOAD_LD if xlen == Xlen::X64 => Ld(i_type).into(),
            FUNCT3_LOAD_LBU => Lbu(i_type).into(),
            FUNCT3_LOAD_LHU => Lhu(i_type).into(),
            FUNCT3_LOAD_LWU => Lwu(i_type).into(),
            _ => Err(())?
        },
        OPCODE_STORE => match funct3 {
            FUNCT3_STORE_SB => Sb(s_type).into(),
            FUNCT3_STORE_SH => Sh(s_type).into(),
            FUNCT3_STORE_SW => Sw(s_type).into(),
            FUNCT3_STORE_SD if xlen == Xlen::X64 => Sd(s_type).into(),
            _ => Err(())?
        },
        OPCODE_MISC_MEM => match funct3 {
            FUNCT3_MISC_MEM_FENCE => Fence(i_type).into(),
            _ => Err(())?,
        },
        OPCODE_SYSTEM => match funct3 {
            FUNCT3_SYSTEM_PRIV => match funct12 {
                FUNCT12_SYSTEM_ECALL if funct3 == FUNCT3_SYSTEM_PRIV && rs1 == 0 && rd == 0 => 
                    Ecall(i_type).into(),
                FUNCT12_SYSTEM_EBREAK if funct3 == FUNCT3_SYSTEM_PRIV && rs1 == 0 && rd == 0 => 
                    Ebreak(i_type).into(),
                _ => Err(())?,
            },
            FUNCT3_SYSTEM_CSRRW => Csrrw(csr_type).into(),
            FUNCT3_SYSTEM_CSRRS => Csrrs(csr_type).into(),
            FUNCT3_SYSTEM_CSRRC => Csrrc(csr_type).into(),
            FUNCT3_SYSTEM_CSRRWI => Csrrwi(csr_type).into(),
            FUNCT3_SYSTEM_CSRRSI => Csrrsi(csr_type).into(),
            FUNCT3_SYSTEM_CSRRCI => Csrrci(csr_type).into(),
            _ => Err(())?,
        },
        OPCODE_OP_IMM => match funct3 {
            FUNCT3_OP_ADD_SUB => Addi(i_type).into(),
            FUNCT3_OP_SLT => Slti(i_type).into(),
            FUNCT3_OP_SLTU => Sltiu(i_type).into(),
            FUNCT3_OP_XOR => Xori(i_type).into(),
            FUNCT3_OP_OR => Ori(i_type).into(),
            FUNCT3_OP_AND => Andi(i_type).into(),
            FUNCT3_OP_SLL if funct7 == 0 && xlen == Xlen::X32 => 
                RV32I::Slli(i_type).into(),
            FUNCT3_OP_SLL if funct7 & 0b1111110 == 0 && xlen == Xlen::X64 => 
                RV64I::Slli(i_type).into(),
            FUNCT3_OP_SRL_SRA => match funct7 {
                FUNCT7_OP_SRL if xlen == Xlen::X32 => RV32I::Srli(i_type).into(),
                FUNCT7_OP_SRA if xlen == Xlen::X32 => RV32I::Srai(i_type).into(),
                x if x & 0b1111110 == FUNCT7_OP_SRL && xlen == Xlen::X64 =>
                    RV64I::Srli(i_type).into(),
                x if x & 0b1111110 == FUNCT7_OP_SRA && xlen == Xlen::X64 =>
                    RV64I::Srai(i_type).into(),
                _ => Err(())?
            },
            _ => Err(())?
        },
        OPCODE_OP => match funct3 {
            FUNCT3_OP_ADD_SUB => match funct7 {
                FUNCT7_OP_ADD => Add(r_type).into(),
                FUNCT7_OP_SUB => Sub(r_type).into(),
                _ => Err(())?
            },
            FUNCT3_OP_SLT if funct7 == 0 => Slt(r_type).into(),
            FUNCT3_OP_SLTU if funct7 == 0 => Sltu(r_type).into(),
            FUNCT3_OP_XOR if funct7 == 0 => Xor(r_type).into(),
            FUNCT3_OP_OR if funct7 == 0 => Or(r_type).into(),
            FUNCT3_OP_AND if funct7 == 0 => And(r_type).into(),
            FUNCT3_OP_SLL if funct7 == 0 && xlen == Xlen::X32 => 
                RV32I::Sll(r_type).into(),
            FUNCT3_OP_SLL if funct7 & 0b1111110 == 0 && xlen == Xlen::X64 => 
                RV64I::Sll(r_type).into(),
            FUNCT3_OP_SRL_SRA => match funct7 {
                FUNCT7_OP_SRL if xlen == Xlen::X32 => RV32I::Srl(r_type).into(),
                FUNCT7_OP_SRA if xlen == Xlen::X32 => RV32I::Sra(r_type).into(),
                FUNCT7_OP_SRL if xlen == Xlen::X64 => RV64I::Srl(r_type).into(),
                FUNCT7_OP_SRA if xlen == Xlen::X64 => RV64I::Sra(r_type).into(),
                _ => Err(())?
            },
            _ => Err(())?
        },
        OPCODE_OP_IMM32 if xlen == Xlen::X64 => match funct3 {
            FUNCT3_OP_ADD_SUB => Addiw(i_type).into(),
            FUNCT3_OP_SLL if funct7 == 0 => Slliw(i_type).into(),
            FUNCT3_OP_SRL_SRA => match funct7 {
                FUNCT7_OP_SRL => Srliw(i_type).into(),
                FUNCT7_OP_SRA => Sraiw(i_type).into(),
                _ => Err(())?,
            },
            _ => Err(())?
        },
        OPCODE_OP_32 if xlen == Xlen::X64 => match funct3 {
            FUNCT3_OP_ADD_SUB => match funct7 {
                FUNCT7_OP_ADD => Addw(r_type).into(),
                FUNCT7_OP_SUB => Subw(r_type).into(),
                _ => Err(())?
            },
            FUNCT3_OP_SLL if funct7 == 0 => Sllw(r_type).into(),
            FUNCT3_OP_SRL_SRA => match funct7 {
                FUNCT7_OP_SRL => Srlw(r_type).into(),
                FUNCT7_OP_SRA => Sraw(r_type).into(),
                _ => Err(())?,
            },
            _ => Err(())?,
        },
        _ => Err(())?
    };
    Ok(ans)
}

#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    RV32I(RV32I),
    RV64I(RV64I),
    RVC(RVC),
    RVZicsr(RVZicsr),
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

impl From<RVC> for Instruction {
    fn from(src: RVC) -> Instruction {
        Instruction::RVC(src)
    }
}

impl From<RVZicsr> for Instruction {
    fn from(src: RVZicsr) -> Instruction {
        Instruction::RVZicsr(src)
    }
}

#[derive(Debug, Clone, Copy)]
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

    Fence(IType),
    Ecall(IType),
    Ebreak(IType),
}

#[derive(Debug, Clone, Copy)]
pub enum RV64I { 
    Lwu(IType),
    Ld(IType),
    Sd(SType),

    Sll(RType),
    Srl(RType),
    Sra(RType),

    Slli(IType),
    Srli(IType),
    Srai(IType),

    Addiw(IType),
    Slliw(IType),
    Srliw(IType),
    Sraiw(IType),

    Addw(RType),
    Subw(RType),
    Sllw(RType),
    Srlw(RType),
    Sraw(RType),
}

#[derive(Debug, Clone, Copy)]
pub struct UType {
    pub rd: u8,
    pub imm: Uimm,
}

#[derive(Debug, Clone, Copy)]
pub struct JType {
    pub rd: u8,
    pub imm: Imm,
}

#[derive(Debug, Clone, Copy)]
pub struct IType {
    pub rd: u8,
    pub rs1: u8,
    pub funct3: u8,
    pub imm: Imm,
}

#[derive(Debug, Clone, Copy)]
pub struct SType {
    pub rs1: u8,
    pub rs2: u8,
    pub funct3: u8,
    pub imm: Imm,
}

#[derive(Debug, Clone, Copy)]
pub struct BType {
    pub rs1: u8,
    pub rs2: u8,
    pub funct3: u8,
    pub imm: Imm,
}

#[derive(Debug, Clone, Copy)]
pub struct RType {
    pub rd: u8,
    pub rs1: u8,
    pub rs2: u8,
    pub funct3: u8,
    pub funct7: u8,
}

#[derive(Debug, Clone, Copy)]
pub enum RVC {
    
}

#[derive(Debug, Clone, Copy)]
pub struct CRType {
    pub rdrs1: u8,
    pub rs2: u8,
    pub funct4: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct CIType {
    pub rdrs1: u8,
    pub funct3: u8,
    pub imm_ci: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct CSSType {
    pub rdrs1: u8,
    pub funct3: u8,
    pub imm_css: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct CIWType {
    pub rd: u8,
    pub funct3: u8,
    pub imm_ciw: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct CLType {
    pub rd: u8,
    pub rs1: u8,
    pub funct3: u8,
    pub imm_cl: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct CSType {
    pub rs1: u8,
    pub rs2: u8,
    pub funct3: u8,
    pub imm_cs: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct CAType {
    pub rdrs1: u8,
    pub rs2: u8,
    pub funct2: u8,
    pub funct6: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct CBType {
    pub rs1: u8,
    pub funct3: u8,
    pub off: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct CJType {
    pub funct3: u8,
    pub target: u32,
}

#[derive(Debug, Clone, Copy)]
pub enum RVZicsr {
    Csrrw(CsrType),
    Csrrs(CsrType),
    Csrrc(CsrType),
    Csrrwi(CsrType),
    Csrrsi(CsrType),
    Csrrci(CsrType),
}

#[derive(Debug, Clone, Copy)]
pub struct CsrType {
    pub rd: u8,
    pub rs1uimm: u8,
    pub funct3: u8,
    pub csr: u16,
}
