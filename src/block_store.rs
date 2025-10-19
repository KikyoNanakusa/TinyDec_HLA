use std::collections::HashMap;
use petgraph::graph::NodeIndex;
use anyhow::{Context, Result};

use crate::{block::{Block, BlockId}, instruction::Instruction, util::graph_utils::Cfg};

#[derive(Clone, Default)]
pub struct BlockStore {
	next_id: BlockId, 
	pub blocks: HashMap<BlockId, Block>,
	pub block_to_node: HashMap<BlockId, Option<NodeIndex>>,
}

impl BlockStore {
	pub fn new() -> Self { Self::default() }
	pub fn get(&self, id: BlockId) -> Option<&Block> { self.blocks.get(&id) }
	pub fn get_mut(&mut self, id: BlockId) -> Option<&mut Block> { self.blocks.get_mut(&id) }
	pub fn get_node_index(&self, id: BlockId) -> Option<NodeIndex> { *self.block_to_node.get(&id).unwrap_or(&None) }
	pub fn new_block(&mut self, start: u64, end: u64, insn: Vec<Instruction>, label: Option<String>) -> BlockId {
		let id = self.next_id; 
		self.next_id += 1; 
		let block = Block::new(id, start, end, insn, label);
		self.blocks.insert(id, block);
		id
	}
	pub fn add_block_to_graph(&mut self, graph: &mut Cfg, block_id: BlockId) -> Result<NodeIndex>{
		let block = self.get(block_id).context("Block missing")?;
		let node_index = graph.add_node(block.id);
		self.block_to_node.insert(block_id, Some(node_index));
		Ok(node_index)
	}
	pub fn remove_block_from_graph(&mut self, graph: &mut Cfg, block_id: BlockId) -> Result<()>{
		let node_index = self.block_to_node.get(&block_id).context("Node index for Block missing")?;
		if let Some(node_index) = node_index { graph.remove_node(*node_index); }
		Ok(())
	}
}