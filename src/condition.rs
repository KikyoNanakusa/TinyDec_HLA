use std::fmt;

use crate::block::Block;

#[derive(Clone, PartialEq)]
pub struct Condition {
    pub op: String,
    pub lhs: String,
    pub rhs: String,
}

impl Condition {
    pub fn new(lhs: impl Into<String>, op: impl Into<String>, rhs: impl Into<String>) -> Self {
        Self { lhs: lhs.into(), op: op.into(), rhs: rhs.into() }
    }
    pub fn negate(&self) -> Self {
        let nop = match self.op.as_str() {
            "==" => "!=",
            "!=" => "==",
            "<" => ">=",
            "<=" => ">",
            ">" => "<=",
            ">=" => "<",
            other => other,
        };
        Self { lhs: self.lhs.clone(), op: nop.to_string(), rhs: self.rhs.clone() }
    }
    pub fn from_block(block: &Block) -> Option<Self> {
        if block.instructions.len() < 2 { return None; }
        let term = block.instructions.last().unwrap();
        if !term.is_conditional_jump() { return None; }
        let prev = &block.instructions[block.instructions.len() - 2];
        let jcc = term.mnemonic.to_lowercase();
        let op = Self::jcc_to_op(&jcc)?;
        let ops = prev.operands.as_ref().unwrap().clone();
        match prev.mnemonic.as_str() {
            "cmp" => {
                if ops.len() == 2 {
                    Some(Condition::new(ops[0].clone(), op, ops[1].clone()))
                } else { None }
            }
            "test" => {
                if matches!(jcc.as_str(), "je" | "jz" | "jne" | "jnz") {
                    if ops.len() == 2 {
                        let expr = format!("({} & {})", ops[0], ops[1]);
                        Some(Condition::new(expr, op, "0"))
                    } else if ops.len() == 1 {
                        Some(Condition::new(ops[0].clone(), op, "0"))
                    } else { None }
                } else { None }
            }
            _ => None,
        }
    }
    fn jcc_to_op(mnemonic: &str) -> Option<&'static str> {
        match mnemonic {
            // equal / not equal
            "je" | "jz" => Some("=="),
            "jne" | "jnz" => Some("!="),
            // signed
            "jl" | "jnge" => Some("<"),
            "jle" | "jng" => Some("<="),
            "jg" | "jnle" => Some(">"),
            "jge" | "jnl" => Some(">="),
            // unsigned
            "jb" | "jnae" => Some("<"),
            "jbe" | "jna" => Some("<="),
            "ja" | "jnbe" => Some(">"),
            "jae" | "jnb" => Some(">="),
            _ => None,
        }
    }
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.lhs, self.op, self.rhs)
    }
}