use crate::{
    ByteStream, Register,
    data::{Data, DataArg, RelativeJump},
    instruction::Operands,
    target::{MemoryAddress, Target},
};

pub(crate) fn parse_reg_mem_either_way(
    byte_1: u8,
    bytes: &mut ByteStream,
) -> anyhow::Result<Operands> {
    let byte_2 = bytes.next()?;
    let is_wide = (byte_1 & 0b1) == 1;
    let reg = Target::Register(Register::from_reg((byte_2 >> 3) & 0b111, is_wide)?);
    let target = Target::parse(bytes, byte_2, is_wide)?;

    let (destination, source) = if (byte_1 & 0b10) != 0 {
        (reg, target)
    } else {
        (target, reg)
    };

    Ok((Some(destination.into()), Some(source.into())))
}

pub(crate) fn parse_imm_to_reg_mem(
    byte_1: u8,
    byte_2: u8,
    bytes: &mut ByteStream,
    check_sign_bit: bool,
) -> anyhow::Result<Operands> {
    let is_wide = byte_1 & 0b1 == 1;
    let destination = Target::parse(bytes, byte_2, is_wide)?;
    let explicit = matches!(&destination, Target::Memory(_));
    Ok((
        Some(destination.into()),
        Some(
            DataArg {
                explicit,
                data: Data::parse(bytes, is_wide, check_sign_bit && byte_1 & 0b10 != 0)?,
            }
            .into(),
        ),
    ))
}

pub(crate) fn parse_imm_to_acc(byte_1: u8, bytes: &mut ByteStream) -> anyhow::Result<Operands> {
    let is_wide = (byte_1 & 0b1) != 0;
    Ok((
        Some(if is_wide { Register::AX } else { Register::AL }.into()),
        Some(Data::parse(bytes, is_wide, false)?.into()),
    ))
}

pub(crate) fn parse_ip_inc_8(byte: u8) -> Operands {
    (
        Some(
            RelativeJump {
                offset: byte as i8 + 2,
            }
            .into(),
        ),
        None,
    )
}
pub(crate) fn parse_mov_imm_to_reg(byte_1: u8, bytes: &mut ByteStream) -> anyhow::Result<Operands> {
    let is_wide = (byte_1 & 0b1000) != 0;
    Ok((
        Some(Register::from_reg(byte_1 & 0b111, is_wide)?.into()),
        Some(Data::parse(bytes, is_wide, false)?.into()),
    ))
}

pub(crate) fn parse_mov_acc_to_mem(byte_1: u8, bytes: &mut ByteStream) -> anyhow::Result<Operands> {
    Ok((
        Some(parse_mem(byte_1, bytes)?.into()),
        Some(Register::AX.into()),
    ))
}

pub(crate) fn parse_mov_mem_to_acc(byte_1: u8, bytes: &mut ByteStream) -> anyhow::Result<Operands> {
    Ok((
        Some(Register::AX.into()),
        Some(parse_mem(byte_1, bytes)?.into()),
    ))
}

fn parse_mem(byte_1: u8, bytes: &mut ByteStream) -> anyhow::Result<MemoryAddress> {
    Ok(MemoryAddress::Direct(Data::parse(
        bytes,
        (byte_1 & 0b1) == 1,
        false,
    )?))
}

pub(crate) fn parse_sm_to_rm(bytes: &mut ByteStream) -> anyhow::Result<Operands> {
    let b = bytes.next()?;
    let sr = b >> 3 & 0b11;
    let sr = Register::from_sr(sr)?;
    let t = Target::parse(bytes, b, true)?;
    Ok((Some(t.into()), Some(sr.into())))
}

pub(crate) fn parse_rm_to_sm(bytes: &mut ByteStream) -> anyhow::Result<Operands> {
    let (a, b) = parse_sm_to_rm(bytes)?;
    Ok((b, a))
}
