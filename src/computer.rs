use crate::{Inst, Mnemonic, Target, data::Data, instruction::Operand, register::Register};
use anyhow::anyhow;
use std::fmt::Display;

#[derive(Debug)]
pub(crate) struct Computer {
    pub(crate) registers: [u16; 8],
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub(crate) enum Reg {
    AX,
    BX,
    CX,
    DX,
    SP,
    BP,
    SI,
    DI,
}

impl Reg {
    pub(crate) fn from(r: Register) -> Self {
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
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Reg::AX => "ax",
            Reg::BX => "bx",
            Reg::CX => "cx",
            Reg::DX => "dx",
            Reg::SP => "sp",
            Reg::BP => "bp",
            Reg::SI => "si",
            Reg::DI => "di",
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
        Self { registers: [0; 8] }
    }

    pub(crate) fn execute_instruction(&mut self, i: &Inst) -> anyhow::Result<Update> {
        use Mnemonic::*;
        match i.mnemonic {
            Add => todo!(),
            Mov => {
                let (Some(dest), Some(source)) = &i.operands else {
                    return Err(anyhow!("invalid operands for move"));
                };
                match (dest, source) {
                    (Operand::Target(target), Operand::Data(data)) => match target {
                        Target::Register(register) => self.update_register(*register, data),
                        Target::Memory(_) => todo!(),
                    },
                    _ => todo!(),
                }
            }
            Sub => todo!(),
            Cmp => todo!(),
            Jnz => todo!(),
            Je => todo!(),
            Jl => todo!(),
            Jle => todo!(),
            Jb => todo!(),
            Jbe => todo!(),
            Jp => todo!(),
            Jo => todo!(),
            Js => todo!(),
            Jnl => todo!(),
            Jg => todo!(),
            Jnb => todo!(),
            Ja => todo!(),
            Jnp => todo!(),
            Jno => todo!(),
            Jns => todo!(),
            Loop => todo!(),
            Loopz => todo!(),
            Loopnz => todo!(),
            Jcxz => todo!(),
        }
    }

    pub(crate) fn print_registers(&self) {
        println!();
        println!("Final registers:");
        for (i, r) in self.registers.iter().enumerate() {
            println!(
                "      {}: {:#06x} ({})",
                unsafe { std::mem::transmute::<u8, Reg>(i as u8) }.as_str(),
                r,
                r
            )
        }
        println!()
    }

    pub(crate) fn update_register(
        &mut self,
        register: Register,
        data: &Data,
    ) -> anyhow::Result<Update> {
        let reg = Reg::from(register);
        let from_val = self.registers[reg as usize];
        let to_val = *match (register, data) {
            (Register::AX, Data::Word(v)) => v,
            (Register::CX, Data::Word(v)) => v,
            (Register::DX, Data::Word(v)) => v,
            (Register::BX, Data::Word(v)) => v,
            (Register::SP, Data::Word(v)) => v,
            (Register::BP, Data::Word(v)) => v,
            (Register::SI, Data::Word(v)) => v,
            (Register::DI, Data::Word(v)) => v,
            _ => todo!(),
        };
        self.registers[reg as usize] = to_val;
        Ok(Update {
            reg,
            from_val,
            to_val,
        })
    }
}
