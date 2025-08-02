use derive_more::Display;
use std::fmt::Display;

use crate::{
    ByteStream, Register, Target, base::RegMemEitherWay, data::Data, target::MemoryAddress,
};

#[derive(Debug)]
pub(crate) struct MovImmToReg {
    pub(crate) destination: Register,
    pub(crate) data: Data,
}

impl MovImmToReg {
    pub(crate) fn parse(byte_1: u8, bytes: &mut ByteStream) -> anyhow::Result<Self> {
        let is_wide = (byte_1 & 0b1000) != 0;
        Ok(MovImmToReg {
            destination: Register::from(byte_1 & 0b111, is_wide)?,
            data: Data::parse(bytes, is_wide, false)?,
        })
    }
}

impl Display for MovImmToReg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}", self.destination, self.data)
    }
}

#[derive(Debug, Display)]
pub(crate) struct MovAccToMem {
    pub(crate) base: RegMemEitherWay,
}

impl MovAccToMem {
    pub(crate) fn parse(byte_1: u8, bytes: &mut ByteStream) -> anyhow::Result<Self> {
        Ok(Self {
            base: RegMemEitherWay {
                destination: parse_mem(byte_1, bytes)?,
                source: Target::Register(Register::AX),
            },
        })
    }
}

#[derive(Debug, Display)]
pub(crate) struct MovMemToAcc {
    pub(crate) base: RegMemEitherWay,
}

impl MovMemToAcc {
    pub(crate) fn parse(byte_1: u8, bytes: &mut ByteStream) -> anyhow::Result<Self> {
        Ok(Self {
            base: RegMemEitherWay {
                destination: Target::Register(Register::AX),
                source: parse_mem(byte_1, bytes)?,
            },
        })
    }
}

pub(crate) fn parse_mem(byte_1: u8, bytes: &mut ByteStream) -> anyhow::Result<Target> {
    Ok(Target::Memory(MemoryAddress::Direct(Data::parse(
        bytes,
        (byte_1 & 0b1) == 1,
        false,
    )?)))
}
