use crate::{
    ByteStream, Inst, Mnemonic,
    instruction::Operand,
    register::{RegType, Register},
};
use anyhow::anyhow;
use bitflags::bitflags;
use enum_iterator::all;
use std::{
    fmt::{self, Display, Write},
    io::{Read, Seek},
    mem::take,
};

bitflags! {
    #[derive(Debug, Clone, Copy)]
    struct Flags: u16 {
        const Sign = 0b000001;
        const Parity = 0b000010;
        const Zero = 0b000100;
    }
}

impl Display for Flags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for flag in self.iter() {
            f.write_char(match flag {
                f if f.contains(Flags::Sign) => 'S',
                f if f.contains(Flags::Parity) => 'P',
                f if f.contains(Flags::Zero) => 'Z',
                _ => unreachable!(),
            })?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct Computer<T> {
    program: ByteStream<T>,
    registers: [u16; 12],
    flags: Flags,
    last_update: Update,
    print_ip: bool,
}

#[derive(Debug)]
pub(crate) struct RegUpdate {
    reg: Register,
    from_val: u16,
    to_val: u16,
}

impl Display for RegUpdate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{:#x}->{:#x}",
            self.reg.as_wide_str(),
            self.from_val,
            self.to_val
        )
    }
}

#[derive(Debug, Default)]
pub(crate) struct Update {
    reg_update: Option<RegUpdate>,
    flag_update: Option<(Flags, Flags)>,
    ip_update: Option<(u64, u64)>,
}

impl Update {
    pub(crate) fn print(&self, print_ip: bool) -> Result<String, fmt::Error> {
        let mut parts = vec![];

        if let Some(reg) = &self.reg_update {
            parts.push(reg.to_string());
        }

        if print_ip && let Some((from, to)) = &self.ip_update {
            parts.push(format!("ip:{from:#x}->{to:#x}"));
        }

        if let Some((from, to)) = &self.flag_update {
            parts.push(format!("flags:{from}->{to}"));
        }

        Ok(parts.join(" "))
    }
}

#[derive(Debug)]
pub(crate) enum ExeResult {
    Halt,
    Success(Inst, Update),
}

impl<T: Read + Seek> Computer<T> {
    pub(crate) fn new(program: ByteStream<T>, print_ip: bool) -> Self {
        Self {
            program,
            registers: [0; 12],
            flags: Flags::empty(),
            last_update: Update::default(),
            print_ip,
        }
    }

    pub(crate) fn execute_instruction(&mut self) -> anyhow::Result<ExeResult> {
        use Mnemonic::*;
        use Operand::*;

        let ip_before = self.program.get_iptr()?;
        let Some(i) = Inst::parse(&mut self.program)? else {
            self.print_registers(ip_before);
            return Ok(ExeResult::Halt);
        };
        let ip_after = self.program.get_iptr()?;
        self.update_ip(ip_before, ip_after);
        let Inst { mnemonic, operands } = &i;

        let (Some(dest), Some(source)) = &operands else {
            todo!("Haven't implemented: {i} => {:?}", i)
        };
        match mnemonic {
            Mov => match (dest, source) {
                (Register(r), Data(d)) => self.update_register(*r, d.into()),
                (Register(r1), Register(r2)) => self.update_register(*r1, self.get_register(*r2)),
                _ => todo!("Haven't implemented: {i} => {:?}", i),
            },
            Sub | Cmp | Add => {
                let op: fn(u16, u16) -> u16 = match mnemonic {
                    Add => |a, b| a.wrapping_add(b),
                    Sub | Cmp => |a, b| a.wrapping_sub(b),
                    _ => unreachable!(),
                };
                let Register(r) = dest else {
                    return Err(anyhow!("invalid destination operand for {i}"));
                };
                let res = self.do_op(
                    r,
                    match source {
                        Register(r) => self.get_register(*r),
                        DataArg(d) => (&d.data).into(),
                        Data(d) => d.into(),
                        _ => Err(anyhow!("invalid source operand for {i}"))?,
                    },
                    op,
                );

                if !matches!(mnemonic, Cmp) {
                    self.update_register(*r, res);
                }
            }
            _ => todo!("Haven't implemented: {i} => {:?}", i),
        };
        Ok(ExeResult::Success(i, take(&mut self.last_update)))
    }

    fn do_op(&mut self, a: &Register, b: u16, op: fn(u16, u16) -> u16) -> u16 {
        let a = self.get_register(*a);
        let res = op(a, b);
        self.update_flags(res);
        res
    }

    pub(crate) fn print_registers(&self, ip: u64) {
        println!();
        println!("Final registers:");
        for r in all::<Register>().filter(|r| matches!(r.get_type(), RegType::Wide)) {
            let val = self.get_register(r);
            if val > 0 {
                println!("      {}: {val:#06x} ({val})", r.as_str());
            }
        }
        if self.print_ip {
            println!("      ip: {ip:#06x} ({ip})");
        }
        if !self.flags.is_empty() {
            println!("   flags: {}", self.flags);
        }
        println!()
    }

    fn update_register(&mut self, reg: Register, to_val: u16) {
        let from_val = self.registers[reg.get_reg_ix()];
        let to_val = match reg.get_type() {
            RegType::Low => (from_val & 0b1111111100000000) + to_val,
            RegType::High => (to_val << 8) + (from_val & 0b0000000011111111),
            RegType::Wide => to_val,
        };
        self.registers[reg.get_reg_ix()] = to_val;
        self.last_update.reg_update = Some(RegUpdate {
            reg,
            from_val,
            to_val,
        })
    }

    fn update_flags(&mut self, result: u16) {
        let flags_before = self.flags;
        self.flags.set(Flags::Sign, (result as i16) < 0);
        self.flags.set(Flags::Zero, result == 0);
        self.flags.set(
            Flags::Parity,
            (16 - (result & 0x00FF).count_zeros()) % 2 == 0,
        );

        if flags_before.bits() != self.flags.bits() {
            self.last_update.flag_update = Some((flags_before, self.flags));
        }
    }

    fn get_register(&self, reg: Register) -> u16 {
        let val = self.registers[reg.get_reg_ix()];
        match reg.get_type() {
            RegType::Low => val & 0b0000000011111111,
            RegType::High => (val & 0b1111111100000000) >> 8,
            RegType::Wide => val,
        }
    }

    fn update_ip(&mut self, ip_before: u64, ip_after: u64) {
        self.last_update.ip_update = Some((ip_before, ip_after));
    }
}
