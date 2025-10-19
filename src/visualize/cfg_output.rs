use std::{fs::File, io::Write, process::Command};
use petgraph::visit::IntoEdgeReferences;
use petgraph::{prelude::StableGraph, visit::EdgeRef};
use petgraph::graph::NodeIndex;

use crate::block::BlockId;
use crate::block_store::BlockStore;
use crate::edge_label::EdgeLabel;

fn edge_label_and_style(label: &EdgeLabel) -> (&'static str, &'static str) {
    match label {
        EdgeLabel::TrueBranch(_)      => ("True",       "color=green"),
        EdgeLabel::FalseBranch(_)     => ("False",      "color=red"),
        EdgeLabel::Unconditional      => ("",           "color=black"),
        EdgeLabel::Virtualized        => ("Virtualized", "color=gray, style=dashed"),
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
}

pub fn write_cfg_dot(
    cfg: &StableGraph<BlockId, EdgeLabel>,
	block_store: &BlockStore,
    filename: &str,
) {
    const SKIP_VIRTUALIZED_EDGES: bool = true;

    // image directory
    let image_dir = "images";
    std::fs::create_dir_all(image_dir).expect("Failed to create image directory");

    let file_path = format!("{}/{}.dot", image_dir, filename);
    let mut file = File::create(&file_path).expect("Failed to create .dot file");

    writeln!(file, "digraph cfg {{").expect("write failed");
    writeln!(file, "  rankdir=TB;").expect("write failed");
    writeln!(file, "  node [shape=plaintext];").expect("write failed");

    // ノードは NodeIndex を使って安定に出力
    for n in cfg.node_indices() {
        let block_id = *cfg.node_weight(n).expect("missing node weight");
        let block = block_store.get(block_id).expect("missing block");
        let block_str = if let Some(label) = &block.label {
            format!("NodeIndex({}), BlockId({}) {}:\n{}", n.index(), block_id, label, block)
        } else {
            format!("NodeIndex({}), BlockId({}) {}", n.index(), block_id, block)
        };
        let html = block_str
            .lines()
            .map(escape_html)
            .collect::<Vec<_>>()
            .join("<BR/>");
        // ノードIDは n{index}
        writeln!(file, "  n{} [label=<{}>];", n.index(), html).expect("write failed");
    }

    for e in cfg.edge_references() {
        let u: NodeIndex = e.source();
        let v: NodeIndex = e.target();
        let w = e.weight();

        if SKIP_VIRTUALIZED_EDGES {
            if matches!(w, EdgeLabel::Virtualized) {
                continue;
            }
        }

        let (label, style) = edge_label_and_style(w);
        if label.is_empty() {
            writeln!(file, "  n{} -> n{} [{}];", u.index(), v.index(), style).expect("write failed");
        } else {
            writeln!(
                file,
                "  n{} -> n{} [label=\"{}\", {}];",
                u.index(),
                v.index(),
                label,
                style
            )
            .expect("write failed");
        }
    }

    writeln!(file, "}}").expect("write failed");
    file.flush().ok();

    // generate PNG
    let png_name = if let Some(stem) = file_path.strip_suffix(".dot") {
        format!("{stem}.png")
    } else {
        format!("{}.png", file_path)
    };

    let output = Command::new("dot")
        .arg("-Tpng")
        .arg(file_path)
        .arg("-o")
        .arg(&png_name)
        .output()
        .expect("Failed to run dot");

    if !output.status.success() {
        eprintln!("dot command failed:");
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    } else {
        println!("{} successfully generated", png_name);
    }
}
