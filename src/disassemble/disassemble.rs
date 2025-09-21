use std::{fs::File, io::{Read, Seek, SeekFrom}, path::Path};
use capstone::{arch::{BuildsCapstone, BuildsCapstoneSyntax}, Arch, Capstone};
use goblin::{elf::Elf, Object};
use anyhow::{Result, bail};

use crate::{symbol::Symbol, instruction::Instruction};

fn init_capstone(architecture: Arch) -> Result<Capstone> {
    let cs = Capstone::new();
    match architecture {
        Arch::X86 => {
            match cs.x86().mode(capstone::arch::x86::ArchMode::Mode64)
                .syntax(capstone::arch::x86::ArchSyntax::Intel)
                .detail(true)
                .build()
            { 
                Ok(cs) => Ok(cs), 
                Err(e) => bail!("Failed to build Capstone: {}", e) 
            }
        }
        _ => bail!("Architecture is not still supported"),
    }
}

fn read_section_by_name(path: &Path, section_name: &str) -> Result<(Vec<u8>, u64)> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let object = Object::parse(&buffer)?;
    if let Object::Elf(elf) = object {
        if let Some(section) = elf.section_headers.iter()
            .find(|section| {
                elf.shdr_strtab.get_at(section.sh_name as usize)
                    .map_or(false, |name| name == section_name)
            })
        {
            let mut data = vec![0u8; section.sh_size as usize];
            file.seek(SeekFrom::Start(section.sh_offset))?;
            file.read_exact(&mut data)?;
            return Ok((data, section.sh_addr));
        }
    }
    bail!("No {} section found or not an ELF", section_name)
}

pub fn extract_symbols(path: &Path) -> Result<Vec<Symbol>> {
    let data = std::fs::read(path)?;
    let elf = Elf::parse(&data)?;
    Ok(Symbol::from_elf(&elf))
}

pub fn disassemble_file(path: &Path) -> Result<Vec<Instruction>> {
    let (data, addr) = read_section_by_name(path, ".text")?;
    let cs = init_capstone(Arch::X86)?;
    let insns = cs.disasm_all(&data, addr)?;
    let instructions = insns.into_iter()
        .map(|i| Instruction::from_insn(&i, &cs))
        .collect::<Result<Vec<_>>>()?;

    Ok(instructions)
}
