use anyhow::anyhow;
use std::fmt::Display;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Register {
    // byte
    AL,
    CL,
    DL,
    BL,
    AH,
    CH,
    DH,
    BH,
    // word
    AX,
    CX,
    DX,
    BX,
    SP,
    BP,
    SI,
    DI,
}

impl Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Register::*;
        f.write_str(match self {
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
        })
    }
}

impl Register {
    pub(crate) fn from(reg: u8, is_wide: bool) -> anyhow::Result<Self> {
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

    pub(crate) fn is_wide(&self) -> bool {
        match self {
            Register::AX
            | Register::CX
            | Register::DX
            | Register::BX
            | Register::SP
            | Register::BP
            | Register::SI
            | Register::DI => true,
            _ => false,
        }
    }
}
