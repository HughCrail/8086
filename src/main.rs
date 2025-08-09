use anyhow::anyhow;
use bytestream::ByteStream;
use clap::Parser;
use instruction::{Inst, Mnemonic};
use register::Register;
use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::PathBuf,
};

#[macro_use]
mod macros;

mod bytestream;
mod computer;
mod data;
mod instruction;
mod parsers;
mod register;
mod target;

#[derive(Debug, Parser)]
struct Cli {
    #[arg(value_name = "BINFILE")]
    infile: PathBuf,
    #[arg(short, long, value_name = "ASMFILE")]
    outfile: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let mut byte_stream = ByteStream {
        bytes: BufReader::new(File::open(&cli.infile)?).bytes(),
    };

    let infile_name = cli
        .infile
        .file_name()
        .ok_or(anyhow!("invalid in file"))?
        .display();

    if let Some(out_file_path) = cli.outfile {
        let mut out_file = BufWriter::new(File::create(&out_file_path)?);
        writeln!(out_file, ";{infile_name}")?;
        writeln!(out_file)?;
        writeln!(out_file, "bits 16")?;
        writeln!(out_file)?;

        while let Some(instruction) = Inst::parse(&mut byte_stream)? {
            writeln!(out_file, "{instruction}")?;
        }

        return Ok(());
    }

    let mut computer = computer::Computer::new();
    println!("--- test\\{infile_name} execution ---");
    while let Some(instruction) = Inst::parse(&mut byte_stream)? {
        let update = computer.execute_instruction(&instruction)?;
        println!("{instruction} ; {update} ");
    }
    computer.print_registers();

    Ok(())
}
