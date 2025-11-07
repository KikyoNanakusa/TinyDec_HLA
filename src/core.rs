use std::{collections::HashMap, path::Path};
use crate::{block::BlockId, block_store::BlockStore, cfg_construction::{self, add_virtual_exit::{self, add_virtual_exit}}, cfg_structuring::{acyclic_match::match_acyclic, cyclic_match::{contract_cyclic_region, find_smallest_loop, match_cyclic}}, disassemble, instruction::Instruction, region::{Region, RegionId}, region_arena::RegionArena, symbol::Symbol, util::graph_utils::{self, find_entry_node, has_backedge}, visualize::{self, arena_output::print_arena_tree, cfg_output::write_cfg_dot}};
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
    let (mut main_cfg, vexit_node) = add_virtual_exit::add_virtual_exit(main_cfg, &mut block_store)?;
    visualize::cfg_output::write_cfg_dot(&main_cfg, &block_store, "main");

    let mut arena = RegionArena::new();
    let mut block_id_to_rid = HashMap::<BlockId, RegionId>::new();
    for n in main_cfg.node_indices() {
        let bid = *main_cfg.node_weight(n).unwrap();
        block_id_to_rid.insert(bid, arena.alloc(Region::Leaf(bid)));
    }
    
    let mut seq_count = 0;
    let mut virtualized_edges = Vec::new();
    let mut structured_loops = Vec::new();
    loop {
        if main_cfg.node_count() == 1 { break; }
        let mut progress = false;
        let entry = find_entry_node(&main_cfg).context("No entry node found")?;
        let mut order = DfsPostOrder::new(&main_cfg, entry);
        let dom = simple_fast(&main_cfg, entry_node);

        while let Some(head) = order.next(&main_cfg) {
            if has_backedge(&main_cfg, head, vexit_node, &dom) {
                println!("cycle discovered: {}", head.index());
                let mut raw_loop = find_smallest_loop(&main_cfg, head, structured_loops.len(), &dom);
                let (structured_loop, tail_edges) = match_cyclic(&mut main_cfg, &mut raw_loop, entry, vexit_node, &mut block_store);
                virtualized_edges.extend(tail_edges);
                structured_loops.push(structured_loop.clone());

                // Create a new graph for the body of the loop
                let mut body_graph = main_cfg.clone();
                for n in body_graph.clone().node_indices() {
                    if !structured_loop.body.contains(body_graph.node_weight(n).unwrap()) {
                        body_graph.remove_node(n);
                    }
                }
                let (mut body_graph, body_vexit) = add_virtual_exit(body_graph, &mut block_store)?;
                let body_vexit_bid = *body_graph.node_weight(body_vexit).unwrap();
                block_id_to_rid.insert(body_vexit_bid, arena.alloc(Region::Leaf(body_vexit_bid)));
                write_cfg_dot(&body_graph, &block_store, format!("body_graph_of_loop_{}.dot", structured_loop.index).as_str());

                // Contract the body of the loop
                loop {
                    let mut body_progress = false;
                    if body_graph.node_count() <= 2 { break; }
                    let body_entry = block_store.get_node_index(structured_loop.entry).or_else(|| graph_utils::find_entry_node(&body_graph)).expect("loop body entry");
                    let mut order = DfsPostOrder::new(&body_graph, body_entry);
                    let dom = simple_fast(&body_graph, body_entry);

                    // Contract Acyclic Patterns in the body of the loop
                    while let Some(head) = order.next(&body_graph) {
                        if match_acyclic(&mut body_graph, head, body_vexit, &mut seq_count, &mut block_store, &mut arena, &mut block_id_to_rid, &dom) { 
                            body_progress = true;
                            break; 
                        }
                    }
                    if !body_progress { return Err(anyhow::anyhow!("Failed to find any acyclic pattern in loop body of index {}", structured_loop.index)); }
                }

                // Contract the loop in the main CFG
                let loop_block_id = contract_cyclic_region(&mut main_cfg, &structured_loop, &mut block_store, &mut arena, &mut block_id_to_rid);
                body_graph.remove_node(body_vexit);
                if body_graph.node_count() == 1 {
                    if let Some(root_node) = body_graph.node_indices().next() {
                        let &body_root_bid = body_graph.node_weight(root_node).unwrap();
                        let &body_root_rid = block_id_to_rid.get(&body_root_bid).unwrap();
                        let &loop_rid = block_id_to_rid.get(&loop_block_id).unwrap();
                        arena.set_loop_body(loop_rid, body_root_rid);
                    }
                }
                progress = true;
            } else {
                progress = 
                    match_acyclic(&mut main_cfg, head, vexit_node, &mut seq_count, &mut block_store, &mut arena, &mut block_id_to_rid, &dom);
            }
        }
        if !progress { break; }
    }

    if main_cfg.node_count() == 2 {
        main_cfg.remove_node(vexit_node);
        let root_node = main_cfg.node_indices().next().context("root node not found")?;
        let root_bid = *main_cfg.node_weight(root_node).context("root node weight not found")?;
        let root_rid = block_id_to_rid.get(&root_bid).context("root region not found for root block")?;
        visualize::arena_output::print_arena_tree(*root_rid, &arena);
    }
	Ok(String::from("Hello, Decompiler!"))
}
