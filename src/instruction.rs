use capstone::{arch::{x86::X86OperandType, DetailsArchInsn}, Capstone, Insn};
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
}