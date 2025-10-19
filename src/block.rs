use std::fmt;
use crate::instruction::Instruction;

pub type BlockId = u32;

#[derive(Clone)]
pub struct Block {
	pub id: BlockId,
	pub start: u64,
	pub end: u64,
	pub instructions: Vec<Instruction>,
	pub label: Option<String>,
}

impl Block {
	pub fn new(id: BlockId, start: u64, end: u64, instructions: Vec<Instruction>, label: Option<String>) -> Self {
		Self { id, start, end, instructions, label }
	}
}

impl fmt::Display for Block {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		writeln!(f, "Block {{")?;
		writeln!(f, "  start: 0x{:x},", self.start)?;
		writeln!(f, "  end: 0x{:x},", self.end)?;
		writeln!(f, "  instructions: [")?;
		for insn in &self.instructions {
			writeln!(f, "    0x{:x}: {} {}",
				insn.address,
				insn.mnemonic,
				insn.operands.as_ref().unwrap_or(&Vec::new()).join(", ")
			)?;
		}
		writeln!(f, "  ]")?;
		write!(f, "}}")
	}
}