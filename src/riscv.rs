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

use crate::mmu64::Physical;
use crate::mmu64::Result;

pub struct Fetch<'a> {
    pub mem: &'a Physical<'a>
}

impl<'a> Fetch<'a> {
    pub fn new(mem: &'a Physical<'a>) -> Self {
        Fetch { mem }
    }

    pub fn next_instruction(&mut self, mut pc: u64) -> Result<(Instruction, u64)> {
        let ins = self.next_u16(&mut pc)?;
        if ins & 0b11 != 0b11 {
            println!("16 bit");
            return Ok((resolve_u16(ins), pc));
        }
        if ins & 0b11100 != 0b11100 {
            let ins = (ins as u32) + ((self.next_u16(&mut pc)? as u32) << 16);
            // println!("32 bit {:08X}", ins);
            return Ok((resolve_u32(ins), pc));
        }
        todo!()
    }

    fn next_u16(&mut self, pc: &mut u64) -> Result<u16> {
        let ans = self.mem.fetch_ins_u16(*pc);
        *pc += 2;
        ans
    }
}

fn resolve_u16(ins: u16) -> Instruction {
    use {Instruction::*, self::RVC::*};
    todo!()
}

fn resolve_u32(ins: u32) -> Instruction {
    use {Instruction::*, self::RV32I::*, self::RV64I::*};
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
    imm_j: u32,
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
    regs: [u64; 32],
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
            regs: Box::new(IntRegister { regs: [0u64; 32] }) 
        }
    }

    pub fn execute(&mut self, ins: Instruction, pc: u64) -> Result<()> {
        use {Instruction::*, self::RV32I::*, self::RV64I::*};
        match ins {
            RV32I(Auipc(u)) => self.reg_w(u.rd, pc.wrapping_add(sext_u32_u64(u.imm_u))),
            RV64I(Ld(i)) => self.reg_w(i.rd, 
                self.data_mem.read_u64(self.reg_r_off(i.rs1, i.imm_i as i64))?),
            _ => panic!("todo"),
        }
        Ok(())
    }

    pub(crate) fn dump_regs(&self) {
        println!("{:?}", self.regs);
    }

    fn reg_r(&self, index: u8) -> u64 {
        self.regs.regs[index as usize]
    }

    fn reg_r_off(&self, index: u8, off: i64) -> u64 {
        let v = self.regs.regs[index as usize];
        if off > 0 {
            v.wrapping_add(off as u64)
        } else { 
            v.wrapping_sub(off.abs() as u64)
        }
    }

    fn reg_w(&mut self, index: u8, data: u64) {
        self.regs.regs[index as usize] = data
    }
}

fn sext_u32_u64(i: u32) -> u64 {
    (i as u64) | if (i >> 31) != 0 { 0xFFFFFFFF00000000 } else { 0 }
}
