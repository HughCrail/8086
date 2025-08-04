use crate::{
    ByteStream, Register, Target,
    data::{Data, DataArg, RelativeJump},
    instruction::{Operand, Operands},
    target::MemoryAddress,
};

pub(crate) fn parse_reg_mem_either_way(
    byte_1: u8,
    bytes: &mut ByteStream,
) -> anyhow::Result<Operands> {
    let byte_2 = bytes.next()?;
    let is_wide = (byte_1 & 0b1) == 1;
    let reg = Target::Register(Register::from((byte_2 >> 3) & 0b111, is_wide)?);
    let target = Target::parse(bytes, byte_2, is_wide)?;

    let (destination, source) = if (byte_1 & 0b10) != 0 {
        (reg, target)
    } else {
        (target, reg)
    };

    Ok((
        Some(Operand::Target(destination)),
        Some(Operand::Target(source)),
    ))
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
        Some(Operand::Target(destination)),
        Some(Operand::DataArg(DataArg {
            explicit,
            data: Data::parse(bytes, is_wide, check_sign_bit && byte_1 & 0b10 != 0)?,
        })),
    ))
}

pub(crate) fn parse_imm_to_acc(byte_1: u8, bytes: &mut ByteStream) -> anyhow::Result<Operands> {
    let is_wide = (byte_1 & 0b1) != 0;
    Ok((
        Some(Operand::Target(Target::Register(if is_wide {
            Register::AX
        } else {
            Register::AL
        }))),
        Some(Operand::Data(Data::parse(bytes, is_wide, false)?)),
    ))
}

pub(crate) fn parse_ip_inc_8(byte: u8) -> Operands {
    (
        Some(Operand::RelativeJump(RelativeJump {
            offset: byte as i8 + 2,
        })),
        None,
    )
}
pub(crate) fn parse_mov_imm_to_reg(byte_1: u8, bytes: &mut ByteStream) -> anyhow::Result<Operands> {
    let is_wide = (byte_1 & 0b1000) != 0;
    Ok((
        Some(crate::instruction::Operand::Target(Target::Register(
            Register::from(byte_1 & 0b111, is_wide)?,
        ))),
        Some(crate::instruction::Operand::Data(Data::parse(
            bytes, is_wide, false,
        )?)),
    ))
}

pub(crate) fn parse_mov_acc_to_mem(byte_1: u8, bytes: &mut ByteStream) -> anyhow::Result<Operands> {
    Ok((
        Some(crate::instruction::Operand::Target(parse_mem(
            byte_1, bytes,
        )?)),
        Some(crate::instruction::Operand::Target(Target::Register(
            Register::AX,
        ))),
    ))
}

pub(crate) fn parse_mov_mem_to_acc(byte_1: u8, bytes: &mut ByteStream) -> anyhow::Result<Operands> {
    Ok((
        Some(crate::instruction::Operand::Target(Target::Register(
            Register::AX,
        ))),
        Some(crate::instruction::Operand::Target(parse_mem(
            byte_1, bytes,
        )?)),
    ))
}

fn parse_mem(byte_1: u8, bytes: &mut ByteStream) -> anyhow::Result<Target> {
    Ok(Target::Memory(MemoryAddress::Direct(Data::parse(
        bytes,
        (byte_1 & 0b1) == 1,
        false,
    )?)))
}
