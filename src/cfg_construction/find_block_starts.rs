use std::collections::HashSet;
use capstone::arch::x86::X86Insn;

use crate::instruction::Instruction;

pub fn find_block_starts(instructions: &Vec<Instruction>) -> HashSet<u64> {
    let mut starts = HashSet::new();
    if instructions.is_empty() { return starts; }
    starts.insert(instructions[0].address); // add base address
    for (i, insn) in instructions.iter().enumerate() {
        let id = insn.cs_id;
        if id == X86Insn::X86_INS_ENDBR64 as u32 || id == X86Insn::X86_INS_ENDBR32 as u32 {
            starts.insert(insn.address);
            continue;
        }

        if insn.is_unconditional_jump() || insn.is_call(){
            if let Some(imm) = insn.imm { starts.insert(imm); }
            continue;
        }

        if insn.is_conditional_jump() {
            if let Some(imm) = insn.imm { starts.insert(imm); }
            let next_insn_addr = instructions.get(i + 1).map(|i| i.address);
            if let Some(ft_addr) = next_insn_addr { starts.insert(ft_addr); }
            continue;
        }
    }
    starts
}