use crate::{block::BlockId, condition::Condition};

pub struct IfSchema {
	pub if_node: BlockId,
	pub then_nodes: Vec<BlockId>,
	pub else_nodes: Option<Vec<BlockId>>,
	pub join_node: BlockId,
    pub cond: Option<Condition>,
}

impl IfSchema {
	pub fn new(if_node: BlockId, then_nodes: Vec<BlockId>, else_nodes: Option<Vec<BlockId>>, join_node: BlockId, cond: Option<Condition>) -> Self {
		Self { if_node, then_nodes, else_nodes, join_node, cond }
	}
	pub fn all_blocks(&self) -> Vec<BlockId> {
		let mut nodes = Vec::new();
		nodes.push(self.if_node);
		nodes.extend(self.then_nodes.iter().copied());
		if let Some(else_nodes) = &self.else_nodes {
			nodes.extend(else_nodes.iter().copied());
		}
		nodes.push(self.join_node);
		nodes
	}
}
