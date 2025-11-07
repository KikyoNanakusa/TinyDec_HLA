use std::collections::HashSet;
use petgraph::graph::{EdgeIndex, Node, NodeIndex};

use crate::raw_loop::RawLoop;
use crate::{block::BlockId, condition::Condition, edge_label::EdgeLabel};
use crate::util::graph_utils::Cfg;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopKind { While, DoWhile, NatLoop }

#[derive(Clone)]
pub struct StructuredLoop {
	pub index: usize,
	pub kind: LoopKind, // While | DoWhile | NatLoop
	pub head: BlockId,
	pub entry: BlockId, 
	pub succ: Option<BlockId>,
	pub body: HashSet<BlockId>, // lexical containment
	pub exit_edge: Option<EdgeIndex>,
	pub tails: Vec<(BlockId /* source */, BlockId /* target */, EdgeLabel)>,
    pub cond: Option<Condition>,
}

impl StructuredLoop {
	pub fn new(
		index: usize, 
		kind: LoopKind, 
		head: BlockId, 
		entry: BlockId, 
		succ: Option<BlockId>, 
		body: HashSet<BlockId>, 
		tails: Vec<(BlockId, BlockId, EdgeLabel)>, 
		exit_edge: Option<EdgeIndex>, 
		cond: Option<Condition>
	) -> Self {
		Self { index, kind, head, entry, succ, body, tails, exit_edge, cond }
	}
	pub fn build_structured_loop(
		raw_loop: &RawLoop, 
		kind: LoopKind, 
		single_entry: NodeIndex, 
		single_succ: Option<NodeIndex>, 
		single_exit_edge: Option<EdgeIndex>,
		cfg: &Cfg, 
		tails: &Vec<(BlockId, BlockId, EdgeLabel)>, 
		body: &HashSet<BlockId>
	) -> StructuredLoop {
		let id = raw_loop.loop_index;
		let head: BlockId = *cfg.node_weight(raw_loop.head).expect("head must exist in tree");
		let entry: BlockId = *cfg.node_weight(single_entry).expect("entry must exist in tree");
		let succ: Option<BlockId> = single_succ.map(|n| *cfg.node_weight(n).expect("succ must exist in tree"));
		let cond = extract_condition(cfg, raw_loop.head, single_entry, single_succ);

		StructuredLoop::new(
			id, kind, head, 
			entry, succ,
			body.clone(), tails.clone(), 
			single_exit_edge, cond)
	}
}

fn extract_condition(cfg: &Cfg, head_idx: NodeIndex, entry_idx: NodeIndex, succ_idx: Option<NodeIndex>) -> Option<Condition> {
	let mut cond: Option<Condition> = None;
	if let Some(eid) = cfg.find_edge(head_idx, entry_idx) {
		match cfg.edge_weight(eid) {
			Some(EdgeLabel::TrueBranch(c)) => { cond = c.clone(); }
			Some(EdgeLabel::FalseBranch(c)) => { cond = c.clone().map(|cc| cc.negate()); }
			_ => {}
		}
	} else if let Some(sidx) = succ_idx {
		if let Some(eid) = cfg.find_edge(head_idx, sidx) {
			match cfg.edge_weight(eid) {
				Some(EdgeLabel::TrueBranch(c)) => { cond = c.clone().map(|cc| cc.negate()); }
				Some(EdgeLabel::FalseBranch(c)) => { cond = c.clone(); }
				_ => {}
			}
		}
	}
	cond
}
