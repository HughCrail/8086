use crate::bytestream::ByteStream;
use std::fmt::Display;

#[derive(Debug)]
pub(crate) enum Data {
    Byte(u8),
    Word(u16),
}

impl Data {
    pub(crate) fn parse(
        bytes: &mut ByteStream,
        is_wide: bool,
        sign_bit: bool,
    ) -> anyhow::Result<Self> {
        Ok(match (is_wide, sign_bit) {
            (true, false) => Data::to_word(bytes.next()?, bytes.next()?),
            (true, true) => Data::Word(bytes.next()? as i8 as u16),
            (false, _) => Data::Byte(bytes.next()?),
        })
    }
    pub(crate) fn to_word(b1: u8, b2: u8) -> Self {
        Data::Word(create_word(b1, b2))
    }
}

impl Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Data::Byte(x) => write!(f, "{x}"),
            Data::Word(x) => write!(f, "{x}"),
        }
    }
}

#[derive(Debug)]
pub(crate) struct DataArg {
    pub(crate) explicit: bool,
    pub(crate) data: Data,
}

impl Display for DataArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.explicit {
            write!(
                f,
                "{} {}",
                match self.data {
                    Data::Byte(_) => "byte",
                    Data::Word(_) => "word",
                },
                self.data
            )
        } else {
            write!(f, "{}", self.data)
        }
    }
}

#[derive(Debug)]
pub(crate) enum Displacement {
    Byte(u8),
    Word(u16),
}

impl Displacement {
    pub(crate) fn to_word(b1: u8, b2: u8) -> Self {
        Displacement::Word(create_word(b1, b2))
    }
}

impl Display for Displacement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Displacement::Byte(x) => {
                let val = *x as i8;
                write!(f, "{} {}", if val >= 0 { '+' } else { '-' }, val.abs())
            }
            Displacement::Word(x) => write!(f, "+ {x}"),
        }
    }
}

pub(crate) fn create_word(b1: u8, b2: u8) -> u16 {
    ((b2 as u16) << 8) + b1 as u16
}
