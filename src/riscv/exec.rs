
use crate::mem64::Physical;
use crate::error::Result;
use super::*;
use super::fetch::*;
use super::regfile::{XReg, Csr};

fn pc_to_mem_addr(pc: Usize) -> u64 {
    match pc {
        Usize::U32(a) => a as u64,
        Usize::U64(a) => a,
    }
}

pub struct Execute<'a> {
    data_mem: &'a mut Physical<'a>,
    x: Box<XReg>,
    csrs: Box<Csr>,
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
            csrs: Box::new(Csr { csr: [0u64; 4096] }),
            xlen
        }
    }

    pub fn execute(&mut self, ins: Instruction, pc: Usize, pc_nxt: &mut Usize) -> Result<()> {
        use {Instruction::*, self::RV32I::*, self::RV64I::*, self::RVZicsr::*};
        match ins {
            RV32I(Lui(u)) => self.x.w_zext32(u.rd, u.imm_u),
            RV32I(Auipc(u)) => self.x.w_usize(u.rd, pc + u.imm_u),
            RV32I(Jal(j)) => {
                let pc_link = *pc_nxt;
                *pc_nxt = pc + j.imm_j;
                self.x.w_usize(j.rd, pc_link);
            },
            RV32I(Jalr(i)) => {
                let pc_link = *pc_nxt;
                *pc_nxt = self.x.r_usize(i.rs1) + i.imm_i;
                self.x.w_usize(i.rd, pc_link);
            },
            RV32I(Beq(b)) => {
                if self.x.r_usize(b.rs1) == self.x.r_usize(b.rs2) {
                    *pc_nxt = pc + b.imm_b
                }
            },
            RV32I(Bne(b)) => {
                if self.x.r_usize(b.rs1) != self.x.r_usize(b.rs2) {
                    *pc_nxt = pc + b.imm_b
                }
            },
            RV32I(Blt(b)) => {
                if self.x.r_isize(b.rs1) < self.x.r_isize(b.rs2) {
                    *pc_nxt = pc + b.imm_b
                }
            },
            RV32I(Bge(b)) => {
                if self.x.r_isize(b.rs1) >= self.x.r_isize(b.rs2) {
                    *pc_nxt = pc + b.imm_b
                }
            },
            RV32I(Bltu(b)) => {
                if self.x.r_usize(b.rs1) < self.x.r_usize(b.rs2) {
                    *pc_nxt = pc + b.imm_b
                }
            },
            RV32I(Bgeu(b)) => {
                if self.x.r_usize(b.rs1) >= self.x.r_usize(b.rs2) {
                    *pc_nxt = pc + b.imm_b
                }
            },
            RV32I(Lb(i)) => {
                let addr = pc_to_mem_addr(self.x.r_usize(i.rs1) + i.imm_i);
                let data = self.data_mem.read_u8(addr)?;
                self.x.w_sext8(i.rd, data);
            },
            RV32I(Lh(i)) => {
                let addr = pc_to_mem_addr(self.x.r_usize(i.rs1) + i.imm_i);
                let data = self.data_mem.read_u16(addr)?;
                self.x.w_sext16(i.rd, data);
            },
            RV32I(Lw(i)) => {
                let addr = pc_to_mem_addr(self.x.r_usize(i.rs1) + i.imm_i);
                let data = self.data_mem.read_u32(addr)?;
                self.x.w_sext32(i.rd, data);
            },
            RV32I(Lbu(i)) => {
                let addr = pc_to_mem_addr(self.x.r_usize(i.rs1) + i.imm_i);
                let data = self.data_mem.read_u8(addr)?;
                self.x.w_zext8(i.rd, data);
            },
            RV32I(Lhu(i)) => {
                let addr = pc_to_mem_addr(self.x.r_usize(i.rs1) + i.imm_i);
                let data = self.data_mem.read_u16(addr)?;
                self.x.w_zext16(i.rd, data);
            },
            RV32I(Sb(s)) => self.data_mem.write_u8(
                pc_to_mem_addr(self.x.r_usize(s.rs1) + s.imm_s),
                self.x.r_low8(s.rs2)
            )?,
            RV32I(Sh(s)) => self.data_mem.write_u16(
                pc_to_mem_addr(self.x.r_usize(s.rs1) + s.imm_s),
                self.x.r_low16(s.rs2)
            )?,
            RV32I(Sw(s)) => self.data_mem.write_u32(
                pc_to_mem_addr(self.x.r_usize(s.rs1) + s.imm_s),
                self.x.r_low32(s.rs2)
            )?,
            RV32I(Addi(i)) => 
                self.x.w_usize(i.rd, self.x.r_usize(i.rs1) + i.imm_i),
            RV32I(Slti(i)) => {
                let value = if self.x.r_isize(i.rs1) < i.imm_i { 1 } else { 0 };
                self.x.w_zext8(i.rd, value);
            },
            RV32I(Sltiu(i)) => {
                let imm = u32::from_ne_bytes(i32::to_ne_bytes(i.imm_i));
                let value = if self.x.r_usize(i.rs1) < imm { 1 } else { 0 };
                self.x.w_zext8(i.rd, value);
            },
            // RV32I(Ori(i)) => {
            //     let imm = u64::from_ne_bytes(i64::to_ne_bytes(i.imm_i as i64));
            //     self.xw(i.rd, self.xr(i.rs1) | imm);
            // },
            // RV32I(Andi(i)) => {
            //     let imm = u64::from_ne_bytes(i64::to_ne_bytes(i.imm_i as i64));
            //     self.reg_w(i.rd, self.reg_r(i.rs1) & imm);
            // },
            // RV32I(Xori(i)) => {
            //     let imm = u64::from_ne_bytes(i64::to_ne_bytes(i.imm_i as i64));
            //     self.reg_w(i.rd, self.reg_r(i.rs1) ^ imm);
            // },
            // RV32I(self::RV32I::Slli(i)) => {
            //     let shamt = shamt_from_imm_xlen32(i.imm_i);
            //     self.reg_w(i.rd, self.reg_r(i.rs1) << shamt);
            // },
            // RV32I(self::RV32I::Srli(i)) => {
            //     let shamt = shamt_from_imm_xlen32(i.imm_i);
            //     self.reg_w(i.rd, self.reg_r(i.rs1) >> shamt);
            // },
            // RV32I(self::RV32I::Srai(i)) => {
            //     let shamt = shamt_from_imm_xlen32(i.imm_i);
            //     let sra = self.reg_r_i64(i.rs1) >> shamt;
            //     self.reg_w(i.rd, u64::from_ne_bytes(i64::to_ne_bytes(sra)));
            // },
            // RV32I(Add(r)) => self.reg_w(r.rd, 
            //     self.reg_r(r.rs1).wrapping_add(self.reg_r(r.rs2))),
            // RV32I(Sub(r)) => self.reg_w(r.rd, 
            //     self.reg_r(r.rs1).wrapping_sub(self.reg_r(r.rs2))),
            // RV32I(self::RV32I::Sll(r)) => {
            //     let shamt = shamt_from_reg_xlen32(self.reg_r(r.rs2));
            //     self.reg_w(r.rd, self.reg_r(r.rs1) << shamt);
            // },
            // RV32I(Slt(r)) => {
            //     let value = if self.reg_r_i64(r.rs1) < self.reg_r_i64(r.rs2)
            //         { 1 } else { 0 };
            //     self.reg_w(r.rd, value);
            // },
            // RV32I(Sltu(r)) => {
            //     let value = if self.reg_r(r.rs1) < self.reg_r(r.rs2) 
            //         { 1 } else { 0 };
            //     self.reg_w(r.rd, value);
            // },
            // RV32I(Xor(r)) => {
            //     self.reg_w(r.rd, self.reg_r(r.rs1) ^ self.reg_r(r.rs2));
            // },
            // RV32I(self::RV32I::Srl(r)) => { 
            //     let shamt = shamt_from_reg_xlen32(self.reg_r(r.rs2));
            //     self.reg_w(r.rd, self.reg_r(r.rs1) >> shamt);
            // },
            // RV32I(self::RV32I::Sra(r)) => {
            //     let shamt = shamt_from_reg_xlen32(self.reg_r(r.rs2));
            //     let sra = self.reg_r_i64(r.rs1) >> shamt;
            //     self.reg_w(r.rd, u64::from_ne_bytes(i64::to_ne_bytes(sra)));
            // },
            // RV32I(Or(r)) => {
            //     self.reg_w(r.rd, self.reg_r(r.rs1) | self.reg_r(r.rs2));
            // },
            // RV32I(And(r)) => {
            //     self.reg_w(r.rd, self.reg_r(r.rs1) & self.reg_r(r.rs2));
            // },
            // RV64I(Ld(i)) => self.reg_w(i.rd, 
            //     self.data_mem.read_u64(u64_add_i32(self.reg_r(i.rs1), i.imm_i))?),
            // RV64I(Sd(s)) => self.data_mem.write_u64(
            //     u64_add_i32(self.reg_r(s.rs1), s.imm_s),
            //     self.reg_r(s.rs2)
            // )?,
            // RV64I(self::RV64I::Slli(i)) => {
            //     let shamt = shamt_from_imm_xlen64(i.imm_i);
            //     self.reg_w(i.rd, self.reg_r(i.rs1) << shamt);
            // },
            // RV64I(self::RV64I::Srli(i)) => {
            //     let shamt = shamt_from_imm_xlen64(i.imm_i);
            //     self.reg_w(i.rd, self.reg_r(i.rs1) >> shamt);
            // },
            // RV64I(self::RV64I::Srai(i)) => {
            //     let shamt = shamt_from_imm_xlen64(i.imm_i);
            //     let sra = self.reg_r_i64(i.rs1) >> shamt;
            //     self.reg_w(i.rd, u64::from_ne_bytes(i64::to_ne_bytes(sra)));
            // },
            // RV64I(self::RV64I::Sll(r)) => {
            //     let shamt = shamt_from_reg_xlen64(self.reg_r(r.rs2));
            //     self.reg_w(r.rd, self.reg_r(r.rs1) << shamt);
            // },
            // RV64I(self::RV64I::Srl(r)) => {
            //     let shamt = shamt_from_reg_xlen64(self.reg_r(r.rs2));
            //     self.reg_w(r.rd, self.reg_r(r.rs1) >> shamt);
            // },
            // RV64I(self::RV64I::Sra(r)) => {
            //     let shamt = shamt_from_reg_xlen64(self.reg_r(r.rs2));
            //     let sra = self.reg_r_i64(r.rs1) >> shamt;
            //     self.reg_w(r.rd, u64::from_ne_bytes(i64::to_ne_bytes(sra)));
            // },
            // // side effect?
            // RVZicsr(Csrrw(csr)) => {
            //     self.reg_w(csr.rd, self.csr_r(csr.csr));
            //     self.csr_w(csr.csr, self.reg_r(csr.rs1uimm));
            // },
            // RVZicsr(Csrrs(csr)) => {
            //     self.reg_w(csr.rd, self.csr_r(csr.csr));
            //     self.csr_w(csr.csr, self.csr_r(csr.csr) | self.reg_r(csr.rs1uimm));
            // },
            // RVZicsr(Csrrc(csr)) => {
            //     self.reg_w(csr.rd, self.csr_r(csr.csr));
            //     self.csr_w(csr.csr, self.csr_r(csr.csr) & !self.reg_r(csr.rs1uimm));
            // },
            // RVZicsr(Csrrwi(csr)) => {
            //     self.reg_w(csr.rd, self.csr_r(csr.csr));
            //     self.csr_w(csr.csr, csr.rs1uimm as u64);
            // },
            // RVZicsr(Csrrsi(csr)) => {
            //     self.reg_w(csr.rd, self.csr_r(csr.csr));
            //     self.csr_w(csr.csr, self.csr_r(csr.csr) | (csr.rs1uimm as u64));
            // },
            // RVZicsr(Csrrci(csr)) => {
            //     self.reg_w(csr.rd, self.csr_r(csr.csr));
            //     self.csr_w(csr.csr, self.csr_r(csr.csr) & !(csr.rs1uimm as u64));
            // },
            _ => panic!("todo"),
        }
        Ok(())
    }

    pub(crate) fn dump_regs(&self) {
        println!("{:?}", self.x);
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
