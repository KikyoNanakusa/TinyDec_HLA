use crate::region::{Region, RegionId};
use crate::region_arena::RegionArena;

fn indent(buf: &mut String, n: usize) { for _ in 0..n { buf.push_str("  "); } }

pub fn print_arena_tree(root_region_id: RegionId, arena: &RegionArena) {
	println!("Arena tree:");
	let root_region = arena.get(root_region_id);
	let indent_lvl = 0;
	let mut buf = String::new();
	print_region(root_region, arena, indent_lvl, &mut buf);
	println!("{}", buf);
}

fn print_region(region: &Region, arena: &RegionArena, mut indent_lvl: usize, buf: &mut String) {
	indent_lvl += 1;
	indent(buf, indent_lvl);
	match region {
		Region::Leaf(bid) => {
			buf.push_str(&format!("Leaf: {}", bid));
			buf.push_str("\n");
		}
		Region::Seq(items) => {
			buf.push_str(&format!("Seq: {}", items.len()));
			buf.push_str("\n");
			for item in items {
				print_region(arena.get(*item), arena, indent_lvl + 1, buf);
			}
		}
		Region::IfThen { head, then_br, join, .. } => {
			buf.push_str(&format!("IfThen: {}", head));
			buf.push_str("\n");
			print_region(arena.get(*head), arena, indent_lvl + 1, buf);
			for item in then_br {
				print_region(arena.get(*item), arena, indent_lvl + 1, buf);
			}
			print_region(arena.get(*join), arena, indent_lvl + 1, buf);
		}
		Region::IfThenElse { head, then_br, else_br, join, .. } => {
			buf.push_str(&format!("IfThenElse: {}", head));
			buf.push_str("\n");
			print_region(arena.get(*head), arena, indent_lvl + 1, buf);
			for item in then_br {
				print_region(arena.get(*item), arena, indent_lvl + 1, buf);
			}
			for item in else_br {
				print_region(arena.get(*item), arena, indent_lvl + 1, buf);
			}
			print_region(arena.get(*join), arena, indent_lvl + 1, buf);
		}
		Region::LoopWhile { meta, body } => {
			buf.push_str(&format!("LoopWhile: {}", meta.head));
			buf.push_str("\n");
			print_region(arena.get(*body), arena, indent_lvl + 1, buf);
		}
		Region::LoopDoWhile { meta, body } => {
			buf.push_str(&format!("LoopDoWhile: {}", meta.head));
			buf.push_str("\n");
			print_region(arena.get(*body), arena, indent_lvl + 1, buf);
		}
		Region::LoopNat { meta, body } => {
			buf.push_str(&format!("LoopNat: {}", meta.head));
			buf.push_str("\n");
			print_region(arena.get(*body), arena, indent_lvl + 1, buf);
		}
	}
}
