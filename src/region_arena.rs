use std::collections::HashMap;

use crate::{block::BlockId, cfg_structuring::{if_schema::IfSchema, structured_loop::{LoopKind, StructuredLoop}}, region::{Region, RegionId}};

pub struct RegionArena { regions: Vec<Region> }
impl RegionArena {
    pub fn new() -> Self { Self { regions: vec![] } }
    pub fn alloc(&mut self, r: Region) -> RegionId { let id = self.regions.len(); self.regions.push(r); id}
    pub fn get(&self, id: RegionId) -> &Region { &self.regions[id] }
    pub fn attach_seq(&mut self, seq_bid: BlockId, inner_bids: Vec<BlockId>, bid2rid: &mut HashMap<BlockId, RegionId>) -> RegionId {
        let rids = self.rids_of(bid2rid, inner_bids.iter());
        let rid = self.alloc(Region::Seq(rids));
        bid2rid.insert(seq_bid, rid);
        rid
    }
    pub fn attach_if_then_region(&mut self, if_block_id: BlockId,meta: IfSchema, block_to_rid: &mut HashMap<BlockId, RegionId>) -> RegionId {
        let head_rid = self.rid_of(block_to_rid, meta.if_node);
        let then_br_rid = self.rids_of(block_to_rid, meta.then_nodes.iter());
        let join_rid = self.rid_of(block_to_rid, meta.join_node);
        let rid = self.alloc(Region::IfThen { head: head_rid, then_br: then_br_rid, join: join_rid, cond: meta.cond });
        block_to_rid.insert(if_block_id, rid);
        rid
    }
    pub fn attach_if_then_else_region(&mut self, if_block_id: BlockId, meta: IfSchema, block_to_rid: &mut HashMap<BlockId, RegionId>) -> RegionId {
        let head_rid = self.rid_of(block_to_rid, meta.if_node);
        let then_br_rid = self.rids_of(block_to_rid, meta.then_nodes.iter());
        let else_br_rid = self.rids_of(block_to_rid, meta.else_nodes.as_ref().expect("else_nodes not found in meta").iter());
        let join_rid = self.rid_of(block_to_rid, meta.join_node);
        let rid = self.alloc(Region::IfThenElse { head: head_rid, then_br: then_br_rid, else_br: else_br_rid, join: join_rid, cond: meta.cond });
        block_to_rid.insert(if_block_id, rid);
        rid
    }
    pub fn attach_cyclic_region(&mut self, cyclic_block_id: BlockId, structured_loop: &StructuredLoop, block_to_rid: &mut HashMap<BlockId, RegionId>) -> RegionId {
        let body_rids = self.rids_of(block_to_rid, structured_loop.body.iter());
        let body_rid = self.alloc(Region::Seq(body_rids));
        let rid = match structured_loop.kind {
            LoopKind::While => self.alloc(Region::LoopWhile { meta: structured_loop.clone(), body: body_rid }),
            LoopKind::DoWhile => self.alloc(Region::LoopDoWhile { meta: structured_loop.clone(), body: body_rid }),
            LoopKind::NatLoop => self.alloc(Region::LoopNat { meta: structured_loop.clone(), body: body_rid })
        };
        block_to_rid.insert(cyclic_block_id, rid);
        rid
    }
    pub fn set_loop_body(&mut self, loop_rid: RegionId, new_body: RegionId) {
        match &mut self.regions[loop_rid] {
            Region::LoopWhile { body, .. } => *body = new_body,
            Region::LoopDoWhile { body, .. } => *body = new_body,
            Region::LoopNat { body, .. } => *body = new_body,
            _ => {}
        }
    }

    fn rid_of(&self, bid2rid: &HashMap<BlockId, RegionId>, bid: BlockId) -> RegionId{
        *bid2rid.get(&bid).expect(&format!("block {} not found in bid2rid", bid))
    }
    fn rids_of<'a, I>(&self, bid2rid: &HashMap<BlockId, RegionId>, bids: I) -> Vec<RegionId>
    where I: IntoIterator<Item = &'a BlockId> { 
        bids.into_iter().map(|&bid| self.rid_of(bid2rid, bid)).collect()
    }
}