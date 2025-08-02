use crate::{
    ByteStream,
    base::{ImmToAcc, ImmToRegMem, RegMemEitherWay},
    mov::{MovAccToMem, MovImmToReg, MovMemToAcc},
};
use anyhow::anyhow;
use derive_more::Display;
use std::fmt::Display;

#[derive(Debug)]
enum Mnemonic {
    Add,
    Mov,
}

impl Display for Mnemonic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Mnemonic::Add => "add",
            Mnemonic::Mov => "mov",
        })
    }
}

#[derive(Debug)]
pub(crate) struct Inst {
    mnemonic: Mnemonic,
    encoding: Encoding,
}

impl Display for Inst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.mnemonic, self.encoding)
    }
}

#[derive(Debug, Display)]
pub(crate) enum Encoding {
    RegMemEitherWay(RegMemEitherWay),
    ImmToRegMem(ImmToRegMem),
    ImmToAcc(ImmToAcc),
    MovImmToReg(MovImmToReg),
    MovImmToRegMem(ImmToRegMem),
    MovAccToMem(MovAccToMem),
    MovMemToAcc(MovMemToAcc),
}

impl Inst {
    fn new(mnemonic: Mnemonic, encoding: Encoding) -> Self {
        Self { mnemonic, encoding }
    }

    pub(crate) fn parse(bytes: &mut ByteStream) -> anyhow::Result<Option<Self>> {
        let Some(byte_1) = bytes.maybe_next()? else {
            return Ok(None);
        };

        use Mnemonic::*;

        Ok(Some(match byte_1 {
            b if b >> 2 == 0b000000 => Self::new(
                Add,
                Encoding::RegMemEitherWay(RegMemEitherWay::parse(byte_1, bytes)?),
            ),

            b if b >> 2 == 0b100000 => {
                let byte_2 = bytes.next()?;
                Self::new(
                    Add,
                    Encoding::ImmToRegMem(ImmToRegMem::parse(byte_1, byte_2, bytes, true)?),
                )
            }
            b if b >> 1 == 0b0000010 => {
                Self::new(Add, Encoding::ImmToAcc(ImmToAcc::parse(byte_1, bytes)?))
            }
            b if b >> 2 == 0b100010 => Self::new(
                Mov,
                Encoding::RegMemEitherWay(RegMemEitherWay::parse(byte_1, bytes)?),
            ),
            b if b >> 4 == 0b1011 => {
                Self::new(Mov, Encoding::MovImmToReg(MovImmToReg::parse(b, bytes)?))
            }
            b if b >> 1 == 0b1100011 => Self::new(
                Mov,
                Encoding::MovImmToRegMem(ImmToRegMem::parse(b, bytes.next()?, bytes, false)?),
            ),
            b if b >> 1 == 0b1010000 => {
                Self::new(Mov, Encoding::MovMemToAcc(MovMemToAcc::parse(b, bytes)?))
            }
            b if b >> 1 == 0b1010001 => {
                Self::new(Mov, Encoding::MovAccToMem(MovAccToMem::parse(b, bytes)?))
            }
            _ => {
                return Err(anyhow!("unsupported opcode in byte: {byte_1:b}"));
            }
        }))
    }
}
