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

use crate::mem64::Physical;
use crate::error::Result;
use thiserror::Error;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Xlen {
    X32,
    X64,
}

pub struct Fetch<'a> {
    mem: &'a Physical<'a>,
    xlen: Xlen
}

impl<'a> Fetch<'a> {
    pub fn new(mem: &'a Physical<'a>, xlen: Xlen) -> Self {
        Fetch { mem, xlen }
    }

    pub fn next_instruction(&mut self, mut pc: XData) -> Result<(Instruction, XData)> {
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

    fn next_u16(&mut self, pc: &mut XData) -> Result<u16> {
        let ans = self.mem.fetch_ins_u16(pc.to_mem_addr());
        *pc += 2;
        ans
    }
}

#[derive(Error, Clone, Debug)]
pub enum FetchError {
    #[error("Illegal 16-bit instruction 0x{ins:04X} at address: 0x{addr:?} ")]
    IllegalInstruction16 { addr: XData, ins: u16 },
    #[error("Illegal 32-bit instruction 0x{ins:08X} at address: 0x{addr:?} ")]
    IllegalInstruction32 { addr: XData, ins: u32 },
    #[error("Illegal instruction at address: 0x{addr:?}; length over 32-bit is not supported")]
    InstructionLength { addr: XData },
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
        let mut val = (if ins & 0b1000_0000_0000_0000_0000_0000_0000_0000 != 0 {
            0b1111_1111_1111_0000_0000_0000_0000_0000
        } else { 0 }) | 
        (((ins & 0b0111_1111_1110_0000_0000_0000_0000_0000) >> 21) << 1) | 
        (((ins & 0b0000_0000_0001_0000_0000_0000_0000_0000) >> 20) << 11) | 
        (((ins & 0b0000_0000_0000_1111_1111_0000_0000_0000) >> 12) << 12);
        if ins >> 31 != 0 { 
            val |= 0b1111_1111_1111_1111_1111_0000_0000_0000
        }
        i32::from_ne_bytes(u32::to_ne_bytes(val))
    };
    let csr = ((ins >> 20) & 0xFFF) as u16;
    let u_type = UType { rd, imm_u }; 
    let j_type = JType { rd, imm_j };
    let b_type = BType { rs1, rs2, funct3, imm_b };
    let i_type = IType { rd, rs1, funct3, imm_i };
    let s_type = SType { rs1, rs2, funct3, imm_s };
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
    rd: u8,
    imm_u: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct JType {
    rd: u8,
    imm_j: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct IType {
    rd: u8,
    rs1: u8,
    funct3: u8,
    imm_i: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct SType {
    rs1: u8,
    rs2: u8,
    funct3: u8,
    imm_s: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct BType {
    rs1: u8,
    rs2: u8,
    funct3: u8,
    imm_b: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct RType {
    rd: u8,
    rs1: u8,
    rs2: u8,
    funct3: u8,
    funct7: u8,
}

#[derive(Debug, Clone, Copy)]
pub enum RVC {
    
}

#[derive(Debug, Clone, Copy)]
pub struct CRType {
    rdrs1: u8,
    rs2: u8,
    funct4: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct CIType {
    rdrs1: u8,
    funct3: u8,
    imm_ci: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct CSSType {
    rdrs1: u8,
    funct3: u8,
    imm_css: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct CIWType {
    rd: u8,
    funct3: u8,
    imm_ciw: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct CLType {
    rd: u8,
    rs1: u8,
    funct3: u8,
    imm_cl: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct CSType {
    rs1: u8,
    rs2: u8,
    funct3: u8,
    imm_cs: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct CAType {
    rdrs1: u8,
    rs2: u8,
    funct2: u8,
    funct6: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct CBType {
    rs1: u8,
    funct3: u8,
    off: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct CJType {
    funct3: u8,
    target: u32,
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
    rd: u8,
    rs1uimm: u8,
    funct3: u8,
    csr: u16,
}

#[derive(Debug)]
pub struct XReg {
    x: [XData; 32],
}

// X64s > X32s; however in impl this cannot happen
#[derive(Debug, Clone, Copy, Ord, Eq, PartialEq)]
pub enum XData {
    X32(u32),
    X64(u64),
}

impl core::cmp::PartialOrd for XData {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use XData::*;
        match (self, other) {
            (X32(a), X32(b)) => a.partial_cmp(b),
            (X64(a), X64(b)) => a.partial_cmp(b),
            _ => None
        } 
    }
}

impl core::cmp::PartialEq<u32> for XData {
    fn eq(&self, other: &u32) -> bool {
        use XData::*;
        match self {
            X32(a) => *a == *other,
            X64(a) => *a == *other as u64,
        } 
    }
}

impl core::cmp::PartialOrd<u32> for XData {
    fn partial_cmp(&self, other: &u32) -> Option<std::cmp::Ordering> {
        use XData::*;
        match self {
            X32(a) => a.partial_cmp(other),
            X64(a) => a.partial_cmp(&(*other as u64)),
        } 
    }
}

#[derive(Debug, Clone, Copy, Ord, Eq, PartialEq)]
pub enum XDataI {
    X32(i32),
    X64(i64),
}

impl core::cmp::PartialOrd for XDataI {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use XDataI::*;
        match (self, other) {
            (X32(a), X32(b)) => a.partial_cmp(b),
            (X64(a), X64(b)) => a.partial_cmp(b),
            _ => None
        } 
    }
}

impl core::cmp::PartialEq<i32> for XDataI {
    fn eq(&self, other: &i32) -> bool {
        use XDataI::*;
        match self {
            X32(a) => *a == *other,
            X64(a) => *a == *other as i64,
        } 
    }
}

impl core::cmp::PartialOrd<i32> for XDataI {
    fn partial_cmp(&self, other: &i32) -> Option<std::cmp::Ordering> {
        use XDataI::*;
        match self {
            X32(a) => a.partial_cmp(other),
            X64(a) => a.partial_cmp(&(*other as i64)),
        } 
    }
}

impl XData {
    fn set_zext_u8(&mut self, a: u8) {
        match self {
            XData::X32(data) => *data = a as u32,
            XData::X64(data) => *data = a as u64,
        }
    }

    fn set_zext_u16(&mut self, a: u16) {
        match self {
            XData::X32(data) => *data = a as u32,
            XData::X64(data) => *data = a as u64,
        }
    }

    fn set_zext_u32(&mut self, a: u32) {
        match self {
            XData::X32(data) => *data = a,
            XData::X64(data) => *data = a as u64,
        }
    }

    fn set_sext_u8(&mut self, a: u8) {
        match self {
            XData::X32(data) => 
                *data = (a as u32) | if (a >> 7) != 0 { 0xFFFFFF00 } else { 0 },
            XData::X64(data) => 
                *data = (a as u64) | if (a >> 7) != 0 { 0xFFFFFFFFFFFFFF00 } else { 0 },
        }
    }

    fn set_sext_u16(&mut self, a: u16) {
        match self {
            XData::X32(data) => 
                *data = (a as u32) | if (a >> 7) != 0 { 0xFFFF0000 } else { 0 },
            XData::X64(data) => 
                *data = (a as u64) | if (a >> 7) != 0 { 0xFFFFFFFFFFFF0000 } else { 0 },
        }
    }

    fn set_sext_u32(&mut self, a: u32) {
        match self {
            XData::X32(data) => *data = a,
            XData::X64(data) => 
                *data = (a as u64) | if (a >> 7) != 0 { 0xFFFFFFFF00000000 } else { 0 },
        }
    }

    fn set_sext_u64(&mut self, a: u64) {
        match self {
            XData::X32(_) => panic!("Write u64 on XLEN < u64"),
            XData::X64(data) => *data = a,
        }
    }

    fn lowbit_u8(self) -> u8 {
        match self {
            XData::X32(data) => (data & 0xFF) as u8,
            XData::X64(data) => (data & 0xFF) as u8,
        }
    }

    fn lowbit_u16(self) -> u16 {
        match self {
            XData::X32(data) => (data & 0xFFFF) as u16,
            XData::X64(data) => (data & 0xFFFF) as u16,
        }
    }

    fn lowbit_u32(self) -> u32 {
        match self {
            XData::X32(data) => data,
            XData::X64(data) => (data & 0xFFFFFFFF) as u32,
        }
    }

    fn to_mem_addr(self) -> u64 {
        match self {
            XData::X32(data) => data as u64,
            XData::X64(data) => data,
        }
    }

    fn as_signed(self) -> XDataI {
        match self {
            XData::X32(data) => XDataI::X32(i32::from_ne_bytes(u32::to_ne_bytes(data))),
            XData::X64(data) => XDataI::X64(i64::from_ne_bytes(u64::to_ne_bytes(data))),
        }
    }
}

impl core::ops::Add<u32> for XData {
    type Output = XData;
    fn add(self, rhs: u32) -> Self::Output {
        match self {
            XData::X32(u32_val) => 
                XData::X32(u32_val.wrapping_add(rhs)),
            XData::X64(u64_val) => 
                XData::X64(u64_val.wrapping_add(rhs as u64)),
        }
    }
}

impl core::ops::Add<i32> for XData {
    type Output = XData;
    fn add(self, rhs: i32) -> Self::Output {
        match self {
            XData::X32(u32_val) => 
                XData::X32(if rhs > 0 {
                    u32_val.wrapping_add(rhs as u32)
                } else { 
                    u32_val.wrapping_sub(rhs.abs() as u32)
                }),
            XData::X64(u64_val) => 
                XData::X64(if rhs > 0 {
                    u64_val.wrapping_add(rhs as u64)
                } else { 
                    u64_val.wrapping_sub(rhs.abs() as u64)
                }),
        }
    }
}

impl core::ops::AddAssign<i32> for XData {
    fn add_assign(&mut self, rhs: i32) {
        match self {
            XData::X32(u32_val) => 
                *u32_val = if rhs > 0 {
                    u32_val.wrapping_add(rhs as u32)
                } else { 
                    u32_val.wrapping_sub(rhs.abs() as u32)
                },
            XData::X64(u64_val) => 
                *u64_val = if rhs > 0 {
                    u64_val.wrapping_add(rhs as u64)
                } else { 
                    u64_val.wrapping_sub(rhs.abs() as u64)
                },
        }
    }
}

pub struct Csr {
    csr: [u64; 4096],
}

pub struct Execute<'a> {
    data_mem: &'a mut Physical<'a>,
    xreg: Box<XReg>,
    csrs: Box<Csr>,
    xlen: Xlen,
}

impl<'a> core::fmt::Debug for Execute<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Execute")
            .field("x", &self.xreg)
            .field("xlen", &self.xlen)
            .finish()
    }
}

impl<'a> Execute<'a> {
    pub fn new(data_mem: &'a mut Physical<'a>, xlen: Xlen) -> Execute<'a> {
        let init = match xlen {
            Xlen::X32 => XData::X32(0),
            Xlen::X64 => XData::X64(0),
        };
        Execute { 
            data_mem,
            xreg: Box::new(XReg { x: [init; 32] }),
            csrs: Box::new(Csr { csr: [0u64; 4096] }),
            xlen
        }
    }

    pub fn execute(&mut self, ins: Instruction, pc: XData, pc_nxt: &mut XData) -> Result<()> {
        use {Instruction::*, self::RV32I::*, self::RV64I::*, self::RVZicsr::*};
        match ins {
            RV32I(Lui(u)) => self.xw(u.rd).set_zext_u32(u.imm_u),
            RV32I(Auipc(u)) => *self.xw(u.rd) = pc + u.imm_u,
            RV32I(Jal(j)) => {
                let pc_link = *pc_nxt;
                *pc_nxt = pc + j.imm_j;
                *self.xw(j.rd) = pc_link;
            },
            RV32I(Jalr(i)) => {
                let pc_link = *pc_nxt;
                *pc_nxt = *self.xr(i.rs1) + i.imm_i;
                *self.xw(i.rd) = pc_link;
            },
            RV32I(Beq(b)) => {
                if self.xr(b.rs1) == self.xr(b.rs2) {
                    *pc_nxt = pc + b.imm_b
                }
            },
            RV32I(Bne(b)) => {
                if self.xr(b.rs1) != self.xr(b.rs2) {
                    *pc_nxt = pc + b.imm_b
                }
            },
            RV32I(Blt(b)) => {
                if self.xr(b.rs1).as_signed() < self.xr(b.rs2).as_signed() {
                    *pc_nxt = pc + b.imm_b
                }
            },
            RV32I(Bge(b)) => {
                if self.xr(b.rs1).as_signed() >= self.xr(b.rs2).as_signed() {
                    *pc_nxt = pc + b.imm_b
                }
            },
            RV32I(Bltu(b)) => {
                if self.xr(b.rs1) < self.xr(b.rs2) {
                    *pc_nxt = pc + b.imm_b
                }
            },
            RV32I(Bgeu(b)) => {
                if self.xr(b.rs1) >= self.xr(b.rs2) {
                    *pc_nxt = pc + b.imm_b
                }
            },
            RV32I(Lb(i)) => {
                let addr = (*self.xr(i.rs1) + i.imm_i).to_mem_addr();
                let data = self.data_mem.read_u8(addr)?;
                self.xw(i.rd).set_sext_u8(data);
            },
            RV32I(Lh(i)) => {
                let addr = (*self.xr(i.rs1) + i.imm_i).to_mem_addr();
                let data = self.data_mem.read_u16(addr)?;
                self.xw(i.rd).set_sext_u16(data);
            },
            RV32I(Lw(i)) => {
                let addr = (*self.xr(i.rs1) + i.imm_i).to_mem_addr();
                let data = self.data_mem.read_u32(addr)?;
                self.xw(i.rd).set_sext_u32(data);
            },
            RV32I(Lbu(i)) => {
                let addr = (*self.xr(i.rs1) + i.imm_i).to_mem_addr();
                let data = self.data_mem.read_u8(addr)?;
                self.xw(i.rd).set_zext_u8(data);
            },
            RV32I(Lhu(i)) => {
                let addr = (*self.xr(i.rs1) + i.imm_i).to_mem_addr();
                let data = self.data_mem.read_u16(addr)?;
                self.xw(i.rd).set_zext_u16(data);
            },
            RV32I(Sb(s)) => self.data_mem.write_u8(
                (*self.xr(s.rs1) + s.imm_s).to_mem_addr(),
                self.xr(s.rs2).lowbit_u8()
            )?,
            RV32I(Sh(s)) => self.data_mem.write_u16(
                (*self.xr(s.rs1) + s.imm_s).to_mem_addr(),
                self.xr(s.rs2).lowbit_u16()
            )?,
            RV32I(Sw(s)) => self.data_mem.write_u32(
                (*self.xr(s.rs1) + s.imm_s).to_mem_addr(),
                self.xr(s.rs2).lowbit_u32()
            )?,
            RV32I(Addi(i)) => 
                *self.xw(i.rd) = *self.xr(i.rs1) + i.imm_i,
            RV32I(Slti(i)) => {
                let value = if self.xr(i.rs1).as_signed() < i.imm_i
                    { 1 } else { 0 };
                self.xw(i.rd).set_zext_u8(value);
            },
            RV32I(Sltiu(i)) => {
                let imm = u32::from_ne_bytes(i32::to_ne_bytes(i.imm_i));
                let value = if *self.xr(i.rs1) < imm { 1 } else { 0 };
                self.xw(i.rd).set_zext_u8(value);
            },
            RV32I(Ori(i)) => {
                let imm = u64::from_ne_bytes(i64::to_ne_bytes(i.imm_i as i64));
                self.reg_w(i.rd, self.reg_r(i.rs1) | imm);
            },
            RV32I(Andi(i)) => {
                let imm = u64::from_ne_bytes(i64::to_ne_bytes(i.imm_i as i64));
                self.reg_w(i.rd, self.reg_r(i.rs1) & imm);
            },
            RV32I(Xori(i)) => {
                let imm = u64::from_ne_bytes(i64::to_ne_bytes(i.imm_i as i64));
                self.reg_w(i.rd, self.reg_r(i.rs1) ^ imm);
            },
            RV32I(self::RV32I::Slli(i)) => {
                let shamt = shamt_from_imm_xlen32(i.imm_i);
                self.reg_w(i.rd, self.reg_r(i.rs1) << shamt);
            },
            RV32I(self::RV32I::Srli(i)) => {
                let shamt = shamt_from_imm_xlen32(i.imm_i);
                self.reg_w(i.rd, self.reg_r(i.rs1) >> shamt);
            },
            RV32I(self::RV32I::Srai(i)) => {
                let shamt = shamt_from_imm_xlen32(i.imm_i);
                let sra = self.reg_r_i64(i.rs1) >> shamt;
                self.reg_w(i.rd, u64::from_ne_bytes(i64::to_ne_bytes(sra)));
            },
            RV32I(Add(r)) => self.reg_w(r.rd, 
                self.reg_r(r.rs1).wrapping_add(self.reg_r(r.rs2))),
            RV32I(Sub(r)) => self.reg_w(r.rd, 
                self.reg_r(r.rs1).wrapping_sub(self.reg_r(r.rs2))),
            RV32I(self::RV32I::Sll(r)) => {
                let shamt = shamt_from_reg_xlen32(self.reg_r(r.rs2));
                self.reg_w(r.rd, self.reg_r(r.rs1) << shamt);
            },
            RV32I(Slt(r)) => {
                let value = if self.reg_r_i64(r.rs1) < self.reg_r_i64(r.rs2)
                    { 1 } else { 0 };
                self.reg_w(r.rd, value);
            },
            RV32I(Sltu(r)) => {
                let value = if self.reg_r(r.rs1) < self.reg_r(r.rs2) 
                    { 1 } else { 0 };
                self.reg_w(r.rd, value);
            },
            RV32I(Xor(r)) => {
                self.reg_w(r.rd, self.reg_r(r.rs1) ^ self.reg_r(r.rs2));
            },
            RV32I(self::RV32I::Srl(r)) => { 
                let shamt = shamt_from_reg_xlen32(self.reg_r(r.rs2));
                self.reg_w(r.rd, self.reg_r(r.rs1) >> shamt);
            },
            RV32I(self::RV32I::Sra(r)) => {
                let shamt = shamt_from_reg_xlen32(self.reg_r(r.rs2));
                let sra = self.reg_r_i64(r.rs1) >> shamt;
                self.reg_w(r.rd, u64::from_ne_bytes(i64::to_ne_bytes(sra)));
            },
            RV32I(Or(r)) => {
                self.reg_w(r.rd, self.reg_r(r.rs1) | self.reg_r(r.rs2));
            },
            RV32I(And(r)) => {
                self.reg_w(r.rd, self.reg_r(r.rs1) & self.reg_r(r.rs2));
            },
            RV64I(Ld(i)) => self.reg_w(i.rd, 
                self.data_mem.read_u64(u64_add_i32(self.reg_r(i.rs1), i.imm_i))?),
            RV64I(Sd(s)) => self.data_mem.write_u64(
                u64_add_i32(self.reg_r(s.rs1), s.imm_s),
                self.reg_r(s.rs2)
            )?,
            RV64I(self::RV64I::Slli(i)) => {
                let shamt = shamt_from_imm_xlen64(i.imm_i);
                self.reg_w(i.rd, self.reg_r(i.rs1) << shamt);
            },
            RV64I(self::RV64I::Srli(i)) => {
                let shamt = shamt_from_imm_xlen64(i.imm_i);
                self.reg_w(i.rd, self.reg_r(i.rs1) >> shamt);
            },
            RV64I(self::RV64I::Srai(i)) => {
                let shamt = shamt_from_imm_xlen64(i.imm_i);
                let sra = self.reg_r_i64(i.rs1) >> shamt;
                self.reg_w(i.rd, u64::from_ne_bytes(i64::to_ne_bytes(sra)));
            },
            RV64I(self::RV64I::Sll(r)) => {
                let shamt = shamt_from_reg_xlen64(self.reg_r(r.rs2));
                self.reg_w(r.rd, self.reg_r(r.rs1) << shamt);
            },
            RV64I(self::RV64I::Srl(r)) => {
                let shamt = shamt_from_reg_xlen64(self.reg_r(r.rs2));
                self.reg_w(r.rd, self.reg_r(r.rs1) >> shamt);
            },
            RV64I(self::RV64I::Sra(r)) => {
                let shamt = shamt_from_reg_xlen64(self.reg_r(r.rs2));
                let sra = self.reg_r_i64(r.rs1) >> shamt;
                self.reg_w(r.rd, u64::from_ne_bytes(i64::to_ne_bytes(sra)));
            },
            // side effect?
            RVZicsr(Csrrw(csr)) => {
                self.reg_w(csr.rd, self.csr_r(csr.csr));
                self.csr_w(csr.csr, self.reg_r(csr.rs1uimm));
            },
            RVZicsr(Csrrs(csr)) => {
                self.reg_w(csr.rd, self.csr_r(csr.csr));
                self.csr_w(csr.csr, self.csr_r(csr.csr) | self.reg_r(csr.rs1uimm));
            },
            RVZicsr(Csrrc(csr)) => {
                self.reg_w(csr.rd, self.csr_r(csr.csr));
                self.csr_w(csr.csr, self.csr_r(csr.csr) & !self.reg_r(csr.rs1uimm));
            },
            RVZicsr(Csrrwi(csr)) => {
                self.reg_w(csr.rd, self.csr_r(csr.csr));
                self.csr_w(csr.csr, csr.rs1uimm as u64);
            },
            RVZicsr(Csrrsi(csr)) => {
                self.reg_w(csr.rd, self.csr_r(csr.csr));
                self.csr_w(csr.csr, self.csr_r(csr.csr) | (csr.rs1uimm as u64));
            },
            RVZicsr(Csrrci(csr)) => {
                self.reg_w(csr.rd, self.csr_r(csr.csr));
                self.csr_w(csr.csr, self.csr_r(csr.csr) & !(csr.rs1uimm as u64));
            },
            _ => panic!("todo"),
        }
        Ok(())
    }

    pub(crate) fn dump_regs(&self) {
        println!("{:?}", self.xreg);
    }

    fn xr(&self, index: u8) -> &XData {
        &self.xreg.x[index as usize]
    }

    fn xw(&mut self, index: u8) -> &mut XData {
        &mut self.xreg.x[index as usize]
    }

    fn reg_r(&self, index: u8) -> u64 {
        if index == 0 { return 0 }
        // match &self.xreg {
        //     XReg::X32(x) => x[index as usize] as u64,
        //     XReg::X64(x) => x[index as usize],
        // }
        todo!()
    }

    fn reg_r_i64(&self, index: u8) -> i64 {
        if index == 0 { return 0 }
        // let data = match &self.xreg {
        //     XReg::X32(x) => x[index as usize] as u64,
        //     XReg::X64(x) => x[index as usize],
        // };
        // i64::from_ne_bytes(u64::to_ne_bytes(data))
        todo!()
    }

    fn reg_w(&mut self, index: u8, data: u64) {
        if index == 0 { return }
        // match &mut self.xreg {
        //     XReg::X32(x) => x[index as usize] = data as u32,
        //     XReg::X64(x) => x[index as usize] = data,
        // }
        todo!()
    }

    fn csr_r(&self, csr: u16) -> u64 {
        match self.xlen {
            Xlen::X32 => self.csrs.csr[csr as usize] & 0xFFFFFFFF,
            Xlen::X64 => self.csrs.csr[csr as usize],
        }
    }

    fn csr_w(&mut self, csr: u16, data: u64) {
        match self.xlen {
            Xlen::X32 => self.csrs.csr[csr as usize] = data & 0xFFFFFFFF,
            Xlen::X64 => self.csrs.csr[csr as usize] = data,
        }
    }
}

fn sext_u8_u64(i: u8) -> u64 {
    (i as u64) | if (i >> 7) != 0 { 0xFFFFFFFFFFFFFF00 } else { 0 }
}

fn sext_u16_u64(i: u16) -> u64 {
    (i as u64) | if (i >> 15) != 0 { 0xFFFFFFFFFFFF0000 } else { 0 }
}

fn sext_u32_u64(i: u32) -> u64 {
    (i as u64) | if (i >> 31) != 0 { 0xFFFFFFFF00000000 } else { 0 }
}

fn lowbit_u64_u8(i: u64) -> u8 {
    (i & 0xFF) as u8
}

fn lowbit_u64_u16(i: u64) -> u16 {
    (i & 0xFFFF) as u16
}

fn lowbit_u64_u32(i: u64) -> u32 {
    (i & 0xFFFFFFFF) as u32
}

fn u64_add_i32(base: u64, off: i32) -> u64 {
    if off > 0 {
        base.wrapping_add(off as u64)
    } else { 
        base.wrapping_sub(off.abs() as u64)
    }
}

fn shamt_from_imm_xlen32(imm: i32) -> u8 {
    (u32::from_ne_bytes(i32::to_ne_bytes(imm)) & 0x1F) as u8
}

fn shamt_from_imm_xlen64(imm: i32) -> u8 {
    (u32::from_ne_bytes(i32::to_ne_bytes(imm)) & 0x3F) as u8
}

fn shamt_from_reg_xlen32(val: u64) -> u8 {
    (val & 0x1F) as u8
}

fn shamt_from_reg_xlen64(val: u64) -> u8 {
    (val & 0x3F) as u8
}
