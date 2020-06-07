use super::fetch::*;
use super::imm::{Imm, Uimm};
use super::regfile::{Csr, XReg, FReg};
use super::*;
use crate::error::Result;
use crate::mem64::Physical;
use crate::size::{Isize, Usize};
use thiserror::Error;

#[derive(Error, Clone, Debug)]
pub enum ExecError {
    #[error("extension is not supported")]
    ExtensionNotSupported,
}

fn pc_to_mem_addr(pc: Usize) -> u64 {
    match pc {
        Usize::U32(a) => a as u64,
        Usize::U64(a) => a,
    }
}

pub struct Execute<'a> {
    data_mem: &'a mut Physical<'a>,
    x: Box<XReg>,
    f: Box<FReg>,
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
            f: Box::new(FReg::new_zeroed()),
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
                |imm| imm.sext(xlen),
                || xlen == Xlen::X64 || xlen == Xlen::X128,
            )?,
            Instruction::RVZicsr(ins) => exec_rvzicsr(
                ins,
                &mut self.x,
                &mut self.csr,
                |uimm| uimm.zext(xlen)
            )?,
            Instruction::RVC(ins) => exec_rvc(
                ins, 
                &mut self.x,
                &mut self.data_mem,
                pc, 
                pc_nxt,
                |imm| imm.sext(xlen),
                |uimm| uimm.zext(xlen),
                || xlen == Xlen::X64 || xlen == Xlen::X128,
                || xlen == Xlen::X128,
                || true, // todo: read from CSR
                || true // todo: read from CSR
            )?,
            Instruction::RVF(_ins) => todo!()
        }
        Ok(())
    }
}

fn shamt32(imm: Imm) -> u32 {
    imm.low_u32() & 0b11111
}

fn shamt32r(data: Usize) -> u32 {
    data.low_u32() & 0b11111
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
            let data = data_mem.read_i8(addr)?;
            x.w_sext8(i.rd, data);
        }
        Lh(i) => {
            let addr = pc_to_mem_addr(x.r_usize(i.rs1) + sext(i.imm));
            let data = data_mem.read_i16(addr)?;
            x.w_sext16(i.rd, data);
        }
        Lw(i) => {
            let addr = pc_to_mem_addr(x.r_usize(i.rs1) + sext(i.imm));
            let data = data_mem.read_i32(addr)?;
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
            x.r_u8(s.rs2),
        )?,
        Sh(s) => data_mem.write_u16(
            pc_to_mem_addr(x.r_usize(s.rs1) + sext(s.imm)),
            x.r_u16(s.rs2),
        )?,
        Sw(s) => data_mem.write_u32(
            pc_to_mem_addr(x.r_usize(s.rs1) + sext(s.imm)),
            x.r_u32(s.rs2),
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
        Ebreak(_) => todo!("ebreak"),
    }
    Ok(())
}

fn shamt64(imm: Imm) -> u32 {
    imm.low_u32() & 0b111111
}

fn shamt64r(data: Usize) -> u32 {
    data.low_u32() & 0b111111
}

fn exec_rv64i<'a, SEXT: Fn(Imm) -> Isize, X64: Fn() -> bool>(
    ins: RV64I,
    x: &mut XReg,
    data_mem: &mut Physical<'a>,
    sext: SEXT,
    has_x64: X64,
) -> Result<()> {
    if !has_x64() { 
        return Err(ExecError::ExtensionNotSupported)?;
    }
    use RV64I::*;
    match ins {
        Lwu(i) => {
            let addr = pc_to_mem_addr(x.r_usize(i.rs1) + sext(i.imm));
            let data = data_mem.read_u32(addr)?;
            x.w_zext32(i.rd, data);
        }
        Ld(i) => {
            let addr = pc_to_mem_addr(x.r_usize(i.rs1) + sext(i.imm));
            let data = data_mem.read_i64(addr)?;
            x.w_sext64(i.rd, data);
        }
        Sd(s) => data_mem.write_u64(
            pc_to_mem_addr(x.r_usize(s.rs1) + sext(s.imm)),
            x.r_u64(s.rs2),
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
        Addiw(i) => x.w_sext32(i.rd, x.r_i32(i.rs1).wrapping_add(i.imm.low_i32())),
        Slliw(i) => {
            let val = x.r_i32(i.rs1).checked_shl(shamt32(i.imm)).unwrap_or(0);
            x.w_sext32(i.rd, val)
        }
        Srliw(i) => {
            let val = x.r_u32(i.rs1).checked_shr(shamt32(i.imm)).unwrap_or(0);
            x.w_sext32(i.rd, i32::from_ne_bytes(val.to_be_bytes()))
        }
        Sraiw(i) => {
            let val = x.r_i32(i.rs1).checked_shr(shamt32(i.imm)).unwrap_or(0);
            x.w_sext32(i.rd, val)
        }
        Addw(r) => x.w_sext32(r.rd, x.r_i32(r.rs1).wrapping_add(x.r_i32(r.rs2))),
        Subw(r) => x.w_sext32(r.rd, x.r_i32(r.rs1).wrapping_sub(x.r_i32(r.rs2))),
        Sllw(r) => {
            let val = x
                .r_i32(r.rs1)
                .checked_shl(shamt32r(x.r_usize(r.rs2)))
                .unwrap_or(0);
            x.w_sext32(r.rd, val)
        }
        Srlw(r) => {
            let val = x
                .r_u32(r.rs1)
                .checked_shr(shamt32r(x.r_usize(r.rs2)))
                .unwrap_or(0);
            x.w_sext32(r.rd, i32::from_ne_bytes(val.to_be_bytes()))
        }
        Sraw(r) => {
            let val = x
                .r_i32(r.rs1)
                .checked_shr(shamt32r(x.r_usize(r.rs2)))
                .unwrap_or(0);
            x.w_sext32(r.rd, val)
        }
    }
    Ok(())
}

fn exec_rvzicsr<ZEXT: Fn(Uimm) -> Usize>(
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

const X1_RA: u8 = 1;
const X2_SP: u8 = 2;

fn exec_rvc<
    'a,
    SEXT: Fn(Imm) -> Isize,
    ZEXT: Fn(Uimm) -> Usize,
    X64: Fn() -> bool,
    X128: Fn() -> bool,
    F32: Fn() -> bool,
    F64: Fn() -> bool,
>(
    ins: RVC,
    x: &mut XReg,
    data_mem: &mut Physical<'a>,
    pc: Usize,
    pc_nxt: &mut Usize,
    sext: SEXT,
    zext: ZEXT,
    has_x64: X64,
    has_x128: X128,
    has_f32: F32,
    has_f64: F64,
) -> Result<()> {
    let shamt_c = |imm: Imm| -> Result<u32> {
        if has_x128() {
            todo!("RV128I")
        }
        let s64 = imm.low_u32() & 0b111111;
        if !has_x64() && s64 >= 0b100000 {
            return Err(ExecError::ExtensionNotSupported)?;
        };
        Ok(s64)
    };
    use RVC::*;
    // if r.rd!=0 or r.rs1 != 0 => prevent side effects
    match ins {
        Caddi4spn(ciw) => x.w_usize(ciw.rd, x.r_usize(X2_SP) + zext(ciw.uimm)),
        Cfld(_cl) => {
            if has_x128() || !has_f64() { // RV32DC or RV64DC
                return Err(ExecError::ExtensionNotSupported)?;
            }
            todo!("D extension")
        },
        Clq(_cl) => {
            if !has_x128() { // RV128C
                return Err(ExecError::ExtensionNotSupported)?;
            }
            todo!("RV128I")
        },
        Clw(cl) => {
            let addr = pc_to_mem_addr(x.r_usize(cl.rs1) + sext(cl.imm));
            let data = data_mem.read_i32(addr)?;
            x.w_sext32(cl.rd, data);
        },
        Cflw(_clt) => {
            if !has_f32() || has_x64() || has_x128() { // RV32FC
                return Err(ExecError::ExtensionNotSupported)?;
            }
            todo!("F extension")
        },
        Cld(cl) => {
            if !has_x64() && !has_x128() { // RV64C or RV128C
                return Err(ExecError::ExtensionNotSupported)?;
            }
            let addr = pc_to_mem_addr(x.r_usize(cl.rs1) + sext(cl.imm));
            let data = data_mem.read_i64(addr)?;
            x.w_sext64(cl.rd, data);
        },
        Cfsd(_cs) => {
            if has_x128() || !has_f64() { // RV32DC or RV64DC
                return Err(ExecError::ExtensionNotSupported)?;
            }
            todo!("D extension")
        },
        Csq(_cs) => {
            if !has_x128() { // RV128C
                return Err(ExecError::ExtensionNotSupported)?;
            }
            todo!("RV128I")
        },
        Csw(cs) => data_mem.write_u32(
            pc_to_mem_addr(x.r_usize(cs.rs1) + sext(cs.imm)),
            x.r_u32(cs.rs2),
        )?,
        Cfsw(_cs) => {
            if !has_f32() || has_x64() || has_x128() { // RV32FC
                return Err(ExecError::ExtensionNotSupported)?;
            }
            todo!("F extension")
        },
        Csd(cs) => {
            if !has_x64() && !has_x128() { // RV64C or RV128C
                return Err(ExecError::ExtensionNotSupported)?;
            }
            data_mem.write_u64(
                pc_to_mem_addr(x.r_usize(cs.rs1) + sext(cs.imm)),
                x.r_u64(cs.rs2),
            )?
        },
        Cnop(_) => { /* nop */ }, 
        Caddi(ci) => {
            x.w_usize(ci.rdrs1, x.r_usize(ci.rdrs1) + sext(ci.imm))
        },
        Cjal(cj) => {
            if has_x64() || has_x128() { // RV32C
                return Err(ExecError::ExtensionNotSupported)?;
            }
            let pc_link = *pc_nxt;
            *pc_nxt = pc + sext(cj.target);
            x.w_usize(X1_RA, pc_link);
        },
        Caddiw(ci) => {
            if !has_x64() && !has_x128() { // RV64C or RV128C
                return Err(ExecError::ExtensionNotSupported)?;
            }
            x.w_sext32(ci.rdrs1, x.r_i32(ci.rdrs1).wrapping_add(ci.imm.low_i32()))
        },
        Cli(ci) => {
            x.w_isize(ci.rdrs1, sext(ci.imm))
        },
        Caddi16sp(ci) => {
            x.w_usize(X2_SP, x.r_usize(X2_SP) + sext(ci.imm))
        },
        Clui(ci) => {
            x.w_isize(ci.rdrs1, sext(ci.imm))
        },
        Csrli(ci) => {
            x.w_usize(ci.rdrs1, x.r_usize(ci.rdrs1) >> shamt_c(ci.imm)?);
        },
        Csrli64(_ci) => { // c.srlid?
            if !has_x128() { // RV128C
                return Err(ExecError::ExtensionNotSupported)?;
            }
            todo!("RV128I")
        },
        Csrai(ci) => {
            x.w_isize(ci.rdrs1, x.r_isize(ci.rdrs1) >> shamt_c(ci.imm)?);
        },
        Csrai64(_ci) => { // c.sraid?
            if !has_x128() { // RV128C
                return Err(ExecError::ExtensionNotSupported)?;
            }
            todo!("RV128I")
        },
        Candi(ci) =>  x.w_usize(ci.rdrs1, x.r_usize(ci.rdrs1) & sext(ci.imm)),
        Csub(ca) => {
            x.w_usize(ca.rdrs1, x.r_usize(ca.rdrs1) - x.r_usize(ca.rs2));
        },
        Cxor(ca) => {
            x.w_usize(ca.rdrs1, x.r_usize(ca.rdrs1) ^ x.r_usize(ca.rs2));
        },
        Cor(ca) => {
            x.w_usize(ca.rdrs1, x.r_usize(ca.rdrs1) | x.r_usize(ca.rs2));
        },
        Cand(ca) => {
            x.w_usize(ca.rdrs1, x.r_usize(ca.rdrs1) & x.r_usize(ca.rs2));
        },
        Csubw(ca) => {
            x.w_sext32(ca.rdrs1, x.r_i32(ca.rdrs1).wrapping_sub(x.r_i32(ca.rs2)))
        },
        Caddw(ca) => {
            x.w_sext32(ca.rdrs1, x.r_i32(ca.rdrs1).wrapping_add(x.r_i32(ca.rs2)))
        },
        Cj(cj) => *pc_nxt = pc + sext(cj.target),
        Cbeqz(cb) => {
            if x.r_usize(cb.rs1) == x.r_usize(0) {
                *pc_nxt = pc + sext(cb.off)
            }
        },
        Cbnez(cb) => {
            if x.r_usize(cb.rs1) != x.r_usize(0) {
                *pc_nxt = pc + sext(cb.off)
            }
        },
        Cslli(ci) => {
            x.w_usize(ci.rdrs1, x.r_usize(ci.rdrs1) << shamt_c(ci.imm)?);
        },
        Cslli64(_ci) => { // c.sllid?
            if !has_x128() { // RV128C
                return Err(ExecError::ExtensionNotSupported)?;
            }
            todo!("RV128I")
        },
        Cfldsp(_ci) => {
            if has_x128() || !has_f64() { // RV32DC or RV64DC
                return Err(ExecError::ExtensionNotSupported)?;
            }
            todo!("D extension")
        },
        Clqsp(_ci) => {
            if !has_x128() { // RV128C
                return Err(ExecError::ExtensionNotSupported)?;
            }
            todo!("RV128I")
        },
        Clwsp(ci) => {
            let addr = pc_to_mem_addr(x.r_usize(X2_SP) + sext(ci.imm));
            let data = data_mem.read_i32(addr)?;
            x.w_sext32(ci.rdrs1, data);
        },
        Cflwsp(_ci) => {
            if !has_f32() || has_x64() || has_x128() { // RV32FC
                return Err(ExecError::ExtensionNotSupported)?;
            }
            todo!("F extension")
        },
        Cldsp(ci) => {
            if !has_x64() || !has_x128() { // RV64C or RV128C
                return Err(ExecError::ExtensionNotSupported)?;
            }
            let addr = pc_to_mem_addr(x.r_usize(X2_SP) + sext(ci.imm));
            let data = data_mem.read_i64(addr)?;
            x.w_sext64(ci.rdrs1, data);
        },
        Cjr(cr) => *pc_nxt = x.r_usize(cr.rdrs1),
        Cmv(cr) => x.w_usize(cr.rdrs1, x.r_usize(cr.rs2)),
        Cebreak(_cr) => todo!("ebreak"),
        Cjalr(cr) => {
            let pc_link = *pc_nxt;
            *pc_nxt = x.r_usize(cr.rdrs1);
            x.w_usize(X1_RA, pc_link);
        },
        Cadd(cr) => {
            x.w_usize(cr.rdrs1, x.r_usize(cr.rdrs1) + x.r_usize(cr.rs2));
        },
        Cfsdsp(_css) => {
            if has_x128() || !has_f64() { // RV32DC or RV64DC
                return Err(ExecError::ExtensionNotSupported)?;
            }
            todo!("D extension")
        },
        Csqsp(_css) => {
            if !has_x128() { // RV128C
                return Err(ExecError::ExtensionNotSupported)?;
            }
            todo!("RV128I")
        },
        Cswsp(css) => data_mem.write_u32(
            pc_to_mem_addr(x.r_usize(X2_SP) + sext(css.imm)),
            x.r_u32(css.rs2),
        )?,
        Cfswsp(_css) => {
            if !has_f32() || has_x64() || has_x128() { // RV32FC
                return Err(ExecError::ExtensionNotSupported)?;
            }
            todo!("F extension")
        },
        Csdsp(css) => {
            if !has_x64() || !has_x128() { // RV64C or RV128C
                return Err(ExecError::ExtensionNotSupported)?;
            }
            data_mem.write_u64(
                pc_to_mem_addr(x.r_usize(X2_SP) + sext(css.imm)),
                x.r_u64(css.rs2),
            )?;
        },
    }
    Ok(())
}
