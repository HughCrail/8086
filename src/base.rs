use std::fmt::Display;

use crate::{
    ByteStream, Register, Target,
    data::{Data, DataArg},
};

#[derive(Debug)]
pub(crate) struct RegMemEitherWay {
    pub(crate) destination: Target,
    pub(crate) source: Target,
}

impl RegMemEitherWay {
    pub(crate) fn parse(byte_1: u8, bytes: &mut ByteStream) -> anyhow::Result<Self> {
        let byte_2 = bytes.next()?;
        let is_wide = (byte_1 & 0b1) == 1;
        let reg = Target::Register(Register::from((byte_2 >> 3) & 0b111, is_wide)?);
        let target = Target::parse(bytes, byte_2, is_wide)?;

        let (destination, source) = if (byte_1 & 0b10) != 0 {
            (reg, target)
        } else {
            (target, reg)
        };

        Ok(Self {
            destination,
            source,
        })
    }
}

impl Display for RegMemEitherWay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}", self.destination, self.source)
    }
}

#[derive(Debug)]
pub(crate) struct ImmToRegMem {
    pub(crate) destination: Target,
    pub(crate) data: DataArg,
}

impl ImmToRegMem {
    pub(crate) fn parse(
        byte_1: u8,
        byte_2: u8,
        bytes: &mut ByteStream,
        check_sign_bit: bool,
    ) -> anyhow::Result<Self> {
        let is_wide = byte_1 & 0b1 == 1;
        let destination = Target::parse(bytes, byte_2, is_wide)?;
        let explicit = matches!(&destination, Target::Memory(_));
        Ok(Self {
            destination,
            data: DataArg {
                explicit,
                data: Data::parse(bytes, is_wide, check_sign_bit && byte_1 & 0b10 != 0)?,
            },
        })
    }
}

impl Display for ImmToRegMem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}", self.destination, self.data)
    }
}

#[derive(Debug)]
pub(crate) struct ImmToAcc {
    pub(crate) destination: Register,
    pub(crate) data: Data,
}

impl ImmToAcc {
    pub(crate) fn parse(byte_1: u8, bytes: &mut ByteStream) -> anyhow::Result<Self> {
        let is_wide = (byte_1 & 0b1) != 0;
        Ok(Self {
            destination: if is_wide { Register::AX } else { Register::AL },
            data: Data::parse(bytes, is_wide, false)?,
        })
    }
}

impl Display for ImmToAcc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}", self.destination, self.data)
    }
}

#[derive(Debug)]
pub(crate) struct IPInc8 {
    offset: i8,
}

impl IPInc8 {
    pub(crate) fn parse(byte: u8) -> anyhow::Result<Self> {
        Ok(Self {
            offset: (byte as i8) + 2,
        })
    }
}

impl Display for IPInc8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.offset >= 0 {
            write!(f, "$+{}", self.offset)
        } else {
            write!(f, "$-{}", self.offset.abs())
        }
    }
}
