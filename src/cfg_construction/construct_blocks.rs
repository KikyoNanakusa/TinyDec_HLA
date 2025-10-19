use std::collections::{HashMap, HashSet, VecDeque};
use anyhow::{Context, Result};
use crate::{block::BlockId, block_store::BlockStore, instruction::Instruction};

pub fn construct_blocks(
	block_store: &mut BlockStore,
	addr_to_insn: &HashMap<u64, Instruction>,
	starts: &HashSet<u64>,
) -> Result<Vec<BlockId>> {
	let mut blocks = Vec::new();
	let mut worklist: VecDeque<u64> = {
		let mut v: Vec<_> = starts.iter().copied().collect();
		v.sort_unstable();
		std::collections::VecDeque::from(v)
	};

	let mut seen = HashSet::new();

	while let Some(start_addr) = worklist.pop_front() {
		if !seen.insert(start_addr) { continue; }
		let mut curr_block_insns = match addr_to_insn.get(&start_addr) {
			Some(insn) => vec![insn.clone()],
			None => continue, 
		};

		loop {
			let curr = curr_block_insns.last().context("Current block instructions is empty")?;
			if curr.is_terminator() { break; };
			match Instruction::get_next_insn(addr_to_insn, curr) {
				Some(next) => {
					if starts.contains(&next.address) { break; };
					curr_block_insns.push(next.clone());
				}
				None => break,
			}
		}
		let end_addr = curr_block_insns.last().context("Current block instructions is empty")?.address;
		let block = block_store.new_block(start_addr, end_addr, curr_block_insns, None);
		blocks.push(block);
	}
	Ok(blocks)
}