use crate::{
    Inst, Mnemonic,
    instruction::Operand,
    register::{RegType, Register},
    target::SegmentRegister,
};
use anyhow::anyhow;
use enum_iterator::{Sequence, all};
use std::fmt::Display;

#[derive(Debug)]
pub(crate) struct Computer {
    pub(crate) registers: [u16; 12],
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Sequence)]
pub(crate) enum Reg {
    AX,
    BX,
    CX,
    DX,
    SP,
    BP,
    SI,
    DI,
    ES,
    CS,
    SS,
    DS,
}

impl From<Register> for Reg {
    fn from(r: Register) -> Self {
        match r {
            Register::AL => Self::AX,
            Register::BL => Self::BX,
            Register::CL => Self::CX,
            Register::DL => Self::DX,
            Register::AH => Self::AX,
            Register::BH => Self::BX,
            Register::CH => Self::CX,
            Register::DH => Self::DX,
            Register::AX => Self::AX,
            Register::BX => Self::BX,
            Register::CX => Self::CX,
            Register::DX => Self::DX,
            Register::SP => Self::SP,
            Register::BP => Self::BP,
            Register::SI => Self::SI,
            Register::DI => Self::DI,
        }
    }
}

impl From<SegmentRegister> for Reg {
    fn from(r: SegmentRegister) -> Self {
        match r {
            SegmentRegister::ES => Self::ES,
            SegmentRegister::CS => Self::CS,
            SegmentRegister::SS => Self::SS,
            SegmentRegister::DS => Self::DS,
        }
    }
}

impl Reg {
    pub(crate) fn as_str(self) -> &'static str {
        use Reg::*;
        match self {
            AX => "ax",
            BX => "bx",
            CX => "cx",
            DX => "dx",
            SP => "sp",
            BP => "bp",
            SI => "si",
            DI => "di",
            ES => "es",
            CS => "cs",
            SS => "ss",
            DS => "ds",
        }
    }
}

#[derive(Debug)]
pub(crate) struct Update {
    pub(crate) reg: Reg,
    pub(crate) from_val: u16,
    pub(crate) to_val: u16,
}

impl Display for Update {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{:#x}->{:#x}",
            self.reg.as_str(),
            self.from_val,
            self.to_val
        )
    }
}

impl Computer {
    pub(crate) fn new() -> Self {
        Self { registers: [0; 12] }
    }

    pub(crate) fn execute_instruction(&mut self, i: &Inst) -> anyhow::Result<Update> {
        use Mnemonic::*;
        use Operand::*;
        let (Some(dest), Some(source)) = &i.operands else {
            return Err(anyhow!("invalid operands for move"));
        };
        match i.mnemonic {
            Mov => match (dest, source) {
                (Register(r), Data(d)) => self.update_register((*r).into(), d.into(), r.get_type()),
                (Register(r1), Register(r2)) => self.update_register(
                    (*r1).into(),
                    self.get_register((*r2).into(), r2.get_type()),
                    r1.get_type(),
                ),
                (Register(r), SegmentRegister(sr)) => self.update_register(
                    (*r).into(),
                    self.get_register((*sr).into(), RegType::Wide),
                    r.get_type(),
                ),
                (SegmentRegister(sr), Register(r)) => self.update_register(
                    (*sr).into(),
                    self.get_register((*r).into(), r.get_type()),
                    RegType::Wide,
                ),
                _ => todo!("Haven't implemented: {}", i),
            },
            // Sub => match (dest, source) {},
            _ => todo!("Haven't implemented: {}", i),
        }
    }

    pub(crate) fn print_registers(&self) {
        println!();
        println!("Final registers:");
        for r in all::<Reg>() {
            let val = self.get_register(r, RegType::Wide);
            if val > 0 {
                println!("      {}: {:#06x} ({})", r.as_str(), val, val)
            }
        }
        println!()
    }

    pub(crate) fn update_register(
        &mut self,
        reg: Reg,
        to_val: u16,
        reg_type: RegType,
    ) -> anyhow::Result<Update> {
        let from_val = self.registers[reg as usize];
        let to_val = match reg_type {
            RegType::Low => (from_val & 0b1111111100000000) + to_val,
            RegType::High => (to_val << 8) + (from_val & 0b0000000011111111),
            RegType::Wide => to_val,
        };
        self.registers[reg as usize] = to_val;
        Ok(Update {
            reg,
            from_val,
            to_val,
        })
    }

    fn get_register(&self, reg: Reg, reg_type: RegType) -> u16 {
        let val = self.registers[reg as usize];
        match reg_type {
            RegType::Low => val & 0b0000000011111111,
            RegType::High => (val & 0b1111111100000000) >> 8,
            RegType::Wide => val,
        }
    }
}
