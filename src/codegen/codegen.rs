use std::fmt::Write as _;

use crate::{
    block::BlockId, block_store::BlockStore, condition::Condition, edge_label::{EdgeLabel, TailKind}, region::{Region, RegionId}, region_arena::RegionArena
};

fn indent(buf: &mut String, n: usize) { for _ in 0..n { buf.push_str("    "); } }

fn emit_block_comment(buf: &mut String, bs: &BlockStore, bid: BlockId, indent_lvl: usize) {
    if let Some(b) = bs.get(bid) {
        indent(buf, indent_lvl);
        if let Some(lbl) = &b.label { let _ = writeln!(buf, "/* {} (Block {}) */", lbl, b.id); }
        else { let _ = writeln!(buf, "/* Block {} */", b.id); }
    }
}

fn emit_stmt(buf: &mut String, indent_lvl: usize, s: &str) {
    indent(buf, indent_lvl);
    let _ = writeln!(buf, "{}", s);
}

fn decompile_insn(mnemonic: &str, operands: &[String]) -> Option<String> {
    // Very lightweight pseudo-C for common patterns. Fallback: show asm-like line.
    match mnemonic {
        // moves
        "mov" | "movzx" | "movsx" => if operands.len() == 2 { Some(format!("{} = {};", operands[0], operands[1])) } else { None },
        // arithmetic
        "add" => if operands.len() == 2 { Some(format!("{} += {};", operands[0], operands[1])) } else { None },
        "sub" => if operands.len() == 2 { Some(format!("{} -= {};", operands[0], operands[1])) } else { None },
        "inc" => if operands.len() == 1 { Some(format!("{}++;", operands[0])) } else { None },
        "dec" => if operands.len() == 1 { Some(format!("{}--;", operands[0])) } else { None },
        "imul" => {
            if operands.len() == 2 { Some(format!("{} *= {};", operands[0], operands[1])) }
            else if operands.len() == 3 { Some(format!("{} = ({} * {});", operands[0], operands[1], operands[2])) }
            else { None }
        }
        "idiv" | "div" => {
            if operands.len() == 1 { Some(format!("/* {} {} (implicit dividend in rdx:rax) */", mnemonic, operands[0])) } else { None }
        }
        // bitwise
        "and" => if operands.len() == 2 { Some(format!("{} &= {};", operands[0], operands[1])) } else { None },
        "or"  => if operands.len() == 2 { Some(format!("{} |= {};", operands[0], operands[1])) } else { None },
        "xor" => {
            if operands.len() == 2 {
                if operands[0] == operands[1] { Some(format!("{} = 0;", operands[0])) }
                else { Some(format!("{} ^= {};", operands[0], operands[1])) }
            } else { None }
        }
        "shl" | "sal" => if operands.len() == 2 { Some(format!("{} <<= {};", operands[0], operands[1])) } else { None },
        "shr" | "sar" => if operands.len() == 2 { Some(format!("{} >>= {};", operands[0], operands[1])) } else { None },

        // load effective address: keep as-is
        "lea" => if operands.len() == 2 { Some(format!("{} = &({});", operands[0], operands[1])) } else { None },

        // stack
        "push" => if operands.len() == 1 { Some(format!("push({});", operands[0])) } else { None },
        "pop"  => if operands.len() == 1 { Some(format!("{} = pop();", operands[0])) } else { None },

        // comparisons/tests usually feed a branch: keep as comment
        "cmp" | "test" => Some(format!("/* {} {} */", mnemonic, operands.join(", "))),

        // calls and returns
        "call" => Some(format!("{}({});", operands.get(0).cloned().unwrap_or_default(), "")),
        "ret" | "retf" | "retfq" => Some("return;".to_string()),

        // default to asm-like
        _ => Some(format!("/* {} {} */", mnemonic, operands.join(", "))),
    }
}

fn emit_block_body(buf: &mut String, bs: &BlockStore, bid: BlockId, indent_lvl: usize) {
    if let Some(b) = bs.get(bid) {
        // Emit simple pseudo-C for non-terminator instructions; place control-flow tails elsewhere.
        let mut insns = b.instructions.iter().peekable();
        while let Some(insn) = insns.next() {
            let mnemonic = insn.mnemonic.as_str();
            // Skip unconditional/conditional jumps here; handled by structured regions
            if insn.is_jump() { continue; }
            let ops = insn.operands.as_ref().unwrap().clone();
            if let Some(line) = decompile_insn(mnemonic, &ops) {
                emit_stmt(buf, indent_lvl, &line);
            }
        }
    }
}

fn emit_virtualized_tails(buf: &mut String, bid: BlockId, vtails: &[(BlockId, BlockId, EdgeLabel)], indent_lvl: usize) {
    for (_src, _tgt, lab) in vtails.iter().filter(|(s, _, _)| *s == bid) {
        indent(buf, indent_lvl);
        match lab {
            EdgeLabel::Virtualized(TailKind::Break { .. }) => { buf.push_str("break;\n"); }
            EdgeLabel::Virtualized(TailKind::Continue { .. }) => { buf.push_str("continue;\n"); }
            EdgeLabel::Virtualized(TailKind::Goto { target }) => { let _ = writeln!(buf, "goto L{};", target); }
            _ => {}
        }
    }
}

fn emit_region(arena: &RegionArena, bs: &BlockStore, rid: RegionId, buf: &mut String, indent_lvl: usize, vtails: &[(BlockId, BlockId, EdgeLabel)]) {
    match arena.get(rid) {
        Region::Leaf(bid) => {
            emit_block_comment(buf, bs, *bid, indent_lvl);
            emit_block_body(buf, bs, *bid, indent_lvl);
            emit_virtualized_tails(buf, *bid, vtails, indent_lvl);
        }
        Region::Seq(items) => emit_seq(arena, bs, items, buf, indent_lvl, vtails),
        Region::IfThen { head, then_br, join: _, cond } => emit_if_then(arena, bs, *head, then_br, cond.as_ref(), buf, indent_lvl, vtails),
        Region::IfThenElse { head, then_br, else_br, join: _, cond } => emit_if_then_else(arena, bs, *head, then_br, else_br, cond.as_ref(), buf, indent_lvl, vtails),
        Region::LoopWhile { meta, body } => emit_loop_with_body(arena, bs, *body, buf, indent_lvl, vtails, |
            buf, indent_lvl| {
                indent(buf, indent_lvl);
                buf.push_str("while (");
                if let Some(c) = &meta.cond { let _ = write!(buf, "{}", c); }
                else { buf.push_str("/* cond from "); let _ = write!(buf, "Block {}", meta.head); buf.push_str(" */"); }
                buf.push_str(") {\n");
            },
            |buf, indent_lvl| { indent(buf, indent_lvl); buf.push_str("}\n"); }
        ),
        Region::LoopDoWhile { meta, body } => emit_loop_with_body(arena, bs, *body, buf, indent_lvl, vtails, |
            buf, indent_lvl| { indent(buf, indent_lvl); buf.push_str("do {\n"); },
            |buf, indent_lvl| {
                indent(buf, indent_lvl);
                buf.push_str("} while (");
                if let Some(c) = &meta.cond { let _ = write!(buf, "{}", c); }
                else { buf.push_str("/* cond from "); let _ = write!(buf, "Block {}", meta.head); buf.push_str(" */"); }
                buf.push_str(");\n");
            }
        ),
        Region::LoopNat { meta: _meta, body } => emit_loop_with_body(arena, bs, *body, buf, indent_lvl, vtails, |
            buf, indent_lvl| { indent(buf, indent_lvl); buf.push_str("while (/* unknown cond; nat loop */) {\n"); },
            |buf, indent_lvl| { indent(buf, indent_lvl); buf.push_str("}\n"); }
        ),
    }
}

pub fn generate_function(arena: &RegionArena, root: RegionId, bs: &BlockStore, name: &str, vtails: &[(BlockId, BlockId, EdgeLabel)]) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "void {}() {{", name);
    emit_region(arena, bs, root, &mut out, 1, vtails);
    out.push_str("}\n");
    out
}

fn emit_seq(arena: &RegionArena, bs: &BlockStore, items: &[RegionId], buf: &mut String, indent_lvl: usize, vtails: &[(BlockId, BlockId, EdgeLabel)]) {
    for &rid in items { emit_region(arena, bs, rid, buf, indent_lvl, vtails); }
}

fn emit_if_then(arena: &RegionArena, bs: &BlockStore, head: RegionId, then_br: &[RegionId], cond: Option<&Condition>, buf: &mut String, indent_lvl: usize, vtails: &[(BlockId, BlockId, EdgeLabel)]) {
    emit_if_head(buf, arena, head, cond, indent_lvl);
    emit_seq(arena, bs, then_br, buf, indent_lvl + 1, vtails);
    indent(buf, indent_lvl); buf.push_str("}\n");
    // Note: do not emit the join here; it appears as a separate region in sequence.
}

fn emit_if_then_else(arena: &RegionArena, bs: &BlockStore, head: RegionId, then_br: &[RegionId], else_br: &[RegionId], cond: Option<&Condition>, buf: &mut String, indent_lvl: usize, vtails: &[(BlockId, BlockId, EdgeLabel)]) {
    emit_if_head(buf, arena, head, cond, indent_lvl);
    emit_seq(arena, bs, then_br, buf, indent_lvl + 1, vtails);
    indent(buf, indent_lvl); buf.push_str("} else {\n");
    emit_seq(arena, bs, else_br, buf, indent_lvl + 1, vtails);
    indent(buf, indent_lvl); buf.push_str("}\n");
    // Note: do not emit the join here; it appears as a separate region in sequence.
}

fn emit_if_head(buf: &mut String, arena: &RegionArena, head: RegionId, cond: Option<&Condition>, indent_lvl: usize) {
    indent(buf, indent_lvl);
    buf.push_str("if (");
    if let Some(c) = cond { let _ = write!(buf, "{}", c); }
    else {
        buf.push_str("/* cond from ");
        if let Region::Leaf(bid) = arena.get(head) { let _ = write!(buf, "Block {}", bid); }
        buf.push_str(" */");
    }
    buf.push_str(") {\n");
}

fn emit_loop_with_body<Open, Close>(
    arena: &RegionArena,
    bs: &BlockStore,
    body: RegionId,
    buf: &mut String,
    indent_lvl: usize,
    vtails: &[(BlockId, BlockId, EdgeLabel)],
    open: Open,
    close: Close,
) where
    Open: FnOnce(&mut String, usize),
    Close: FnOnce(&mut String, usize),
{
    open(buf, indent_lvl);
    emit_region(arena, bs, body, buf, indent_lvl + 1, vtails);
    close(buf, indent_lvl);
}