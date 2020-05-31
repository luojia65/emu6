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

use crate::mem64::Physical;
use crate::error::Result;
use thiserror::Error;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Xlen {
    X32,
    X64
}

pub struct Fetch<'a> {
    mem: &'a Physical<'a>,
    xlen: Xlen
}

impl<'a> Fetch<'a> {
    pub fn new(mem: &'a Physical<'a>, xlen: Xlen) -> Self {
        Fetch { mem, xlen }
    }

    pub fn next_instruction(&mut self, mut pc: u64) -> Result<(Instruction, u64)> {
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

    fn next_u16(&mut self, pc: &mut u64) -> Result<u16> {
        let ans = self.mem.fetch_ins_u16(*pc);
        *pc += 2;
        ans
    }
}

#[derive(Error, Clone, Debug)]
pub enum FetchError {
    #[error("Illegal 16-bit instruction 0x{ins:04X} at address: 0x{addr:016X} ")]
    IllegalInstruction16 { addr: u64, ins: u16 },
    #[error("Illegal 32-bit instruction 0x{ins:08X} at address: 0x{addr:016X} ")]
    IllegalInstruction32 { addr: u64, ins: u32 },
    #[error("Illegal instruction at address: 0x{addr:016X}; length over 32-bit is not supported")]
    InstructionLength { addr: u64 },
}

fn resolve_u16(ins: u16, xlen: Xlen) -> core::result::Result<Instruction, ()> {
    use self::RVC::*;
    todo!()
}

fn resolve_u32(ins: u32, xlen: Xlen) -> core::result::Result<Instruction, ()> {
    use {self::RV32I::*, self::RV64I::*};
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
    let u_type = UType { rd, imm_u }; 
    let j_type = JType { rd, imm_j };
    let b_type = BType { rs1, rs2, funct3, imm_b };
    let i_type = IType { rd, rs1, funct3, imm_i };
    let s_type = SType { rs1, rs2, funct3, imm_s };
    let r_type = RType { rd, rs1, rs2, funct3, funct7 };
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
        OPCODE_OP_IMM => match funct3 {
            FUNCT3_ALU_ADD_SUB => Addi(i_type).into(),
            FUNCT3_ALU_SLT => Slti(i_type).into(),
            FUNCT3_ALU_SLTU => Sltiu(i_type).into(),
            FUNCT3_ALU_XOR => Xori(i_type).into(),
            FUNCT3_ALU_OR => Ori(i_type).into(),
            FUNCT3_ALU_AND => Andi(i_type).into(),
            FUNCT3_ALU_SLL if funct7 == 0 && xlen == Xlen::X32 => 
                RV32I::Slli(i_type).into(),
            FUNCT3_ALU_SLL if funct7 & 0b1111110 == 0 && xlen == Xlen::X64 => 
                RV64I::Slli(i_type).into(),
            FUNCT3_ALU_SRL_SRA => match funct7 {
                FUNCT7_ALU_SRL if xlen == Xlen::X32 => RV32I::Srli(i_type).into(),
                FUNCT7_ALU_SRA if xlen == Xlen::X32 => RV32I::Srai(i_type).into(),
                x if x & 0b1111110 == FUNCT7_ALU_SRL && xlen == Xlen::X64 =>
                    RV64I::Srli(i_type).into(),
                x if x & 0b1111110 == FUNCT7_ALU_SRA && xlen == Xlen::X64 =>
                    RV64I::Srai(i_type).into(),
                _ => Err(())?
            },
            _ => Err(())?
        },
        OPCODE_OP => match funct3 {
            FUNCT3_ALU_ADD_SUB => match funct7 {
                FUNCT7_ALU_ADD => Add(r_type).into(),
                FUNCT7_ALU_SUB => Sub(r_type).into(),
                _ => Err(())?
            },
            FUNCT3_ALU_SLT if funct7 == 0 => Slt(r_type).into(),
            FUNCT3_ALU_SLTU if funct7 == 0 => Sltu(r_type).into(),
            FUNCT3_ALU_XOR if funct7 == 0 => Xor(r_type).into(),
            FUNCT3_ALU_OR if funct7 == 0 => Or(r_type).into(),
            FUNCT3_ALU_AND if funct7 == 0 => And(r_type).into(),
            FUNCT3_ALU_SLL if funct7 == 0 && xlen == Xlen::X32 => 
                RV32I::Sll(r_type).into(),
            FUNCT3_ALU_SLL if funct7 & 0b1111110 == 0 && xlen == Xlen::X64 => 
                RV64I::Sll(r_type).into(),
            FUNCT3_ALU_SRL_SRA => match funct7 {
                FUNCT7_ALU_SRL if xlen == Xlen::X32 => RV32I::Srl(r_type).into(),
                FUNCT7_ALU_SRA if xlen == Xlen::X32 => RV32I::Sra(r_type).into(),
                FUNCT7_ALU_SRL if xlen == Xlen::X64 => RV64I::Srl(r_type).into(),
                FUNCT7_ALU_SRA if xlen == Xlen::X64 => RV64I::Sra(r_type).into(),
                _ => Err(())?
            },
            _ => Err(())?
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
    // todo!
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

#[derive(Debug)]
pub struct IntRegister {
    x: [u64; 32],
}

// pub struct Csr {
//     inner: [u64; 4096],
// }

#[derive(Debug)]
pub struct Execute<'a> {
    data_mem: &'a mut Physical<'a>,
    regs: Box<IntRegister>,
}

impl<'a> Execute<'a> {
    pub fn new(data_mem: &'a mut Physical<'a>) -> Execute<'a> {
        Execute { 
            data_mem,
            regs: Box::new(IntRegister { x: [0u64; 32] }) 
        }
    }

    pub fn execute(&mut self, ins: Instruction, pc: u64, pc_nxt: &mut u64) -> Result<()> {
        use {Instruction::*, self::RV32I::*, self::RV64I::*};
        match ins {
            RV32I(Lui(u)) => self.reg_w(u.rd, sext_u32_u64(u.imm_u)),
            RV32I(Auipc(u)) => self.reg_w(u.rd, pc.wrapping_add(sext_u32_u64(u.imm_u))),
            RV32I(Jal(j)) => {
                let pc_link = *pc_nxt;
                *pc_nxt = u64_add_i32(pc, j.imm_j);
                self.reg_w(j.rd, pc_link);
            },
            RV32I(Jalr(i)) => {
                let pc_link = *pc_nxt;
                *pc_nxt = u64_add_i32(self.reg_r(i.rs1), i.imm_i);
                self.reg_w(i.rd, pc_link);
            },
            RV32I(Beq(b)) => {
                if self.reg_r(b.rs1) == self.reg_r(b.rs2) {
                    *pc_nxt = u64_add_i32(pc, b.imm_b)
                }
            },
            RV32I(Bne(b)) => {
                if self.reg_r(b.rs1) != self.reg_r(b.rs2) {
                    *pc_nxt = u64_add_i32(pc, b.imm_b)
                }
            },
            RV32I(Blt(b)) => {
                if self.reg_r_i64(b.rs1) < self.reg_r_i64(b.rs2) {
                    *pc_nxt = u64_add_i32(pc, b.imm_b)
                }
            },
            RV32I(Bge(b)) => {
                if self.reg_r_i64(b.rs1) >= self.reg_r_i64(b.rs2) {
                    *pc_nxt = u64_add_i32(pc, b.imm_b)
                }
            },
            RV32I(Bltu(b)) => {
                if self.reg_r(b.rs1) < self.reg_r(b.rs2) {
                    *pc_nxt = u64_add_i32(pc, b.imm_b)
                }
            },
            RV32I(Bgeu(b)) => {
                if self.reg_r(b.rs1) >= self.reg_r(b.rs2) {
                    *pc_nxt = u64_add_i32(pc, b.imm_b)
                }
            },
            RV32I(Lb(i)) => self.reg_w(i.rd, sext_u8_u64(
                self.data_mem.read_u8(u64_add_i32(self.reg_r(i.rs1), i.imm_i))?)),
            RV32I(Lh(i)) => self.reg_w(i.rd, sext_u16_u64(
                self.data_mem.read_u16(u64_add_i32(self.reg_r(i.rs1), i.imm_i))?)),
            RV32I(Lw(i)) => self.reg_w(i.rd, sext_u32_u64(
                self.data_mem.read_u32(u64_add_i32(self.reg_r(i.rs1), i.imm_i))?)),
            RV32I(Lbu(i)) => self.reg_w(i.rd, 
                self.data_mem.read_u8(u64_add_i32(self.reg_r(i.rs1), i.imm_i))? as u64),
            RV32I(Lhu(i)) => self.reg_w(i.rd, 
                self.data_mem.read_u16(u64_add_i32(self.reg_r(i.rs1), i.imm_i))? as u64),
            RV32I(Sb(s)) => self.data_mem.write_u8(
                u64_add_i32(self.reg_r(s.rs1), s.imm_s),
                lowbit_u64_u8(self.reg_r(s.rs2) )
            )?,
            RV32I(Sh(s)) => self.data_mem.write_u16(
                u64_add_i32(self.reg_r(s.rs1), s.imm_s),
                lowbit_u64_u16(self.reg_r(s.rs2) )
            )?,
            RV32I(Sw(s)) => self.data_mem.write_u32(
                u64_add_i32(self.reg_r(s.rs1), s.imm_s),
                lowbit_u64_u32(self.reg_r(s.rs2) )
            )?,
            RV32I(Addi(i)) => 
                self.reg_w(i.rd, u64_add_i32(self.reg_r(i.rs1), i.imm_i)),
            RV32I(Slti(i)) => {
                let value = if self.reg_r_i64(i.rs1) < (i.imm_i as i64)
                    { 1 } else { 0 };
                self.reg_w(i.rd, value);
            },
            RV32I(Sltiu(i)) => {
                let imm = u64::from_ne_bytes(i64::to_ne_bytes(i.imm_i as i64));
                let value = if self.reg_r(i.rs1) < imm { 1 } else { 0 };
                self.reg_w(i.rd, value);
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
            RV64I(Ld(i)) => self.reg_w(i.rd, 
                self.data_mem.read_u64(u64_add_i32(self.reg_r(i.rs1), i.imm_i))?),
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
            _ => panic!("todo"),
        }
        Ok(())
    }

    pub(crate) fn dump_regs(&self) {
        println!("{:?}", self.regs);
    }

    fn reg_r(&self, index: u8) -> u64 {
        self.regs.x[index as usize]
    }

    fn reg_r_i64(&self, index: u8) -> i64 {
        i64::from_ne_bytes(u64::to_ne_bytes(self.regs.x[index as usize]))
    }

    fn reg_w(&mut self, index: u8, data: u64) {
        self.regs.x[index as usize] = data
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
