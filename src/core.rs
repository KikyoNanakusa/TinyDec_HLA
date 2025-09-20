use std::path::Path;
use crate::disassemble::disassemble_file;
use anyhow::Result;

pub fn decompile(_path: &Path) -> Result<String>{
	let (symbols, instructions) = disassemble_file(_path)?;
	for symbol in symbols {
		println!("Symbol: {} at 0x{:x}", symbol.name, symbol.addr);
	}
	println!();
	println!("Instructions: ");
	for instruction in instructions {
		print!("0x{:x} {} ", instruction.address, instruction.mnemonic);
		if let Some(operands) = instruction.operands {
			print!("{:?}", operands);
		}
		println!();
	}
	Ok(String::from("Hello, Decompiler!"))
}