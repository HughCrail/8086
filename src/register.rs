use anyhow::anyhow;
use enum_iterator::Sequence;
use std::fmt::Display;

#[derive(Debug)]
pub(crate) enum RegType {
    Low,
    High,
    Wide,
}

#[derive(Debug, Clone, Copy, Sequence)]
pub(crate) enum Register {
    // byte
    AL,
    BL,
    CL,
    DL,
    AH,
    BH,
    CH,
    DH,
    // word
    AX,
    BX,
    CX,
    DX,
    SP,
    BP,
    SI,
    DI,
    // Segment Registers
    ES,
    CS,
    SS,
    DS,
}

impl Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Register {
    pub(crate) fn as_str(&self) -> &'static str {
        use Register::*;
        match self {
            AX => "ax",
            AL => "al",
            CX => "cx",
            CL => "cl",
            DX => "dx",
            DL => "dl",
            BX => "bx",
            BL => "bl",
            SP => "sp",
            AH => "ah",
            BP => "bp",
            CH => "ch",
            SI => "si",
            DH => "dh",
            DI => "di",
            BH => "bh",
            ES => "es",
            CS => "cs",
            SS => "ss",
            DS => "ds",
        }
    }

    pub(crate) fn as_wide_str(&self) -> &'static str {
        use Register::*;
        match self {
            AL | AH => "ax",
            BL | BH => "bx",
            CL | CH => "cx",
            DL | DH => "dx",
            _ => self.as_str(),
        }
    }

    pub(crate) fn from_reg(reg: u8, is_wide: bool) -> anyhow::Result<Self> {
        Ok(match (is_wide, reg) {
            (true, 0b000) => Self::AX,
            (false, 0b000) => Self::AL,
            (true, 0b001) => Self::CX,
            (false, 0b001) => Self::CL,
            (true, 0b010) => Self::DX,
            (false, 0b010) => Self::DL,
            (true, 0b011) => Self::BX,
            (false, 0b011) => Self::BL,
            (true, 0b100) => Self::SP,
            (false, 0b100) => Self::AH,
            (true, 0b101) => Self::BP,
            (false, 0b101) => Self::CH,
            (true, 0b110) => Self::SI,
            (false, 0b110) => Self::DH,
            (true, 0b111) => Self::DI,
            (false, 0b111) => Self::BH,
            _ => return Err(anyhow!("unknown 8-bit register code: {reg:#05b}")),
        })
    }

    pub(crate) fn from_sr(sr: u8) -> anyhow::Result<Self> {
        Ok(match sr {
            0b00 => Self::ES,
            0b01 => Self::CS,
            0b10 => Self::SS,
            0b11 => Self::DS,
            _ => return Err(anyhow!("unknown segment register code: {sr:#05b}")),
        })
    }

    pub(crate) fn get_type(&self) -> RegType {
        use RegType::*;
        use Register::*;
        match self {
            AL | CL | DL | BL => Low,
            AH | CH | DH | BH => High,
            _ => Wide,
        }
    }

    pub(crate) fn get_reg_ix(&self) -> usize {
        use Register::*;
        match self {
            AL | AH | AX => 0,
            BL | BH | BX => 1,
            CL | CH | CX => 2,
            DL | DH | DX => 3,
            SP => 4,
            BP => 5,
            SI => 6,
            DI => 7,
            ES => 8,
            CS => 9,
            SS => 10,
            DS => 11,
        }
    }
}
