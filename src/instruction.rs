use capstone::{arch::{x86::X86OperandType, DetailsArchInsn}, Capstone, Insn};
use anyhow::Result;

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


#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Instruction {
	pub cs_id: u32,
	pub address: u64, 
	pub size: usize, 
	pub mnemonic: String, 
	pub operands: Option<String>,
	pub imm: Option<u64>,
}

impl Instruction {
	pub fn from_insn(insn: &Insn, cs: &Capstone) -> Self {
		let imm = match extract_imm(cs, insn) {
			Ok(Some(imm)) => Some(imm),
			Ok(None) => None,
			Err(e) => panic!("extract_imm error: {}", e),
		};
		Self {
			cs_id: insn.id().0,
			address: insn.address(), 
			size: insn.len(), 
			mnemonic: insn.mnemonic().unwrap_or("???").to_string(), 
			operands: insn.op_str().map(|s| s.to_string()), 
			imm, 
		}
	}
}