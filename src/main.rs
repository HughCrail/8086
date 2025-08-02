use anyhow::anyhow;
use bytestream::ByteStream;
use clap::Parser;
use instruction::Inst;
use register::Register;
use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::PathBuf,
};
use target::Target;

mod base;
mod bytestream;
mod data;
mod instruction;
mod mov;
mod register;
mod target;

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
    while let Some(instruction) = Inst::parse(&mut byte_stream)? {
        writeln!(out_file, "{instruction}")?;
    }

    Ok(())
}
