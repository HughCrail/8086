use crate::{
    bytestream::ByteStream,
    data::{Data, Displacement},
    register::Register,
};
use anyhow::anyhow;
use std::fmt::Display;

#[derive(Debug)]
pub(crate) enum MemoryAddress {
    Direct(Data),
    RegnReg(Register, Register),
    Reg(Register),
    RegnData(Register, Displacement),
    RegnRegnData(Register, Register, Displacement),
}

impl Display for MemoryAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryAddress::Direct(data) => write!(f, "[{data}]"),
            MemoryAddress::RegnReg(reg1, reg2) => write!(f, "[{reg1} + {reg2}]",),
            MemoryAddress::Reg(reg) => write!(f, "[{reg}]"),
            MemoryAddress::RegnData(reg, data) => write!(f, "[{reg} {data}]"),
            MemoryAddress::RegnRegnData(reg1, reg2, data) => {
                write!(f, "[{reg1} + {reg2} {data}]")
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum SegmentRegister {
    ES,
    CS,
    SS,
    DS,
}

impl Display for SegmentRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use SegmentRegister::*;
        f.write_str(match self {
            ES => "es",
            CS => "cs",
            SS => "ss",
            DS => "ds",
        })
    }
}

impl SegmentRegister {
    pub(crate) fn from(sr: u8) -> anyhow::Result<Self> {
        Ok(match sr {
            0b00 => Self::ES,
            0b01 => Self::CS,
            0b10 => Self::SS,
            0b11 => Self::DS,
            _ => return Err(anyhow!("unknown segment register code: {sr:#05b}")),
        })
    }
}

#[derive(Debug)]
pub(crate) enum Target {
    Register(Register),
    Memory(MemoryAddress),
}

impl Target {
    pub(crate) fn parse(
        bytes: &mut ByteStream,
        byte_2: u8,
        is_wide: bool,
    ) -> Result<Self, anyhow::Error> {
        use Displacement::Byte;
        use MemoryAddress::*;
        use Register::*;

        let mod_val = byte_2 >> 6;
        let r_m = byte_2 & 0b111;

        if mod_val == 0b11 {
            return Ok(Self::Register(Register::from(r_m, is_wide)?));
        }

        if mod_val == 0b00 {
            return Ok(Self::Memory(match r_m {
                0b000 => RegnReg(BX, SI),
                0b001 => RegnReg(BX, DI),
                0b010 => RegnReg(BP, SI),
                0b011 => RegnReg(BP, DI),
                0b100 => Reg(SI),
                0b101 => Reg(DI),
                0b110 => Direct(Data::to_word(bytes.next()?, bytes.next()?)),
                0b111 => Reg(BX),
                _ => unreachable!(),
            }));
        }

        let displacement = if mod_val == 0b01 {
            let d = bytes.next()?;
            if d == 0 { None } else { Some(Byte(d)) }
        } else {
            let b1 = bytes.next()?;
            let b2 = bytes.next()?;
            if b1 == 0 && b2 == 0 {
                None
            } else {
                Some(Displacement::to_word(b1, b2))
            }
        };

        let mem_address = if let Some(disp) = displacement {
            match r_m {
                0b000 => RegnRegnData(BX, SI, disp),
                0b001 => RegnRegnData(BX, DI, disp),
                0b010 => RegnRegnData(BP, SI, disp),
                0b011 => RegnRegnData(BP, DI, disp),
                0b100 => RegnData(SI, disp),
                0b101 => RegnData(DI, disp),
                0b110 => RegnData(BP, disp),
                0b111 => RegnData(BX, disp),
                _ => unreachable!(),
            }
        } else {
            match r_m {
                0b000 => RegnReg(BX, SI),
                0b001 => RegnReg(BX, DI),
                0b010 => RegnReg(BP, SI),
                0b011 => RegnReg(BP, DI),
                0b100 => Reg(SI),
                0b101 => Reg(DI),
                0b110 => Reg(BP),
                0b111 => Reg(BX),
                _ => unreachable!(),
            }
        };

        Ok(Self::Memory(mem_address))
    }
}
