use std::{collections::HashMap, path::Path};
use crate::{block::BlockId, block_store::BlockStore, cfg_construction::{self, add_virtual_exit}, disassemble, instruction::Instruction, region::{Region, RegionId}, region_arena::RegionArena, symbol::Symbol, util::graph_utils::{self, find_entry_node, has_backedge}, visualize};
use anyhow::{Context, Result};
use petgraph::{algo::dominators::simple_fast, visit::DfsPostOrder};

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

    let (main_cfg, entry_node) = graph_utils::extract_reachable_subgraph(&graph, main_entry, &mut block_store);
    let (main_cfg, vexit_node) = add_virtual_exit::add_virtual_exit(main_cfg, &mut block_store)?;
    visualize::cfg_output::write_cfg_dot(&main_cfg, &block_store, "main");

    let mut arena = RegionArena::new();
    let mut block_id_to_rid = HashMap::<BlockId, RegionId>::new();
    for n in main_cfg.node_indices() {
        let bid = *main_cfg.node_weight(n).unwrap();
        block_id_to_rid.insert(bid, arena.alloc(Region::Leaf(bid)));
    }
    
    let entry = find_entry_node(&main_cfg).context("No entry node found")?;
    let mut order = DfsPostOrder::new(&main_cfg, entry);
    let dom = simple_fast(&main_cfg, entry_node);
    while let Some(head) = order.next(&main_cfg) {
        if has_backedge(&main_cfg, head, vexit_node, &dom) {
            println!("cycle discovered: {}", head.index());
        }
    }

	Ok(String::from("Hello, Decompiler!"))
}

