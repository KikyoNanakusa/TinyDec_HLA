use std::collections::HashSet;

use petgraph::{Direction::Incoming, algo::dominators::Dominators, graph::{EdgeIndex, NodeIndex}, visit::EdgeRef};

use crate::{block::BlockId, block_store::BlockStore, cfg_structuring::structured_loop::LoopKind, edge_label::{EdgeLabel, TailKind}, raw_loop::RawLoop, util::graph_utils::{Cfg, dominates}};

pub fn ensure_single_entry(cfg: &mut Cfg, raw_loop: &mut RawLoop) -> (NodeIndex, Vec<(BlockId, BlockId, EdgeLabel)>) {
	let mut virtualized_edges = Vec::new();
	let mut edge_counts: Vec<(NodeIndex, usize)> = raw_loop.entries.iter().map(|&n| {
		let c = cfg.edges_directed(n, Incoming)
		.filter(|e| !raw_loop.nodes.contains(&e.source()))
		.count();
		(n, c)
	}).collect();
	edge_counts.sort_by_key(|(n, c)| (*c, n.index()));
	let entry = edge_counts.last().unwrap().0; // Node with the max number of incoming edges

	let mut to_virtualize: Vec<EdgeIndex> = Vec::new();
	for &node in raw_loop.entries.iter() {
		if node == entry { continue; }
		for edge in cfg.edges_directed(node, Incoming) {
			if raw_loop.nodes.contains(&edge.source()) { continue; }
			to_virtualize.push(edge.id());
		}
	}

	for eid in to_virtualize {
		let (source, target) = cfg.edge_endpoints(eid).unwrap();
		virtualized_edges.push(virtualize_edge(cfg, eid, source, target));
	}
	(entry, virtualized_edges)
}

pub fn virtualize_edge(cfg: &mut Cfg, eid: EdgeIndex, u: NodeIndex, v: NodeIndex) -> (BlockId, BlockId, EdgeLabel) {
    cfg.remove_edge(eid);
    let virtualized_edge = EdgeLabel::Virtualized(TailKind::Goto { target: *cfg.node_weight(v).unwrap() });
	(*cfg.node_weight(u).unwrap(), *cfg.node_weight(v).unwrap(), virtualized_edge)
}

pub struct SuccesorInfo {
	pub succ: Option<NodeIndex>, 
	pub kind: LoopKind,
	pub exit_edge: Option<EdgeIndex>,
}

pub fn select_single_successor(
	cfg: &mut Cfg, 
	raw_loop: &mut RawLoop, 
	dom: &Dominators<NodeIndex>, 
	pdom: &Dominators<NodeIndex>
) -> Option<SuccesorInfo> { 
	let prefered_succ = pdom.immediate_dominator(raw_loop.head);
	let exit_edges = raw_loop.exit_edges.clone();

	// While loop
	let while_edges: HashSet<EdgeIndex> = exit_edges.iter().cloned().filter(|eid| {
		let (source, _) = cfg.edge_endpoints(*eid).unwrap();
		source == raw_loop.head
	}).collect();
	if !while_edges.is_empty() {
		let edge = pick_edge_deterministically(cfg, &while_edges, prefered_succ);
		let (_, succ) = cfg.edge_endpoints(edge).unwrap();
		return Some(SuccesorInfo { succ: Some(succ), kind: LoopKind::While, exit_edge: Some(edge) });
	}
	// Do-While loop
	let srcs = collect_backedge_sources(cfg, &raw_loop, &dom);
	let dowhile_edges: HashSet<EdgeIndex> = exit_edges.iter().cloned().filter(|eid| {
		let (source, _) = cfg.edge_endpoints(*eid).unwrap();
		srcs.contains(&source)
	}).collect();
	if !dowhile_edges.is_empty() {
		let edge = pick_edge_deterministically(cfg, &dowhile_edges, prefered_succ);
		if let Some((_, succ)) = cfg.edge_endpoints(edge) {
			return Some(SuccesorInfo { succ: Some(succ), kind: LoopKind::DoWhile, exit_edge: Some(edge) });
		}
	}
	// Nat loop
	if exit_edges.is_empty() { return Some(SuccesorInfo { succ: None, kind: LoopKind::NatLoop, exit_edge: None }); }
	let nat_edge = pick_edge_deterministically(cfg, &exit_edges, prefered_succ);
	if let Some((_, succ)) = cfg.edge_endpoints(nat_edge) {
		return Some(SuccesorInfo { succ: Some(succ), kind: LoopKind::NatLoop, exit_edge: Some(nat_edge) });
	}

	None
}		

fn collect_backedge_sources(cfg: &Cfg, raw_loop: &RawLoop, dom: &Dominators<NodeIndex>) -> HashSet<NodeIndex> {
	let mut sources = HashSet::new();
	for e in cfg.edges_directed(raw_loop.head, Incoming) {
		let u = e.source();
		if dominates(raw_loop.head, u, dom) && !raw_loop.nodes.contains(&u) {
			sources.insert(u);
		}
	}
	sources
}

fn pick_edge_deterministically(cfg: &Cfg, eids: &HashSet<EdgeIndex>, prefered_succ: Option<NodeIndex>) -> EdgeIndex {
	if let Some(succ) = prefered_succ {
		for &eid in eids {
			let (_, target) = cfg.edge_endpoints(eid).unwrap();
			if target == succ { return eid; }
		}
	}
	*eids.iter().min().unwrap()
}

pub fn virtualize_loop_tails(
	cfg: &mut Cfg, 
	raw_loop: &mut RawLoop, 
	succ_info: &SuccesorInfo,
	body: &HashSet<BlockId>,
	block_store: &mut BlockStore,
) -> Vec<(BlockId, BlockId, EdgeLabel)> {
	let mut changed = Vec::new();
	let mut virtualized_edges = Vec::new();

	match succ_info {
		SuccesorInfo { succ: None, ..} => { return vec![]; },
		SuccesorInfo { exit_edge: None, ..} => { return vec![]; },
		_ => {},
	}

	let succ = succ_info.succ.unwrap();
	let keep_exit_edge = match succ_info.kind {
		LoopKind::NatLoop => None,
		_ => succ_info.exit_edge,
	};

	for &bid in body.iter() {
		for e in cfg.edges(block_store.get_node_index(bid).unwrap()) {
			let eid = e.id();
			let t = e.target();

			if let Some(exit_edge) = keep_exit_edge {
				if eid == exit_edge { continue; }
			}

			let lab = if t == raw_loop.head {
				EdgeLabel::Virtualized(TailKind::Continue)
			} else if t == succ {
				EdgeLabel::Virtualized(TailKind::Break)
			} else if body.contains(cfg.node_weight(t).unwrap()) { 
				continue; 
			} else {
				EdgeLabel::Virtualized(TailKind::Goto { target: *cfg.node_weight(t).unwrap() })
			};
			changed.push((eid, block_store.get_node_index(bid).unwrap(), t, lab));
		}
	}

	for (_, source, target, lab) in changed {
		while let Some(edge) = cfg.find_edge(source, target) {
			cfg.remove_edge(edge);
		}
		virtualized_edges.push((*cfg.node_weight(source).unwrap(), *cfg.node_weight(target).unwrap(), lab));
	}
	virtualized_edges
}