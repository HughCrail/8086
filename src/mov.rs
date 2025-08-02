use crate::{
    ByteStream, Register, Target,
    data::{Data, DataArg},
    target::MemoryAddress,
};

#[derive(Debug)]
pub(crate) struct Mov {
    pub(crate) destination: Target,
    pub(crate) source: Target,
}

impl Mov {
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

        Ok(Mov {
            destination,
            source,
        })
    }
}

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
            data: Data::parse(bytes, is_wide)?,
        })
    }
}

#[derive(Debug)]
pub(crate) struct MovImmToRegMem {
    pub(crate) destination: Target,
    pub(crate) data: DataArg,
}

impl MovImmToRegMem {
    pub(crate) fn parse(byte_1: u8, bytes: &mut ByteStream) -> anyhow::Result<Self> {
        let is_wide = byte_1 & 0b1 == 1;
        let byte_2 = bytes.next()?;
        let destination = Target::parse(bytes, byte_2, is_wide)?;
        let explicit = matches!(&destination, Target::Memory(_));
        Ok(Self {
            destination,
            data: DataArg {
                explicit,
                data: Data::parse(bytes, is_wide)?,
            },
        })
    }
}

#[derive(Debug)]
pub(crate) struct MovAccToMem {
    pub(crate) mov: Mov,
}

impl MovAccToMem {
    pub(crate) fn parse(byte_1: u8, bytes: &mut ByteStream) -> anyhow::Result<Self> {
        Ok(Self {
            mov: Mov {
                destination: parse_mem(byte_1, bytes)?,
                source: Target::Register(Register::AX),
            },
        })
    }
}

#[derive(Debug)]
pub(crate) struct MovMemToAcc {
    pub(crate) mov: Mov,
}

impl MovMemToAcc {
    pub(crate) fn parse(byte_1: u8, bytes: &mut ByteStream) -> anyhow::Result<Self> {
        Ok(Self {
            mov: Mov {
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
    )?)))
}
