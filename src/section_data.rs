use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use goblin::Object;
use anyhow::{Result, bail};

pub struct SectionData {
    pub data: Vec<u8>,
    pub addr: u64,
}

impl SectionData {
    pub fn text_section_from_file(path: &Path) -> Result<SectionData> {
        Self::read_section_by_name(path, ".text")
    }
    fn read_section_by_name(path: &Path, section_name: &str) -> Result<SectionData> {
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
                return Ok(SectionData {
                    data,
                    addr: section.sh_addr,
                });
            }
        }
        bail!("No {} section found or not an ELF", section_name)
    }
}