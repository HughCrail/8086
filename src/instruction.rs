use crate::{
    Register,
    bytestream::ByteStream,
    data::{Data, DataArg, RelativeJump},
    parsers,
    target::{MemoryAddress, Target},
};
use anyhow::anyhow;
use derive_more::Display;
use std::{fmt::Display, io::Read};

#[derive(Debug)]
pub(crate) enum Mnemonic {
    Add,
    Mov,
    Sub,
    Cmp,
    Jnz,
    Je,
    Jl,
    Jle,
    Jb,
    Jbe,
    Jp,
    Jo,
    Js,
    Jnl,
    Jg,
    Jnb,
    Ja,
    Jnp,
    Jno,
    Jns,
    Loop,
    Loopz,
    Loopnz,
    Jcxz,
}

impl Display for Mnemonic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Mnemonic {
    fn as_str(&self) -> &str {
        match self {
            Mnemonic::Add => "add",
            Mnemonic::Mov => "mov",
            Mnemonic::Sub => "sub",
            Mnemonic::Cmp => "cmp",
            Mnemonic::Jnz => "jnz",
            Mnemonic::Je => "je",
            Mnemonic::Jl => "jl",
            Mnemonic::Jle => "jle",
            Mnemonic::Jb => "jb",
            Mnemonic::Jbe => "jbe",
            Mnemonic::Jp => "jp",
            Mnemonic::Jo => "jo",
            Mnemonic::Js => "js",
            Mnemonic::Jnl => "jnl",
            Mnemonic::Jg => "jg",
            Mnemonic::Jnb => "jnb",
            Mnemonic::Ja => "ja",
            Mnemonic::Jnp => "jnp",
            Mnemonic::Jno => "jno",
            Mnemonic::Jns => "jns",
            Mnemonic::Loop => "loop",
            Mnemonic::Loopz => "loopz",
            Mnemonic::Loopnz => "loopnz",
            Mnemonic::Jcxz => "jcxz",
        }
    }
}

enum_with_matching_struct! {
    #[derive(Debug, Display)]
    pub enum Operand {
        Register,
        MemoryAddress,
        DataArg,
        Data,
        RelativeJump,
    }
}

impl From<Target> for Operand {
    fn from(t: Target) -> Self {
        match t {
            Target::Register(register) => Self::Register(register),
            Target::Memory(memory_address) => Self::MemoryAddress(memory_address),
        }
    }
}

impl From<Register> for Operand {
    fn from(r: Register) -> Self {
        Self::Register(r)
    }
}

impl From<MemoryAddress> for Operand {
    fn from(m: MemoryAddress) -> Self {
        Self::MemoryAddress(m)
    }
}

impl From<Data> for Operand {
    fn from(d: Data) -> Self {
        Self::Data(d)
    }
}

impl From<DataArg> for Operand {
    fn from(d: DataArg) -> Self {
        Self::DataArg(d)
    }
}

impl From<RelativeJump> for Operand {
    fn from(r: RelativeJump) -> Self {
        Self::RelativeJump(r)
    }
}

pub(crate) type Operands = (Option<Operand>, Option<Operand>);

#[derive(Debug)]
pub(crate) struct Inst {
    pub(crate) mnemonic: Mnemonic,
    pub(crate) operands: Operands,
}

impl Display for Inst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.mnemonic.as_str())?;
        if let Some(op) = &self.operands.0 {
            write!(f, " {op}")?;
        }
        if let Some(op) = &self.operands.1 {
            write!(f, ", {op}")?;
        }
        Ok(())
    }
}

impl Inst {
    fn new(mnemonic: Mnemonic, op1: Option<Operand>, op2: Option<Operand>) -> Self {
        Self {
            mnemonic,
            operands: (op1, op2),
        }
    }

    pub(crate) fn parse<T: Read>(bytes: &mut ByteStream<T>) -> anyhow::Result<Option<Self>> {
        let Some(byte_1) = bytes.maybe_next()? else {
            return Ok(None);
        };

        use Mnemonic::*;
        use parsers::*;

        let (mnemonic, (op1, op2)) = match byte_1 {
            b if b >> 2 == 0b000000 => (Add, parse_reg_mem_either_way(b, bytes)?),
            b if b >> 1 == 0b0000010 => (Add, parse_imm_to_acc(b, bytes)?),
            b if b >> 2 == 0b100010 => (Mov, parse_reg_mem_either_way(b, bytes)?),
            b if b >> 4 == 0b1011 => (Mov, parse_mov_imm_to_reg(b, bytes)?),
            b if b >> 1 == 0b1100011 => {
                (Mov, parse_imm_to_reg_mem(b, bytes.next()?, bytes, false)?)
            }
            b if b >> 1 == 0b1010000 => (Mov, parse_mov_mem_to_acc(b, bytes)?),
            b if b >> 1 == 0b1010001 => (Mov, parse_mov_acc_to_mem(b, bytes)?),
            0b10001110 => (Mov, parse_rm_to_sm(bytes)?),
            0b10001100 => (Mov, parse_sm_to_rm(bytes)?),
            b if b >> 2 == 0b001010 => (Sub, parse_reg_mem_either_way(b, bytes)?),
            b if b >> 1 == 0b0010110 => (Sub, parse_imm_to_acc(b, bytes)?),
            b if b >> 2 == 0b001110 => (Cmp, parse_reg_mem_either_way(b, bytes)?),
            b if b >> 1 == 0b0011110 => (Cmp, parse_imm_to_acc(b, bytes)?),
            b if b >> 2 == 0b100000 => {
                let byte_2 = bytes.next()?;
                let op = byte_2 >> 3 & 0b111;
                match op {
                    0b000 => (Add, parse_imm_to_reg_mem(b, byte_2, bytes, true)?),
                    0b101 => (Sub, parse_imm_to_reg_mem(b, byte_2, bytes, true)?),
                    0b111 => (Cmp, parse_imm_to_reg_mem(b, byte_2, bytes, true)?),
                    _ => return Err(anyhow!("usupported op: {op:03b}")),
                }
            }
            0b01110100 => (Je, parse_ip_inc_8(bytes.next()?)),
            0b01111100 => (Jl, parse_ip_inc_8(bytes.next()?)),
            0b01110101 => (Jnz, parse_ip_inc_8(bytes.next()?)),
            0b01111110 => (Jle, parse_ip_inc_8(bytes.next()?)),
            0b01110010 => (Jb, parse_ip_inc_8(bytes.next()?)),
            0b01110110 => (Jbe, parse_ip_inc_8(bytes.next()?)),
            0b01111010 => (Jp, parse_ip_inc_8(bytes.next()?)),
            0b01110000 => (Jo, parse_ip_inc_8(bytes.next()?)),
            0b01111000 => (Js, parse_ip_inc_8(bytes.next()?)),
            0b01111101 => (Jnl, parse_ip_inc_8(bytes.next()?)),
            0b01111111 => (Jg, parse_ip_inc_8(bytes.next()?)),
            0b01110011 => (Jnb, parse_ip_inc_8(bytes.next()?)),
            0b01110111 => (Ja, parse_ip_inc_8(bytes.next()?)),
            0b01111011 => (Jnp, parse_ip_inc_8(bytes.next()?)),
            0b01110001 => (Jno, parse_ip_inc_8(bytes.next()?)),
            0b01111001 => (Jns, parse_ip_inc_8(bytes.next()?)),
            0b11100010 => (Loop, parse_ip_inc_8(bytes.next()?)),
            0b11100001 => (Loopz, parse_ip_inc_8(bytes.next()?)),
            0b11100000 => (Loopnz, parse_ip_inc_8(bytes.next()?)),
            0b11100011 => (Jcxz, parse_ip_inc_8(bytes.next()?)),
            _ => {
                return Err(anyhow!("unsupported opcode in byte: {byte_1:08b}"));
            }
        };
        Ok(Some(Self::new(mnemonic, op1, op2)))
    }
}
