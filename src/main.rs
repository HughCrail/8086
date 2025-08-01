use anyhow::anyhow;
use clap::Parser;
use std::{
    fmt::Display,
    fs::File,
    io::{BufReader, BufWriter, Bytes, Read, Write},
    path::PathBuf,
};

#[derive(Debug, Parser)]
struct Cli {
    #[arg(value_name = "BINFILE")]
    infile: PathBuf,
    #[arg(short, long, value_name = "ASMFILE")]
    outfile: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut out_file = BufWriter::new(File::create(&cli.outfile)?);
    writeln!(
        out_file,
        ";{}",
        cli.outfile
            .file_name()
            .ok_or(anyhow!("invalid out file"))?
            .display()
    )?;
    writeln!(out_file)?;
    writeln!(out_file, "bits 16")?;
    writeln!(out_file)?;

    let mut byte_stream = ByteStream {
        bytes: BufReader::new(File::open(cli.infile)?).bytes(),
    };
    while let Some(instruction) = parse_instruction(&mut byte_stream)? {
        writeln!(out_file, "{instruction}")?;
    }

    Ok(())
}

#[derive(Debug)]
struct ByteStream {
    bytes: Bytes<BufReader<File>>,
}

impl ByteStream {
    fn next(&mut self) -> anyhow::Result<u8> {
        Ok(self.bytes.next().ok_or(anyhow!("unexpected EOF"))??)
    }
    fn maybe_next(&mut self) -> anyhow::Result<Option<u8>> {
        self.bytes.next().transpose().map_err(anyhow::Error::from)
    }
}

#[derive(Debug, Clone, Copy)]
enum Register {
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
    fn from(reg: u8, is_wide: bool) -> anyhow::Result<Self> {
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
}

#[derive(Debug)]
enum MemoryAddress {
    Direct(Data),
    RegnReg(Register, Register),
    Reg(Register),
    RegnData(Register, Displacement),
    RegnRegnData(Register, Register, Displacement),
}

#[derive(Debug)]
enum Target {
    Register(Register),
    Memory(MemoryAddress),
}

impl Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Target::Register(register) => write!(f, "{register}"),
            Target::Memory(memory_address) => match memory_address {
                MemoryAddress::Direct(data) => write!(f, "[{data}]"),
                MemoryAddress::RegnReg(reg1, reg2) => write!(f, "[{reg1} + {reg2}]",),
                MemoryAddress::Reg(reg) => write!(f, "[{reg}]"),
                MemoryAddress::RegnData(reg, data) => write!(f, "[{reg} {data}]"),
                MemoryAddress::RegnRegnData(reg1, reg2, data) => {
                    write!(f, "[{reg1} + {reg2} {data}]")
                }
            },
        }
    }
}

#[derive(Debug)]
struct Mov {
    destination: Target,
    source: Target,
}

impl Mov {
    fn parse(byte_1: u8, bytes: &mut ByteStream) -> anyhow::Result<Self> {
        let byte_2 = bytes.next()?;
        let is_wide = (byte_1 & 0b1) == 1;
        let reg = Target::Register(Register::from((byte_2 >> 3) & 0b111, is_wide)?);
        let target = parse_target(bytes, byte_2, is_wide)?;

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

fn parse_target(
    bytes: &mut ByteStream,
    byte_2: u8,
    is_wide: bool,
) -> Result<Target, anyhow::Error> {
    use Displacement::Byte;
    use MemoryAddress::*;
    use Register::*;

    let mod_val = byte_2 >> 6;
    let r_m = byte_2 & 0b111;

    if mod_val == 0b11 {
        return Ok(Target::Register(Register::from(r_m, is_wide)?));
    }

    if mod_val == 0b00 {
        return Ok(Target::Memory(match r_m {
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

    Ok(Target::Memory(mem_address))
}

#[derive(Debug)]
struct DataArg {
    explicit: bool,
    data: Data,
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
enum Data {
    Byte(u8),
    Word(u16),
}

impl Data {
    fn parse(bytes: &mut ByteStream, is_wide: bool) -> anyhow::Result<Self> {
        Ok(if is_wide {
            Data::to_word(bytes.next()?, bytes.next()?)
        } else {
            Data::Byte(bytes.next()?)
        })
    }
    fn to_word(b1: u8, b2: u8) -> Self {
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

fn create_word(b1: u8, b2: u8) -> u16 {
    ((b2 as u16) << 8) + b1 as u16
}

#[derive(Debug)]
enum Displacement {
    Byte(u8),
    Word(u16),
}

impl Displacement {
    fn to_word(b1: u8, b2: u8) -> Self {
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

#[derive(Debug)]
struct MovImmToReg {
    destination: Register,
    data: Data,
}

impl MovImmToReg {
    fn parse(byte_1: u8, bytes: &mut ByteStream) -> anyhow::Result<Self> {
        let is_wide = (byte_1 & 0b1000) != 0;
        Ok(MovImmToReg {
            destination: Register::from(byte_1 & 0b111, is_wide)?,
            data: Data::parse(bytes, is_wide)?,
        })
    }
}

#[derive(Debug)]
struct MovImmToRegMem {
    destination: Target,
    data: DataArg,
}

impl MovImmToRegMem {
    fn parse(byte_1: u8, bytes: &mut ByteStream) -> anyhow::Result<Self> {
        let is_wide = byte_1 & 0b1 == 1;
        let byte_2 = bytes.next()?;
        let destination = parse_target(bytes, byte_2, is_wide)?;
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
struct MovAccToMem(Mov);

impl MovAccToMem {
    fn parse(byte_1: u8, bytes: &mut ByteStream) -> anyhow::Result<Self> {
        Ok(Self(Mov {
            destination: parse_mem(byte_1, bytes)?,
            source: Target::Register(Register::AX),
        }))
    }
}

#[derive(Debug)]
struct MovMemToAcc(Mov);

impl MovMemToAcc {
    fn parse(byte_1: u8, bytes: &mut ByteStream) -> anyhow::Result<Self> {
        Ok(Self(Mov {
            destination: Target::Register(Register::AX),
            source: parse_mem(byte_1, bytes)?,
        }))
    }
}

fn parse_mem(byte_1: u8, bytes: &mut ByteStream) -> anyhow::Result<Target> {
    Ok(Target::Memory(MemoryAddress::Direct(Data::parse(
        bytes,
        (byte_1 & 0b1) == 1,
    )?)))
}

#[derive(Debug)]
enum Inst {
    Mov(Mov),
    MovImmToReg(MovImmToReg),
    MovImmToRegMem(MovImmToRegMem),
    MovAccToMem(MovAccToMem),
    MovMemToAcc(MovMemToAcc),
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
            Inst::MovMemToAcc(MovMemToAcc { 0: mov }) => {
                write!(f, "mov {}, {}", mov.destination, mov.source)
            }
            Inst::MovAccToMem(MovAccToMem { 0: mov }) => {
                write!(f, "mov {}, {}", mov.destination, mov.source)
            }
        }
    }
}

fn parse_instruction(bytes: &mut ByteStream) -> anyhow::Result<Option<Inst>> {
    let Some(byte_1) = bytes.maybe_next()? else {
        return Ok(None);
    };

    Ok(Some(match byte_1 {
        b if b >> 4 == 0b1011 => Inst::MovImmToReg(MovImmToReg::parse(b, bytes)?),
        b if b >> 2 == 0b100010 => Inst::Mov(Mov::parse(byte_1, bytes)?),
        b if b >> 1 == 0b1100011 => Inst::MovImmToRegMem(MovImmToRegMem::parse(b, bytes)?),
        b if b >> 1 == 0b1010000 => Inst::MovMemToAcc(MovMemToAcc::parse(b, bytes)?),
        b if b >> 1 == 0b1010001 => Inst::MovAccToMem(MovAccToMem::parse(b, bytes)?),
        _ => {
            return Err(anyhow!("unsupported opcode in byte: {byte_1:b}"));
        }
    }))
}
