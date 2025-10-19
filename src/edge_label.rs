use crate::condition::Condition;

#[derive(Clone, PartialEq)]
pub enum EdgeLabel { 
	TrueBranch(Option<Condition>),
	FalseBranch(Option<Condition>),
	Unconditional, 
	Virtualized,
}
