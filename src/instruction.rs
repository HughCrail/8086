use crate::{
    ByteStream,
    mov::{Mov, MovAccToMem, MovImmToReg, MovImmToRegMem, MovMemToAcc},
};
use anyhow::anyhow;
use std::fmt::Display;

#[derive(Debug)]
pub(crate) enum Inst {
    Mov(Mov),
    MovImmToReg(MovImmToReg),
    MovImmToRegMem(MovImmToRegMem),
    MovAccToMem(MovAccToMem),
    MovMemToAcc(MovMemToAcc),
}

impl Inst {
    pub(crate) fn parse(bytes: &mut ByteStream) -> anyhow::Result<Option<Self>> {
        let Some(byte_1) = bytes.maybe_next()? else {
            return Ok(None);
        };

        Ok(Some(match byte_1 {
            b if b >> 4 == 0b1011 => Self::MovImmToReg(MovImmToReg::parse(b, bytes)?),
            b if b >> 2 == 0b100010 => Self::Mov(Mov::parse(byte_1, bytes)?),
            b if b >> 1 == 0b1100011 => Self::MovImmToRegMem(MovImmToRegMem::parse(b, bytes)?),
            b if b >> 1 == 0b1010000 => Self::MovMemToAcc(MovMemToAcc::parse(b, bytes)?),
            b if b >> 1 == 0b1010001 => Self::MovAccToMem(MovAccToMem::parse(b, bytes)?),
            _ => {
                return Err(anyhow!("unsupported opcode in byte: {byte_1:b}"));
            }
        }))
    }
}

impl Display for Inst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Inst::Mov(mov) => {
                write!(f, "mov {}, {}", mov.destination, mov.source)
            }
            Inst::MovImmToReg(mov) => {
                write!(f, "mov {}, {}", mov.destination, mov.data)
            }
            Inst::MovImmToRegMem(mov) => {
                write!(f, "mov {}, {}", mov.destination, mov.data)
            }
            Inst::MovMemToAcc(MovMemToAcc { mov }) => {
                write!(f, "mov {}, {}", mov.destination, mov.source)
            }
            Inst::MovAccToMem(MovAccToMem { mov }) => {
                write!(f, "mov {}, {}", mov.destination, mov.source)
            }
        }
    }
}
