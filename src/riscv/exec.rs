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

    #[rustfmt::skip]
    pub fn execute(&mut self, ins: Instruction, pc: Usize, pc_nxt: &mut Usize) -> Result<()> {
        let xlen = self.xlen;
        match ins {
            Instruction::RV32I(ins) => exec_rv32i(
                ins,
                &mut self.x,
                &mut self.data_mem,
                pc,
                pc_nxt,
                |imm| imm.sext(xlen),
            )?,
            Instruction::RV64I(ins) => exec_rv64i(
                ins,
                &mut self.x,
                &mut self.data_mem,
                |imm| imm.sext(xlen)
            )?,
            Instruction::RVZicsr(ins) => exec_rvzicsr(
                ins,
                &mut self.x,
                &mut self.csr,
                |uimm| uimm.zext(xlen)
            )?,
            Instruction::RVC(_ins) => todo!(),
        }
        Ok(())
    }
}

fn shamt32(imm: Imm) -> u32 {
    imm.low32() & 0b11111
}

fn shamt32r(data: Usize) -> u32 {
    data.low32() & 0b11111
}

fn exec_rv32i<'a, SEXT: Fn(Imm) -> Isize>(
    ins: RV32I,
    x: &mut XReg,
    data_mem: &mut Physical<'a>,
    pc: Usize,
    pc_nxt: &mut Usize,
    sext: SEXT,
) -> Result<()> {
    use RV32I::*;
    match ins {
        Lui(u) => x.w_isize(u.rd, sext(u.imm)),
        Auipc(u) => x.w_usize(u.rd, pc + sext(u.imm)),
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
            let value = if x.r_usize(i.rs1) < sext(i.imm).cast_to_usize() {
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

fn shamt64(imm: Imm) -> u32 {
    imm.low32() & 0b111111
}

fn shamt64r(data: Usize) -> u32 {
    data.low32() & 0b111111
}

fn exec_rv64i<'a, SEXT: Fn(Imm) -> Isize>(
    ins: RV64I,
    x: &mut XReg,
    data_mem: &mut Physical<'a>,
    sext: SEXT,
) -> Result<()> {
    use RV64I::*;
    match ins {
        Lwu(i) => {
            let addr = pc_to_mem_addr(x.r_usize(i.rs1) + sext(i.imm));
            let data = data_mem.read_u32(addr)?;
            x.w_zext32(i.rd, data);
        }
        Ld(i) => {
            let addr = pc_to_mem_addr(x.r_usize(i.rs1) + sext(i.imm));
            let data = data_mem.read_u64(addr)?;
            x.w_sext64(i.rd, data);
        }
        Sd(s) => data_mem.write_u64(
            pc_to_mem_addr(x.r_usize(s.rs1) + sext(s.imm)),
            x.r_low64(s.rs2),
        )?,
        Slli(i) => x.w_usize(i.rd, x.r_usize(i.rs1) << shamt64(i.imm)),
        Srli(i) => x.w_usize(i.rd, x.r_usize(i.rs1) >> shamt64(i.imm)),
        Srai(i) => x.w_isize(i.rd, x.r_isize(i.rs1) >> shamt64(i.imm)),
        Sll(r) => {
            let shamt = shamt64r(x.r_usize(r.rs2));
            x.w_usize(r.rd, x.r_usize(r.rs1) << shamt);
        }
        Srl(r) => {
            let shamt = shamt64r(x.r_usize(r.rs2));
            x.w_usize(r.rd, x.r_usize(r.rs1) >> shamt);
        }
        Sra(r) => {
            let shamt = shamt64r(x.r_usize(r.rs2));
            x.w_isize(r.rd, x.r_isize(r.rs1) >> shamt);
        }
        Addiw(_i) => todo!(),
        Slliw(_i) => todo!(),
        Srliw(_i) => todo!(),
        Sraiw(_i) => todo!(),
        Addw(_r) => todo!(),
        Subw(_r) => todo!(),
        Sllw(_r) => todo!(),
        Srlw(_r) => todo!(),
        Sraw(_r) => todo!(),
    }
    Ok(())
}

fn exec_rvzicsr<'a, ZEXT: Fn(Uimm) -> Usize>(
    ins: RVZicsr,
    x: &mut XReg,
    csr: &mut Csr,
    zext: ZEXT,
) -> Result<()> {
    use RVZicsr::*;
    // if r.rd!=0 or r.rs1 != 0 => prevent side effects
    match ins {
        Csrrw(r) => {
            if r.rd != 0 {
                x.w_usize(r.rd, csr.r_usize(r.csr));
            }
            csr.w_usize(r.csr, x.r_usize(r.rs1));
        }
        Csrrs(r) => {
            x.w_usize(r.rd, csr.r_usize(r.csr));
            if r.rs1 != 0 {
                csr.w_usize(r.csr, csr.r_usize(r.csr) | x.r_usize(r.rs1));
            }
        }
        Csrrc(r) => {
            x.w_usize(r.rd, csr.r_usize(r.csr));
            if r.rs1 != 0 {
                csr.w_usize(r.csr, csr.r_usize(r.csr) & !x.r_usize(r.rs1));
            }
        }
        Csrrwi(i) => {
            if i.rd != 0 {
                x.w_usize(i.rd, csr.r_usize(i.csr));
            }
            csr.w_usize(i.csr, zext(i.uimm));
        }
        Csrrsi(i) => {
            x.w_usize(i.rd, csr.r_usize(i.csr));
            if i.uimm != 0 {
                csr.w_usize(i.csr, csr.r_usize(i.csr) | zext(i.uimm));
            }
        }
        Csrrci(i) => {
            x.w_usize(i.rd, csr.r_usize(i.csr));
            if i.uimm != 0 {
                csr.w_usize(i.csr, csr.r_usize(i.csr) & !zext(i.uimm));
            }
        }
    }
    Ok(())
}
