
use crate::mem64::Physical;
use crate::error::Result;
use super::*;
use super::fetch::*;
use super::regfile::{XReg, Csr};
use super::imm::{Uimm, Imm};
use crate::size::{Isize, Usize};

fn pc_to_mem_addr(pc: Usize) -> u64 {
    match pc {
        Usize::U32(a) => a as u64,
        Usize::U64(a) => a,
    }
}

pub struct Execute<'a> {
    data_mem: &'a mut Physical<'a>,
    x: Box<XReg>,
    csr: Box<Csr>,
    xlen: Xlen,
}

impl<'a> core::fmt::Debug for Execute<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Execute")
            .field("x", &self.x)
            .field("xlen", &self.xlen)
            .finish()
    }
}

impl<'a> Execute<'a> {
    pub fn new(data_mem: &'a mut Physical<'a>, xlen: Xlen) -> Execute<'a> {
        Execute { 
            data_mem,
            x: Box::new(XReg::new_zeroed(xlen)),
            csr: Box::new(Csr::new(xlen)),
            xlen
        }
    }

    fn zext(&self, imm: Uimm) -> Usize {
        imm.zext(self.xlen)
    }

    fn sext(&self, imm: Imm) -> Isize {
        imm.sext(self.xlen)
    }

    pub fn execute(&mut self, ins: Instruction, pc: Usize, pc_nxt: &mut Usize) -> Result<()> {
        use {Instruction::*, self::RV32I::*, self::RV64I::*, self::RVZicsr::*};
        match ins {
            RV32I(Lui(u)) => self.x.w_usize(u.rd, self.zext(u.imm)),
            RV32I(Auipc(u)) => self.x.w_usize(u.rd, pc + self.zext(u.imm)),
            RV32I(Jal(j)) => {
                let pc_link = *pc_nxt;
                *pc_nxt = pc + self.sext(j.imm);
                self.x.w_usize(j.rd, pc_link);
            },
            RV32I(Jalr(i)) => {
                let pc_link = *pc_nxt;
                *pc_nxt = self.x.r_usize(i.rs1) + self.sext(i.imm);
                self.x.w_usize(i.rd, pc_link);
            },
            RV32I(Beq(b)) => {
                if self.x.r_usize(b.rs1) == self.x.r_usize(b.rs2) {
                    *pc_nxt = pc + self.sext(b.imm)
                }
            },
            RV32I(Bne(b)) => {
                if self.x.r_usize(b.rs1) != self.x.r_usize(b.rs2) {
                    *pc_nxt = pc + self.sext(b.imm)
                }
            },
            RV32I(Blt(b)) => {
                if self.x.r_isize(b.rs1) < self.x.r_isize(b.rs2) {
                    *pc_nxt = pc + self.sext(b.imm)
                }
            },
            RV32I(Bge(b)) => {
                if self.x.r_isize(b.rs1) >= self.x.r_isize(b.rs2) {
                    *pc_nxt = pc + self.sext(b.imm)
                }
            },
            RV32I(Bltu(b)) => {
                if self.x.r_usize(b.rs1) < self.x.r_usize(b.rs2) {
                    *pc_nxt = pc + self.sext(b.imm)
                }
            },
            RV32I(Bgeu(b)) => {
                if self.x.r_usize(b.rs1) >= self.x.r_usize(b.rs2) {
                    *pc_nxt = pc + self.sext(b.imm)
                }
            },
            RV32I(Lb(i)) => {
                let addr = pc_to_mem_addr(self.x.r_usize(i.rs1) + self.sext(i.imm));
                let data = self.data_mem.read_u8(addr)?;
                self.x.w_sext8(i.rd, data);
            },
            RV32I(Lh(i)) => {
                let addr = pc_to_mem_addr(self.x.r_usize(i.rs1) + self.sext(i.imm));
                let data = self.data_mem.read_u16(addr)?;
                self.x.w_sext16(i.rd, data);
            },
            RV32I(Lw(i)) => {
                let addr = pc_to_mem_addr(self.x.r_usize(i.rs1) + self.sext(i.imm));
                let data = self.data_mem.read_u32(addr)?;
                self.x.w_sext32(i.rd, data);
            },
            RV32I(Lbu(i)) => {
                let addr = pc_to_mem_addr(self.x.r_usize(i.rs1) + self.sext(i.imm));
                let data = self.data_mem.read_u8(addr)?;
                self.x.w_zext8(i.rd, data);
            },
            RV32I(Lhu(i)) => {
                let addr = pc_to_mem_addr(self.x.r_usize(i.rs1) + self.sext(i.imm));
                let data = self.data_mem.read_u16(addr)?;
                self.x.w_zext16(i.rd, data);
            },
            RV32I(Sb(s)) => self.data_mem.write_u8(
                pc_to_mem_addr(self.x.r_usize(s.rs1) + self.sext(s.imm)),
                self.x.r_low8(s.rs2)
            )?,
            RV32I(Sh(s)) => self.data_mem.write_u16(
                pc_to_mem_addr(self.x.r_usize(s.rs1) + self.sext(s.imm)),
                self.x.r_low16(s.rs2)
            )?,
            RV32I(Sw(s)) => self.data_mem.write_u32(
                pc_to_mem_addr(self.x.r_usize(s.rs1) + self.sext(s.imm)),
                self.x.r_low32(s.rs2)
            )?,
            RV32I(Addi(i)) => 
                self.x.w_usize(i.rd, self.x.r_usize(i.rs1) + self.sext(i.imm)),
            RV32I(Slti(i)) => {
                let value = if self.x.r_isize(i.rs1) < self.sext(i.imm) { 1 } else { 0 };
                self.x.w_zext8(i.rd, value);
            },
            RV32I(Sltiu(i)) => {
                let value = if self.x.r_usize(i.rs1) < self.zext(i.imm.as_uimm()) { 1 } else { 0 };
                self.x.w_zext8(i.rd, value);
            },
            RV32I(Ori(i)) => {
                self.x.w_usize(i.rd, self.x.r_usize(i.rs1) | self.sext(i.imm));
            },
            RV32I(Andi(i)) => {
                self.x.w_usize(i.rd, self.x.r_usize(i.rs1) & self.sext(i.imm));
            },
            RV32I(Xori(i)) => {
                self.x.w_usize(i.rd, self.x.r_usize(i.rs1) ^ self.sext(i.imm));
            },
            RV32I(self::RV32I::Slli(i)) => {
                self.x.w_usize(i.rd, self.x.r_usize(i.rs1) << shamt32(i.imm));
            },
            RV32I(self::RV32I::Srli(i)) => {
                self.x.w_usize(i.rd, self.x.r_usize(i.rs1) >> shamt32(i.imm));
            },
            RV32I(self::RV32I::Srai(i)) => {
                self.x.w_isize(i.rd, self.x.r_isize(i.rs1) >> shamt32(i.imm));
            },
            RV32I(Add(r)) => self.x.w_usize(r.rd, 
                self.x.r_usize(r.rs1) + self.x.r_usize(r.rs2)),
            RV32I(Sub(r)) => self.x.w_usize(r.rd, 
                self.x.r_usize(r.rs1) - self.x.r_usize(r.rs2)),
            RV32I(self::RV32I::Sll(r)) => {
                let shamt = shamt32r(self.x.r_usize(r.rs2));
                self.x.w_usize(r.rd, self.x.r_usize(r.rs1) << shamt);
            },
            RV32I(Slt(r)) => {
                let value = if self.x.r_isize(r.rs1) < self.x.r_isize(r.rs2)
                    { 1 } else { 0 };
                self.x.w_sext8(r.rd, value);
            },
            RV32I(Sltu(r)) => {
                let value = if self.x.r_usize(r.rs1) < self.x.r_usize(r.rs2) 
                    { 1 } else { 0 };
                self.x.w_sext8(r.rd, value);
            },
            RV32I(Xor(r)) => {
                self.x.w_usize(r.rd, self.x.r_usize(r.rs1) ^ self.x.r_usize(r.rs2));
            },
            RV32I(self::RV32I::Srl(r)) => { 
                let shamt = shamt32r(self.x.r_usize(r.rs2));
                self.x.w_usize(r.rd, self.x.r_usize(r.rs1) >> shamt);
            },
            RV32I(self::RV32I::Sra(r)) => {
                let shamt = shamt32r(self.x.r_usize(r.rs2));
                self.x.w_isize(r.rd, self.x.r_isize(r.rs1) >> shamt);
            },
            RV32I(Or(r)) => {
                self.x.w_usize(r.rd, self.x.r_usize(r.rs1) | self.x.r_usize(r.rs2));
            },
            RV32I(And(r)) => {
                self.x.w_usize(r.rd, self.x.r_usize(r.rs1) & self.x.r_usize(r.rs2));
            },
            RV64I(Lwu(i)) => {
                let addr = pc_to_mem_addr(self.x.r_usize(i.rs1) + self.sext(i.imm));
                let data = self.data_mem.read_u32(addr)?;
                self.x.w_zext32(i.rd, data);
            },
            RV64I(Ld(i)) => {
                let addr = pc_to_mem_addr(self.x.r_usize(i.rs1) + self.sext(i.imm));
                let data = self.data_mem.read_u64(addr)?;
                self.x.w_sext64(i.rd, data);
            },
            RV64I(Sd(s)) => self.data_mem.write_u64(
                pc_to_mem_addr(self.x.r_usize(s.rs1) + self.sext(s.imm)),
                self.x.r_low64(s.rs2)
            )?,
            RV64I(self::RV64I::Slli(i)) => {
                self.x.w_usize(i.rd, self.x.r_usize(i.rs1) << shamt64(i.imm));
            },
            RV64I(self::RV64I::Srli(i)) => {
                self.x.w_usize(i.rd, self.x.r_usize(i.rs1) >> shamt64(i.imm));
            },
            RV64I(self::RV64I::Srai(i)) => {
                self.x.w_isize(i.rd, self.x.r_isize(i.rs1) >> shamt64(i.imm));
            },
            RV64I(self::RV64I::Sll(r)) => {
                let shamt = shamt64r(self.x.r_usize(r.rs2));
                self.x.w_usize(r.rd, self.x.r_usize(r.rs1) << shamt);
            },
            RV64I(self::RV64I::Srl(r)) => {
                let shamt = shamt64r(self.x.r_usize(r.rs2));
                self.x.w_usize(r.rd, self.x.r_usize(r.rs1) >> shamt);
            },
            RV64I(self::RV64I::Sra(r)) => {
                let shamt = shamt64r(self.x.r_usize(r.rs2));
                self.x.w_isize(r.rd, self.x.r_isize(r.rs1) >> shamt);
            },
            RVZicsr(Csrrw(csr)) => {
                self.x.w_usize(csr.rd, self.csr.r_usize(csr.csr));
                self.csr.w_usize(csr.csr, self.x.r_usize(csr.rs1));
            },
            RVZicsr(Csrrs(csr)) => {
                self.x.w_usize(csr.rd, self.csr.r_usize(csr.csr));
                self.csr.w_usize(csr.csr, self.csr.r_usize(csr.csr) | self.x.r_usize(csr.rs1));
            },
            RVZicsr(Csrrc(csr)) => {
                self.x.w_usize(csr.rd, self.csr.r_usize(csr.csr));
                self.csr.w_usize(csr.csr, self.csr.r_usize(csr.csr) & !self.x.r_usize(csr.rs1));
            },
            RVZicsr(Csrrwi(csr)) => {
                self.x.w_usize(csr.rd, self.csr.r_usize(csr.csr));
                self.csr.w_usize(csr.csr, self.zext(csr.uimm));
            },
            RVZicsr(Csrrsi(csr)) => {
                self.x.w_usize(csr.rd, self.csr.r_usize(csr.csr));
                self.csr.w_usize(csr.csr, self.csr.r_usize(csr.csr) | self.zext(csr.uimm));
            },
            RVZicsr(Csrrci(csr)) => {
                self.x.w_usize(csr.rd, self.csr.r_usize(csr.csr));
                self.csr.w_usize(csr.csr, self.csr.r_usize(csr.csr) & !self.zext(csr.uimm));
            },
            _ => panic!("todo"),
        }
        Ok(())
    }

    pub(crate) fn dump_regs(&self) {
        println!("{:?}", self.x);
    }
}

fn shamt32(imm: Imm) -> u32 {
    imm.low32() & 0b11111
}

fn shamt32r(data: Usize) -> u32 {
    data.low32() & 0b11111
}

fn shamt64(imm: Imm) -> u32 {
    imm.low32() & 0b111111
}

fn shamt64r(data: Usize) -> u32 {
    data.low32() & 0b111111
}
