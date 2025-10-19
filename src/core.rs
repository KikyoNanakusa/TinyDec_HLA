use std::{collections::HashMap, path::Path};
use crate::{block::BlockId, block_store::BlockStore, cfg_construction::{self, add_virtual_exit}, disassemble, instruction::Instruction, symbol::Symbol, util::graph_utils, visualize};
use anyhow::{Context, Result};

fn label_blocks(block_store: &mut BlockStore, block_ids: &mut [BlockId], symbols: &[Symbol]) {
    for block_id in block_ids.iter_mut() {
        if let Some(block) = block_store.get_mut(*block_id) {
            if let Some(sym) = symbols.iter().find(|s| s.addr == block.start) {
                block.label = Some(sym.name.clone());
            }
        }
    }
}

pub fn decompile(_path: &Path) -> Result<String>{
	let instructions = disassemble::disassemble_file(_path)?;
	let symbols = disassemble::extract_symbols(_path)?;
    let starts = cfg_construction::find_block_starts(&instructions);

    let addr_to_insn = instructions.into_iter()
        .map(|insn| (insn.address, insn))
        .collect::<HashMap<u64, Instruction>>();

    // Build blocks and label
    let mut block_store = BlockStore::new();
    let mut block_ids = cfg_construction::construct_blocks(&mut block_store, &addr_to_insn, &starts)?;
    label_blocks(&mut block_store, &mut block_ids, &symbols);

    let graph = cfg_construction::construct_graph(&addr_to_insn, &mut block_store)?;
    let main_entry = graph
        .node_indices()
        .find(|&idx| block_store.get(*graph.node_weight(idx).unwrap()).unwrap().label == Some("main".to_string()))
        .context("Main entry not found")?;

    let (main_tree, entry_node) = graph_utils::extract_reachable_subgraph(&graph, main_entry, &mut block_store);
    let (main_tree, vexit_node) = add_virtual_exit::add_virtual_exit(main_tree, &mut block_store)?;
    visualize::cfg_output::write_cfg_dot(&main_tree, &block_store, "main");

	Ok(String::from("Hello, Decompiler!"))
}