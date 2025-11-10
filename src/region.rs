use crate::block::BlockId;
use crate::cfg_structuring::structured_loop::StructuredLoop;
use crate::condition::Condition;

#[derive(Clone)]
pub enum Region {
    Leaf (BlockId),
    Seq (Vec<RegionId>),
    IfThen      { head: RegionId, then_br: Vec<RegionId>, join: RegionId, cond: Option<Condition> },
    IfThenElse  { head: RegionId, then_br: Vec<RegionId>, else_br: Vec<RegionId>, join: RegionId, cond: Option<Condition> },
    LoopWhile   { meta: StructuredLoop, body: RegionId },
    LoopDoWhile { meta: StructuredLoop, body: RegionId },
    LoopNat     { meta: StructuredLoop, body: RegionId },
}

pub type RegionId = usize;