use crate::block::BlockId;
use crate::condition::Condition;

#[derive(Clone)]
pub enum Region {
    Leaf (BlockId),
    Seq (Vec<RegionId>),
    IfThen      { head: RegionId, then_br: Vec<RegionId>, join: RegionId, cond: Option<Condition> },
    IfThenElse  { head: RegionId, then_br: Vec<RegionId>, else_br: Vec<RegionId>, join: RegionId, cond: Option<Condition> },
}

pub type RegionId = usize;