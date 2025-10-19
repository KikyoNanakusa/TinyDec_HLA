use std::collections::HashMap;
use anyhow::{Context, Result};
use petgraph::prelude::StableGraph;

use crate::{block_store::BlockStore, condition::Condition, edge_label::EdgeLabel::{FalseBranch, TrueBranch, Unconditional}, instruction::Instruction, util::graph_utils::Cfg};

pub fn construct_graph(
    addr_to_insn: &HashMap<u64, Instruction>,
    block_store: &mut BlockStore,
) -> Result<Cfg> {
	let block_count = block_store.blocks.len();
	let mut graph = StableGraph::with_capacity(block_count, block_count*2);
	let mut addr_to_nid = HashMap::with_capacity(block_count);
	for (id, block) in block_store.clone().blocks.iter() {
		let nid = block_store.add_block_to_graph(&mut graph, *id)?;
		addr_to_nid.insert(block.start, nid);
	}
	for (_, block) in block_store.blocks.iter() {
		let term_insn = block.instructions.last().context("Block has no instructions")?;
        let block_idx = addr_to_nid[&block.start];
        if term_insn.is_ret() { continue; }
        if term_insn.is_conditional_jump()  { 
            let head_cond = Condition::from_block(block);
            if let Some(ft_insn) = Instruction::get_next_insn(addr_to_insn, term_insn) {
                graph.add_edge(block_idx, addr_to_nid[&ft_insn.address], FalseBranch(head_cond.clone()));
            }

            if let Some(target_addr) = term_insn.imm { 
                graph.add_edge(block_idx, addr_to_nid[&target_addr], TrueBranch(head_cond.clone()));
            }
        } else if term_insn.is_unconditional_jump() {
            if let Some(target_addr) = term_insn.imm { 
                graph.add_edge(block_idx, addr_to_nid[&target_addr], Unconditional);
            }
        } else { 
            if let Some(ft_insn) = Instruction::get_next_insn(addr_to_insn, term_insn) {
                graph.add_edge(block_idx, addr_to_nid[&ft_insn.address], Unconditional);
            }
        }
	};
	Ok(graph)
}