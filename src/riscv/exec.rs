use super::fetch::*;
use super::imm::{Imm, Uimm};
use super::regfile::{Csr, XReg};
use super::*;
use crate::error::Result;
use crate::mem64::Physical;
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
            xlen,
        }
    }

    fn zext(&self, imm: Uimm) -> Usize {
        imm.zext(self.xlen)
    }

    fn sext(&self, imm: Imm) -> Isize {
        imm.sext(self.xlen)
    }

    pub fn execute(&mut self, ins: Instruction, pc: Usize, pc_nxt: &mut Usize) -> Result<()> {
        use {self::RVZicsr::*, self::RV64I::*, Instruction::*};
        let xlen = self.xlen;
        match ins {
            RV32I(ins) => exec_rv32i(
                ins,
                &mut self.x,
                &mut self.data_mem,
                pc,
                pc_nxt,
                |uimm| uimm.zext(xlen),
                |imm| imm.sext(xlen),
            )?,
            RV64I(Lwu(i)) => {
                let addr = pc_to_mem_addr(self.x.r_usize(i.rs1) + self.sext(i.imm));
                let data = self.data_mem.read_u32(addr)?;
                self.x.w_zext32(i.rd, data);
            }
            RV64I(Ld(i)) => {
                let addr = pc_to_mem_addr(self.x.r_usize(i.rs1) + self.sext(i.imm));
                let data = self.data_mem.read_u64(addr)?;
                self.x.w_sext64(i.rd, data);
            }
            RV64I(Sd(s)) => self.data_mem.write_u64(
                pc_to_mem_addr(self.x.r_usize(s.rs1) + self.sext(s.imm)),
                self.x.r_low64(s.rs2),
            )?,
            RV64I(self::RV64I::Slli(i)) => {
                self.x
                    .w_usize(i.rd, self.x.r_usize(i.rs1) << shamt64(i.imm));
            }
            RV64I(self::RV64I::Srli(i)) => {
                self.x
                    .w_usize(i.rd, self.x.r_usize(i.rs1) >> shamt64(i.imm));
            }
            RV64I(self::RV64I::Srai(i)) => {
                self.x
                    .w_isize(i.rd, self.x.r_isize(i.rs1) >> shamt64(i.imm));
            }
            RV64I(self::RV64I::Sll(r)) => {
                let shamt = shamt64r(self.x.r_usize(r.rs2));
                self.x.w_usize(r.rd, self.x.r_usize(r.rs1) << shamt);
            }
            RV64I(self::RV64I::Srl(r)) => {
                let shamt = shamt64r(self.x.r_usize(r.rs2));
                self.x.w_usize(r.rd, self.x.r_usize(r.rs1) >> shamt);
            }
            RV64I(self::RV64I::Sra(r)) => {
                let shamt = shamt64r(self.x.r_usize(r.rs2));
                self.x.w_isize(r.rd, self.x.r_isize(r.rs1) >> shamt);
            }
            RVZicsr(Csrrw(csr)) => {
                self.x.w_usize(csr.rd, self.csr.r_usize(csr.csr));
                self.csr.w_usize(csr.csr, self.x.r_usize(csr.rs1));
            }
            RVZicsr(Csrrs(csr)) => {
                self.x.w_usize(csr.rd, self.csr.r_usize(csr.csr));
                self.csr
                    .w_usize(csr.csr, self.csr.r_usize(csr.csr) | self.x.r_usize(csr.rs1));
            }
            RVZicsr(Csrrc(csr)) => {
                self.x.w_usize(csr.rd, self.csr.r_usize(csr.csr));
                self.csr.w_usize(
                    csr.csr,
                    self.csr.r_usize(csr.csr) & !self.x.r_usize(csr.rs1),
                );
            }
            RVZicsr(Csrrwi(csr)) => {
                self.x.w_usize(csr.rd, self.csr.r_usize(csr.csr));
                self.csr.w_usize(csr.csr, self.zext(csr.uimm));
            }
            RVZicsr(Csrrsi(csr)) => {
                self.x.w_usize(csr.rd, self.csr.r_usize(csr.csr));
                self.csr
                    .w_usize(csr.csr, self.csr.r_usize(csr.csr) | self.zext(csr.uimm));
            }
            RVZicsr(Csrrci(csr)) => {
                self.x.w_usize(csr.rd, self.csr.r_usize(csr.csr));
                self.csr
                    .w_usize(csr.csr, self.csr.r_usize(csr.csr) & !self.zext(csr.uimm));
            }
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

fn exec_rv32i<'a, ZEXT: Fn(Uimm) -> Usize, SEXT: Fn(Imm) -> Isize>(
    ins: RV32I,
    x: &mut XReg,
    data_mem: &mut Physical<'a>,
    pc: Usize,
    pc_nxt: &mut Usize,
    zext: ZEXT,
    sext: SEXT,
) -> Result<()> {
    use RV32I::*;
    match ins {
        Lui(u) => x.w_usize(u.rd, zext(u.imm)),
        Auipc(u) => x.w_usize(u.rd, pc + zext(u.imm)),
        Jal(j) => {
            let pc_link = *pc_nxt;
            *pc_nxt = pc + sext(j.imm);
            x.w_usize(j.rd, pc_link);
        }
        Jalr(i) => {
            let pc_link = *pc_nxt;
            *pc_nxt = x.r_usize(i.rs1) + sext(i.imm);
            x.w_usize(i.rd, pc_link);
        }
        Beq(b) => {
            if x.r_usize(b.rs1) == x.r_usize(b.rs2) {
                *pc_nxt = pc + sext(b.imm)
            }
        }
        Bne(b) => {
            if x.r_usize(b.rs1) != x.r_usize(b.rs2) {
                *pc_nxt = pc + sext(b.imm)
            }
        }
        Blt(b) => {
            if x.r_isize(b.rs1) < x.r_isize(b.rs2) {
                *pc_nxt = pc + sext(b.imm)
            }
        }
        Bge(b) => {
            if x.r_isize(b.rs1) >= x.r_isize(b.rs2) {
                *pc_nxt = pc + sext(b.imm)
            }
        }
        Bltu(b) => {
            if x.r_usize(b.rs1) < x.r_usize(b.rs2) {
                *pc_nxt = pc + sext(b.imm)
            }
        }
        Bgeu(b) => {
            if x.r_usize(b.rs1) >= x.r_usize(b.rs2) {
                *pc_nxt = pc + sext(b.imm)
            }
        }
        Lb(i) => {
            let addr = pc_to_mem_addr(x.r_usize(i.rs1) + sext(i.imm));
            let data = data_mem.read_u8(addr)?;
            x.w_sext8(i.rd, data);
        }
        Lh(i) => {
            let addr = pc_to_mem_addr(x.r_usize(i.rs1) + sext(i.imm));
            let data = data_mem.read_u16(addr)?;
            x.w_sext16(i.rd, data);
        }
        Lw(i) => {
            let addr = pc_to_mem_addr(x.r_usize(i.rs1) + sext(i.imm));
            let data = data_mem.read_u32(addr)?;
            x.w_sext32(i.rd, data);
        }
        Lbu(i) => {
            let addr = pc_to_mem_addr(x.r_usize(i.rs1) + sext(i.imm));
            let data = data_mem.read_u8(addr)?;
            x.w_zext8(i.rd, data);
        }
        Lhu(i) => {
            let addr = pc_to_mem_addr(x.r_usize(i.rs1) + sext(i.imm));
            let data = data_mem.read_u16(addr)?;
            x.w_zext16(i.rd, data);
        }
        Sb(s) => data_mem.write_u8(
            pc_to_mem_addr(x.r_usize(s.rs1) + sext(s.imm)),
            x.r_low8(s.rs2),
        )?,
        Sh(s) => data_mem.write_u16(
            pc_to_mem_addr(x.r_usize(s.rs1) + sext(s.imm)),
            x.r_low16(s.rs2),
        )?,
        Sw(s) => data_mem.write_u32(
            pc_to_mem_addr(x.r_usize(s.rs1) + sext(s.imm)),
            x.r_low32(s.rs2),
        )?,
        Addi(i) => x.w_usize(i.rd, x.r_usize(i.rs1) + sext(i.imm)),
        Slti(i) => {
            let value = if x.r_isize(i.rs1) < sext(i.imm) { 1 } else { 0 };
            x.w_zext8(i.rd, value);
        }
        Sltiu(i) => {
            let value = if x.r_usize(i.rs1) < zext(i.imm.as_uimm()) {
                1
            } else {
                0
            };
            x.w_zext8(i.rd, value);
        }
        Ori(i) => {
            x.w_usize(i.rd, x.r_usize(i.rs1) | sext(i.imm));
        }
        Andi(i) => {
            x.w_usize(i.rd, x.r_usize(i.rs1) & sext(i.imm));
        }
        Xori(i) => {
            x.w_usize(i.rd, x.r_usize(i.rs1) ^ sext(i.imm));
        }
        Slli(i) => {
            x.w_usize(i.rd, x.r_usize(i.rs1) << shamt32(i.imm));
        }
        Srli(i) => {
            x.w_usize(i.rd, x.r_usize(i.rs1) >> shamt32(i.imm));
        }
        Srai(i) => {
            x.w_isize(i.rd, x.r_isize(i.rs1) >> shamt32(i.imm));
        }
        Add(r) => x.w_usize(r.rd, x.r_usize(r.rs1) + x.r_usize(r.rs2)),
        Sub(r) => x.w_usize(r.rd, x.r_usize(r.rs1) - x.r_usize(r.rs2)),
        Sll(r) => {
            let shamt = shamt32r(x.r_usize(r.rs2));
            x.w_usize(r.rd, x.r_usize(r.rs1) << shamt);
        }
        Slt(r) => {
            let value = if x.r_isize(r.rs1) < x.r_isize(r.rs2) {
                1
            } else {
                0
            };
            x.w_sext8(r.rd, value);
        }
        Sltu(r) => {
            let value = if x.r_usize(r.rs1) < x.r_usize(r.rs2) {
                1
            } else {
                0
            };
            x.w_sext8(r.rd, value);
        }
        Xor(r) => {
            x.w_usize(r.rd, x.r_usize(r.rs1) ^ x.r_usize(r.rs2));
        }
        Srl(r) => {
            let shamt = shamt32r(x.r_usize(r.rs2));
            x.w_usize(r.rd, x.r_usize(r.rs1) >> shamt);
        }
        Sra(r) => {
            let shamt = shamt32r(x.r_usize(r.rs2));
            x.w_isize(r.rd, x.r_isize(r.rs1) >> shamt);
        }
        Or(r) => {
            x.w_usize(r.rd, x.r_usize(r.rs1) | x.r_usize(r.rs2));
        }
        And(r) => {
            x.w_usize(r.rd, x.r_usize(r.rs1) & x.r_usize(r.rs2));
        }
        Fence(_) => todo!(),
        Ecall(_) => todo!(),
        Ebreak(_) => todo!(),
    }
    Ok(())
}
