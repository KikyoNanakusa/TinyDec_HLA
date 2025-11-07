use std::collections::{HashMap, HashSet, VecDeque};
use petgraph::{Direction::{Incoming, Outgoing}, algo::dominators::{Dominators, simple_fast}, graph::NodeIndex, visit::EdgeRef};
use crate::{block::BlockId, block_store::BlockStore, cfg_structuring::{structured_loop::StructuredLoop, virtualize::{ensure_single_entry, select_single_successor, virtualize_loop_tails}}, edge_label::EdgeLabel, raw_loop::RawLoop, region::RegionId, region_arena::RegionArena, util::graph_utils::{Cfg, calc_post_dominator, dominates, redirect_incoming_except, remove_nodes_descending}, visualize::cfg_output::write_cfg_dot};

pub fn match_cyclic(
	cfg: &mut Cfg, 
	raw_loop: &mut RawLoop, 
	entry_node: NodeIndex, 
	vexit: NodeIndex, 
	block_store: &mut BlockStore,
) -> (StructuredLoop, Vec<(BlockId, BlockId, EdgeLabel)>) {
	let mut tail_edges = Vec::new();
	let (single_entry, tails) = ensure_single_entry(cfg, raw_loop);
	tail_edges.extend(tails);

	let dom = &simple_fast(&*cfg, entry_node);
	let pdom = &calc_post_dominator(cfg, vexit);
	let succ_info = select_single_successor(cfg, raw_loop, dom, pdom).expect("no single successor found");

	let dom = &simple_fast(&*cfg, entry_node);
	let pdom = &calc_post_dominator(cfg, vexit);
	let body = match succ_info.succ {
		Some(succ) => build_loop_body(cfg, single_entry, succ, dom, pdom, vexit),
		None => raw_loop.nodes.iter().map(|n| *cfg.node_weight(*n).unwrap()).collect(),
	};

	let virtualized_tail_edges = virtualize_loop_tails(cfg, raw_loop, &succ_info, &body, block_store);
	tail_edges.extend(virtualized_tail_edges);
	let structured_loop = StructuredLoop::build_structured_loop(raw_loop, succ_info.kind, single_entry, 
		succ_info.succ, succ_info.exit_edge, cfg, &body);
	(structured_loop, tail_edges)
}

pub fn find_smallest_loop(cfg: &Cfg, target: NodeIndex, index: usize, dom: &Dominators<NodeIndex>) -> RawLoop {
	let mut heads = HashSet::new();
	for edge in cfg.edges_directed(target, Outgoing) {
		let head = edge.target();
		if dominates(head, target, &dom) {
			heads.insert(head);
		}
	}

	let mut loops = Vec::new();
	for head in heads {
		let mut nodes = HashSet::new();
		nodes.insert(target);
		nodes.insert(head);
		let mut worklist = VecDeque::new();
		worklist.push_back(target);
		while let Some(node) = worklist.pop_front() {
			for pred in cfg.edges_directed(node, Incoming).map(|e| e.source()) {
				if pred != head && nodes.insert(pred) {
					worklist.push_back(pred);
				}
			}
		}
		loops.push(RawLoop::new(cfg, index, nodes, head));
	}
	loops.into_iter().min_by_key(|l| l.nodes.len()).expect("no loop found")
}

pub fn contract_cyclic_region(
	cfg: &mut Cfg, 
	structured_loop: &StructuredLoop, 
	block_store: &mut BlockStore, 
	arena: &mut RegionArena, 
	block_id_to_rid: &mut HashMap<BlockId, RegionId>
) -> BlockId {
	let loop_block_id = contract_loop(cfg, structured_loop, block_store);
	arena.attach_cyclic_region(loop_block_id, structured_loop, block_id_to_rid);
	write_cfg_dot(&cfg, &block_store, format!("cyclic_loop_contracted{}.dot", structured_loop.index).as_str());
	loop_block_id
}

fn contract_loop(cfg: &mut Cfg, structured_loop: &StructuredLoop, block_store: &mut BlockStore) -> BlockId{ 
	let entry_nid = block_store.get_node_index(structured_loop.entry).unwrap();
	let body_nids: Vec<NodeIndex> = structured_loop.body.iter().map(|&n| block_store.get_node_index(n).unwrap()).collect();
	let succ = block_store.get_node_index(structured_loop.succ.unwrap());
	let exit_eid = structured_loop.exit_edge;

	let label = Some(format!("contracted loop {}", structured_loop.index));
	let loop_block_id = block_store.new_block(0, 0, Vec::new(), label);
	let loop_node = block_store.add_block_to_graph(cfg, loop_block_id).unwrap();

    // Redirect incoming edges to entry from outside the loop body to the loop node
    let body_set: HashSet<NodeIndex> = body_nids.iter().cloned().collect();
    redirect_incoming_except(cfg, entry_nid, loop_node, &body_set, true);

    // Rewire exit edge to originate from the loop node and remove body->succ links
    match (succ, exit_eid) {
        (Some(s), Some(eid)) => {
            let w = cfg.edge_weight(eid).unwrap().clone();
            for u in &body_nids { 
                while let Some(e2) = cfg.find_edge(*u, s) { cfg.remove_edge(e2); } 
            }
            while let Some(e2) = cfg.find_edge(block_store.get_node_index(structured_loop.head).unwrap(), s) { cfg.remove_edge(e2); }
            cfg.remove_edge(eid);
            if cfg.find_edge(loop_node, s).is_none() {
                cfg.add_edge(loop_node, s, w);
            }
        }
        (None, None) => {}
        _ => panic!("invalid structured loop"),
    }

    remove_nodes_descending(cfg, block_store, body_nids.iter().cloned(), None);
	loop_block_id
}

fn build_loop_body(
	cfg: &Cfg, 
	head:NodeIndex, 
	succ: NodeIndex, 
	dom: &Dominators<NodeIndex>, 
	pdom: &Dominators<NodeIndex>, 
	vexit: NodeIndex
) -> HashSet<BlockId> {
	cfg.node_indices()
	.filter(|&n| dominates(head, n, dom))
	.filter(|&n| dominates(succ, n, pdom) && n != succ)
	.filter(|&n| n != vexit)
	.map(|n| *cfg.node_weight(n).unwrap())
	.collect::<HashSet<BlockId>>()
}