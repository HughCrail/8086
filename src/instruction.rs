use crate::{
    ByteStream,
    base::{IPInc8, ImmToAcc, ImmToRegMem, RegMemEitherWay},
    mov::{MovAccToMem, MovImmToReg, MovMemToAcc},
};
use anyhow::anyhow;
use derive_more::Display;
use std::fmt::Display;

#[derive(Debug)]
enum Mnemonic {
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
        f.write_str(match self {
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

enum_with_matching_struct! {
    #[derive(Debug, Display)]
    pub(crate) enum Encoding {
        RegMemEitherWay,
        ImmToRegMem,
        ImmToAcc,
        MovImmToReg,
        MovAccToMem,
        MovMemToAcc,
        IPInc8
    }
}

macro_rules! parse {
    ($variant:ident, $($args:expr),*) => {
        $variant::parse($($args),*).map(Encoding::$variant)
    };
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

        let (mnemonic, encoding) = match byte_1 {
            b if b >> 2 == 0b000000 => (Add, parse!(RegMemEitherWay, b, bytes)?),
            b if b >> 1 == 0b0000010 => (Add, parse!(ImmToAcc, b, bytes)?),
            b if b >> 2 == 0b100010 => (Mov, parse!(RegMemEitherWay, b, bytes)?),
            b if b >> 4 == 0b1011 => (Mov, parse!(MovImmToReg, b, bytes)?),
            b if b >> 1 == 0b1100011 => (Mov, parse!(ImmToRegMem, b, bytes.next()?, bytes, false)?),
            b if b >> 1 == 0b1010000 => (Mov, parse!(MovMemToAcc, b, bytes)?),
            b if b >> 1 == 0b1010001 => (Mov, parse!(MovAccToMem, b, bytes)?),
            b if b >> 2 == 0b001010 => (Sub, parse!(RegMemEitherWay, b, bytes)?),
            b if b >> 1 == 0b0010110 => (Sub, parse!(ImmToAcc, b, bytes)?),
            b if b >> 2 == 0b001110 => (Cmp, parse!(RegMemEitherWay, b, bytes)?),
            b if b >> 1 == 0b0011110 => (Cmp, parse!(ImmToAcc, b, bytes)?),
            b if b >> 2 == 0b100000 => {
                let byte_2 = bytes.next()?;
                let op = byte_2 >> 3 & 0b111;
                match op {
                    0b000 => (Add, parse!(ImmToRegMem, b, byte_2, bytes, true)?),
                    0b101 => (Sub, parse!(ImmToRegMem, b, byte_2, bytes, true)?),
                    0b111 => (Cmp, parse!(ImmToRegMem, b, byte_2, bytes, true)?),
                    _ => return Err(anyhow!("usupported op: {op:03b}")),
                }
            }
            0b01110100 => (Je, parse!(IPInc8, bytes.next()?)?),
            0b01111100 => (Jl, parse!(IPInc8, bytes.next()?)?),
            0b01110101 => (Jnz, parse!(IPInc8, bytes.next()?)?),
            0b01111110 => (Jle, parse!(IPInc8, bytes.next()?)?),
            0b01110010 => (Jb, parse!(IPInc8, bytes.next()?)?),
            0b01110110 => (Jbe, parse!(IPInc8, bytes.next()?)?),
            0b01111010 => (Jp, parse!(IPInc8, bytes.next()?)?),
            0b01110000 => (Jo, parse!(IPInc8, bytes.next()?)?),
            0b01111000 => (Js, parse!(IPInc8, bytes.next()?)?),
            0b01111101 => (Jnl, parse!(IPInc8, bytes.next()?)?),
            0b01111111 => (Jg, parse!(IPInc8, bytes.next()?)?),
            0b01110011 => (Jnb, parse!(IPInc8, bytes.next()?)?),
            0b01110111 => (Ja, parse!(IPInc8, bytes.next()?)?),
            0b01111011 => (Jnp, parse!(IPInc8, bytes.next()?)?),
            0b01110001 => (Jno, parse!(IPInc8, bytes.next()?)?),
            0b01111001 => (Jns, parse!(IPInc8, bytes.next()?)?),
            0b11100010 => (Loop, parse!(IPInc8, bytes.next()?)?),
            0b11100001 => (Loopz, parse!(IPInc8, bytes.next()?)?),
            0b11100000 => (Loopnz, parse!(IPInc8, bytes.next()?)?),
            0b11100011 => (Jcxz, parse!(IPInc8, bytes.next()?)?),
            _ => {
                return Err(anyhow!("unsupported opcode in byte: {byte_1:08b}"));
            }
        };
        Ok(Some(Self::new(mnemonic, encoding)))
    }
}
