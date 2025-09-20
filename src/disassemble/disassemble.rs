use std::path::Path;
use capstone::{arch::{BuildsCapstone, BuildsCapstoneSyntax}, Arch, Capstone};
use goblin::elf::Elf;
use anyhow::Result;

use crate::{symbol::Symbol, instruction::Instruction, section_data::SectionData};

pub fn init_capstone(architecture: Arch) -> Result<Capstone, capstone::Error> {
    let cs = Capstone::new();
    match architecture {
        Arch::X86 => {
            match cs.x86().mode(capstone::arch::x86::ArchMode::Mode64)
                .syntax(capstone::arch::x86::ArchSyntax::Intel)
                .detail(true)
                .build()
            { 
                Ok(cs) => Ok(cs), 
                Err(e) => Err(e) 
            }
        }
        _ => panic!("Architecture is not still supported"),
    }
}

pub fn disassemble_file(path: &Path) -> Result<(Vec<Symbol>, Vec<Instruction>)> {
    // Read binary and parse ELF
    let data = std::fs::read(path)?;
    let elf = Elf::parse(&data)?;
    let symbols = Symbol::from_elf(&elf);

    // Read .text and disassemble
    let text = SectionData::text_section_from_file(path)?;
    let cs = init_capstone(Arch::X86)?;
    let insns = cs
        .disasm_all(&text.data, text.addr)?;
    let instructions = insns.into_iter()
        .map(|i| Instruction::from_insn(&i, &cs))
        .collect::<Vec<_>>();

    Ok((symbols, instructions))
}
