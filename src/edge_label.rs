use crate::{block::BlockId, condition::Condition};

#[derive(Clone, PartialEq)]
pub enum EdgeLabel { 
	TrueBranch(Option<Condition>),
	FalseBranch(Option<Condition>),
	Unconditional, 
	Virtualized(TailKind),
}

#[derive(Clone, Debug, PartialEq)]
pub enum TailKind {
	Break,
	Continue,
	Goto {target: BlockId},
}
