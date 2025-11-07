use std::collections::{HashMap, HashSet, VecDeque};

use petgraph::{Direction::{Incoming, Outgoing}, algo::dominators::Dominators, graph::NodeIndex, visit::EdgeRef};
use crate::{block::BlockId, block_store::BlockStore, cfg_structuring::if_schema::IfSchema, edge_label::EdgeLabel, region::RegionId, region_arena::RegionArena, util::graph_utils::{Cfg, calc_post_dominator, redirect_incoming, redirect_outgoing, remove_nodes_descending}, visualize::cfg_output::write_cfg_dot};

pub fn match_acyclic(
	cfg: &mut Cfg, 
	head: NodeIndex, 
	vexit: NodeIndex, 
	seq_count: &mut usize, 
	block_store: &mut BlockStore, 
	arena: &mut RegionArena, 
	block_id_to_rid: &mut HashMap<BlockId, RegionId>, 
	dom: &Dominators<NodeIndex>
) -> bool {
    if let Some(seq) = find_seq(cfg, head, vexit) {
        let seq_inners: Vec<BlockId> = seq.iter().map(|nid| *cfg.node_weight(*nid).expect(format!("node {:?} not found in main_tree", nid).as_str())).collect();
        let seq_block = contract_seq(cfg, &seq, seq_count, block_store);
        arena.attach_seq(seq_block, seq_inners, block_id_to_rid);
		write_cfg_dot(cfg, block_store, format!("seq_{}.dot", seq_count).as_str());
        return true;
    } 

	let pdom = calc_post_dominator(cfg, vexit);
    if let Some(if_then_else) = find_if_then_else(cfg, &dom, &pdom, head, vexit) {
        let if_then_else_block_id = contract_if_then_else(cfg, block_store, &if_then_else);
        arena.attach_if_then_else_region(if_then_else_block_id, if_then_else, block_id_to_rid);
        write_cfg_dot(cfg, block_store, format!("if_then_else_contracted{}.dot", if_then_else_block_id).as_str());
		return true;
    }

    if let Some(if_then) = find_if_then(cfg, &dom, &pdom, head, vexit) {
        let if_then_block_id = contract_if_then(cfg, block_store, &if_then);
        arena.attach_if_then_region(if_then_block_id, if_then, block_id_to_rid);
        write_cfg_dot(cfg, block_store, format!("if_then_contracted{}.dot", if_then_block_id).as_str());
		return true;
    }

	false
}

fn contract_seq(cfg: &mut Cfg, seq: &Vec<NodeIndex>, seq_count: &mut usize, block_store: &mut BlockStore) -> BlockId {
	let seq_head = *seq.first().unwrap();
	let seq_tail = *seq.last().unwrap();

	let seq_block_id = block_store.new_block(0, 0, Vec::new(), Some(format!("contracted seq {}", *seq_count)));
	let seq_node = block_store.add_block_to_graph(cfg, seq_block_id).unwrap();

	redirect_incoming(cfg, seq_head, seq_node, true);
	redirect_outgoing(cfg, seq_tail, seq_node, true);

	*seq_count += 1;
    remove_nodes_descending(cfg, block_store, seq.iter().copied(), None);
    seq_block_id
}

fn find_seq(cfg: &Cfg, head: NodeIndex, vexit: NodeIndex) -> Option<Vec<NodeIndex>>{ 
	if head == vexit { return None; }
	let incoming_count = cfg.edges_directed(head, Incoming).count();
	let is_start = if incoming_count != 1 {
		true
	} else {
		let mut preds = cfg.neighbors_directed(head, Incoming);
		match (preds.next(), preds.next()) {
			(Some(p), None) => cfg.edges_directed(p, Outgoing).count() != 1,
			_ => true,
		}
	};

	if !is_start { return None; }
	let mut chain = vec![head];
	let mut target = head;
	while cfg.edges_directed(target, Outgoing).count() == 1  {
		let mut succs = cfg.neighbors_directed(target, Outgoing);
		let succ = match (succs.next(), succs.next()) {
			(Some(s), None) => s,
			_ => break,
		};
		if cfg.edges_directed(succ, Incoming).count() == 1 && succ != vexit {
			chain.push(succ);
			target = succ;
		} else {
			break;
		}
	}
	if chain.len() > 1 { return Some(chain); }
	None
}

fn find_if_then(cfg: &Cfg, dom: &Dominators<NodeIndex>, pdom: &Dominators<NodeIndex>, head: NodeIndex, vexit: NodeIndex) -> Option<IfSchema> {
	if cfg.edges_directed(head, Outgoing).count() != 2 { return None; }
	let join = match pdom.immediate_dominator(head) {
		Some(j) => j, 
		None => { return None; }
	};

	let mut succs = cfg.neighbors_directed(head, Outgoing);
	let succ1 = succs.next().unwrap();
	let succ2 = succs.next().unwrap();
	if succ1 == vexit || succ2 == vexit { return None; }

	let then_entry = if succ1 == join { succ2 } 
								else if succ2 == join { succ1 }
								else { return None; }; // find else pattern in other place

	let head_to_join = cfg.edge_weight(cfg.find_edge(head, join).unwrap());
	let head_to_then = cfg.edge_weight(cfg.find_edge(head, then_entry).unwrap());
	match (head_to_join, head_to_then) {
		(Some(EdgeLabel::TrueBranch(_)), Some(EdgeLabel::FalseBranch(_))) => {},
		(Some(EdgeLabel::FalseBranch(_)), Some(EdgeLabel::TrueBranch(_))) => {},
		_ => { return None; }
	}

    let then_nodes = collect_nodes_between(cfg, head, then_entry, join, dom, pdom);
    if then_nodes.is_empty() || contains_sese_violation(cfg, head, then_entry, join, &then_nodes) { return None; }

    let cond = match (head_to_join, head_to_then) {
        (Some(EdgeLabel::TrueBranch(c1)), Some(EdgeLabel::FalseBranch(_))) => c1.clone(),
        (Some(EdgeLabel::FalseBranch(c1)), Some(EdgeLabel::TrueBranch(_))) => c1.clone(),
        _ => None,
    };
    let then_is_true = matches!(head_to_then, Some(EdgeLabel::TrueBranch(_)));
    let norm_cond = cond.map(|c| if then_is_true { c } else { c.negate() });
    Some(IfSchema::new(
        *cfg.node_weight(head).unwrap(), 
        then_nodes.iter().map(|&n| *cfg.node_weight(n).unwrap()).collect::<Vec<_>>(), 
        None, 
        *cfg.node_weight(join).unwrap(),
        norm_cond,
    ))
}

fn contract_if_then(cfg: &mut Cfg, block_store: &mut BlockStore, if_then_schema: &IfSchema) -> BlockId {
    let head_node = block_store.get_node_index(if_then_schema.if_node).unwrap();
    let join_node = block_store.get_node_index(if_then_schema.join_node).unwrap();

    let blocks = if_then_schema.all_blocks();
    let nodes = blocks.iter().map(|&bid| block_store.get_node_index(bid).unwrap()).collect::<Vec<_>>();

    let if_then_block_id = block_store.new_block(0, 0, Vec::new(), Some(format!("if_then_{}", if_then_schema.if_node)));
    let if_then_node = block_store.add_block_to_graph(cfg, if_then_block_id).unwrap();

    redirect_incoming(cfg, head_node, if_then_node, true);
    if let Some(head_to_join) = cfg.find_edge(head_node, join_node) {
        let w = cfg.edge_weight(head_to_join).unwrap().clone();
        cfg.add_edge(if_then_node, join_node, w);
        cfg.remove_edge(head_to_join);
    }

    remove_nodes_descending(cfg, block_store, nodes.into_iter(), Some(join_node));
    if_then_block_id
}

fn find_if_then_else(cfg: &Cfg, dom: &Dominators<NodeIndex>, pdom: &Dominators<NodeIndex>, head: NodeIndex, vexit: NodeIndex) -> Option<IfSchema> {
    if cfg.edges_directed(head, Outgoing).count() != 2 { return None; }
    let join = match pdom.immediate_dominator(head) { 
		Some(j) => j, 
		None => return None,
	};

    let mut succs = cfg.neighbors_directed(head, Outgoing);
    let s1 = succs.next().unwrap();
    let s2 = succs.next().unwrap();
    if s1 == join || s2 == join || s1 == vexit || s2 == vexit { return None; } 

    let then_entry = s1;
    let else_entry = s2;
	let head_to_then = cfg.edge_weight(cfg.find_edge(head, then_entry).unwrap());
	let head_to_else = cfg.edge_weight(cfg.find_edge(head, else_entry).unwrap());
	match (head_to_then, head_to_else) {
		(Some(EdgeLabel::TrueBranch(_)), Some(EdgeLabel::FalseBranch(_))) => {},
		(Some(EdgeLabel::FalseBranch(_)), Some(EdgeLabel::TrueBranch(_))) => {},
		_ => { return None; }
	}

    let then_body = collect_nodes_between(cfg, head, then_entry, join, dom, pdom);
    let else_body = collect_nodes_between(cfg, head, else_entry, join, dom, pdom);

	if contains_sese_violation(cfg, head, then_entry, join, &then_body) { return None; }
    if contains_sese_violation(cfg, head, else_entry, join, &else_body) { return None; }

    if !then_body.is_disjoint(&else_body) { return None; }
    let head_to_then = cfg.edge_weight(cfg.find_edge(head, then_entry).unwrap());
    let head_to_else = cfg.edge_weight(cfg.find_edge(head, else_entry).unwrap());
    let cond = match (head_to_then, head_to_else) {
        (Some(EdgeLabel::TrueBranch(c1)), Some(EdgeLabel::FalseBranch(_))) => c1.clone(),
        (Some(EdgeLabel::FalseBranch(c1)), Some(EdgeLabel::TrueBranch(_))) => c1.clone(),
        _ => None,
    };
    let then_is_true = matches!(head_to_then, Some(EdgeLabel::TrueBranch(_)));
    let norm_cond = cond.map(|c| if then_is_true { c } else { c.negate() });
    Some(IfSchema::new(
        *cfg.node_weight(head).unwrap(), 
        then_body.iter().map(|&n| *cfg.node_weight(n).unwrap()).collect::<Vec<_>>(), 
        Some(else_body.iter().map(|&n| *cfg.node_weight(n).unwrap()).collect::<Vec<_>>()), 
        *cfg.node_weight(join).unwrap(),
        norm_cond,
    ))
}

fn contract_if_then_else(cfg: &mut Cfg, block_store: &mut BlockStore, schema: &IfSchema) -> BlockId {
    let blocks = schema.all_blocks();
    let nodes = blocks.iter().map(|&bid| block_store.get_node_index(bid).unwrap()).collect::<Vec<_>>();
    let head_node = block_store.get_node_index(schema.if_node).unwrap();
    let join_node = block_store.get_node_index(schema.join_node).unwrap();

    let if_then_else_block_id = block_store.new_block(0, 0, Vec::new(), Some(format!("if_then_else_{}", schema.if_node)));
    let if_then_else_node = block_store.add_block_to_graph(cfg, if_then_else_block_id).unwrap();

    redirect_incoming(cfg, head_node, if_then_else_node, true);
    if let Some(then_tail_bid) = schema.then_nodes.last() {
        let then_tail = block_store.get_node_index(*then_tail_bid).unwrap();
        if let Some(eid) = cfg.find_edge(then_tail, join_node) { cfg.remove_edge(eid); }
    }
    if let Some(else_nodes) = &schema.else_nodes {
        if let Some(else_tail_bid) = else_nodes.last() {
            let else_tail = block_store.get_node_index(*else_tail_bid).unwrap();
            if let Some(eid) = cfg.find_edge(else_tail, join_node) { cfg.remove_edge(eid); }
        }
    }
    cfg.add_edge(if_then_else_node, join_node, EdgeLabel::Unconditional);

    remove_nodes_descending(cfg, block_store, nodes.into_iter(), Some(join_node));
    if_then_else_block_id
}

fn collect_nodes_between(cfg: &Cfg, region_head: NodeIndex, branch_entry: NodeIndex, join: NodeIndex, dom: &Dominators<NodeIndex>, pdom: &Dominators<NodeIndex>) -> HashSet<NodeIndex> {
	let mut queue = VecDeque::new();
	let mut visited = HashSet::new();
	queue.push_back(branch_entry);

	while let Some(u) = queue.pop_front() {
		if u == join || visited.contains(&u) { continue; }
		match dom.dominators(u) {
			Some(mut it) => { if !it.any(|v| v == region_head) { continue; } }
			None => { continue; }
		}

		match pdom.dominators(u) {
			Some(mut it) => { if !it.any(|v| v == join) { continue; } }
			None => { continue; }
		}

		visited.insert(u);
		for v in cfg.neighbors_directed(u, Outgoing) { queue.push_back(v); }
	}
	visited
}

fn contains_sese_violation(cfg: &Cfg, head: NodeIndex, then_entry: NodeIndex, join: NodeIndex, region_nodes: &HashSet<NodeIndex>) -> bool {
	for &node in region_nodes {
		for edge in cfg.edges_directed(node, Incoming) {
			let source = edge.source();
			if !region_nodes.contains(&source) && !(source == head && node == then_entry) { return true; }
		}
	}
	for edge in cfg.edges_directed(head, Outgoing) {
		let target = edge.target();
		if region_nodes.contains(&target) && target != then_entry { return true; }
	}
	for &node in region_nodes {
		for edge in cfg.edges_directed(node, Outgoing) {
			let target = edge.target();
			if !region_nodes.contains(&target) && target != join { return true; }
		}
	}
	false
}
