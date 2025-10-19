use std::collections::HashMap;

use capstone::{arch::{x86::{X86OperandType, X86Insn}, DetailsArchInsn}, Capstone, Insn};
use anyhow::{bail, Result};


#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Instruction {
	pub cs_id: u32,
	pub address: u64, 
	pub size: usize, 
	pub mnemonic: String, 
	pub operands: Option<Vec<String>>,
	pub imm: Option<u64>,
}

impl Instruction {
	pub fn from_insn(insn: &Insn, cs: &Capstone) -> Result<Self> {
		let imm = match Self::extract_imm(cs, insn) {
			Ok(Some(imm)) => Some(imm),
			Ok(None) => None,
			Err(e) => bail!("extract_imm error: {}", e),
		};
		let operands = match insn.op_str() {
			Some(s) => Some(Self::split_operands(s)),
			None => None,
		};
		Ok(Self {
			cs_id: insn.id().0,
			address: insn.address(), 
			size: insn.len(), 
			mnemonic: insn.mnemonic().unwrap_or("???").to_string(), 
			operands: operands,
			imm, 
		})
	}
	fn split_operands(operands: &str) -> Vec<String> {
		operands.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
	}
	fn extract_imm(cs: &Capstone, insn: &Insn) -> Result<Option<u64>> {
	    let detail = cs.insn_detail(insn)?;
	    let arch_detail = match detail.arch_detail() {
	        capstone::arch::ArchDetail::X86Detail(d) => d,
	        _ => return Err(anyhow::anyhow!("Not an x86 instruction detail")),
	    };

	    Ok(arch_detail.operands().find_map(|op| {
			match op.op_type {
				X86OperandType::Imm(imm) => Some(imm as u64),
				_ => None,
			}
	    }))
	}
	pub fn is_conditional_jump(&self) -> bool {
		self.cs_id == X86Insn::X86_INS_JE as u32 ||
	    self.cs_id == X86Insn::X86_INS_JNE as u32 ||
	    self.cs_id == X86Insn::X86_INS_JB as u32 ||
	    self.cs_id == X86Insn::X86_INS_JBE as u32 ||
	    self.cs_id == X86Insn::X86_INS_JA as u32 ||
	    self.cs_id == X86Insn::X86_INS_JAE as u32 ||
	    self.cs_id == X86Insn::X86_INS_JL as u32 ||
	    self.cs_id == X86Insn::X86_INS_JLE as u32 ||
	    self.cs_id == X86Insn::X86_INS_JG as u32 ||
	    self.cs_id == X86Insn::X86_INS_JGE as u32
	}
	pub fn is_unconditional_jump(&self) -> bool {
		self.cs_id == X86Insn::X86_INS_JMP as u32
	}
	pub fn is_jump(&self) -> bool {
		self.is_conditional_jump() || self.is_unconditional_jump()
	}
	pub fn is_call(&self) -> bool {
		self.cs_id == X86Insn::X86_INS_CALL as u32
	}
	pub fn is_ret(&self) -> bool {
		self.cs_id == X86Insn::X86_INS_RET as u32
	}
	pub fn is_terminator(&self) -> bool {
		self.is_jump() || self.is_ret()
	}
	pub fn get_next_insn<'a>(addr_to_insn: &'a HashMap<u64, Instruction>, current_insn: &Instruction) -> Option<&'a Instruction> {
		let next_insn_addr = current_insn.address + current_insn.size as u64;
		addr_to_insn.get(&next_insn_addr)
	}
}