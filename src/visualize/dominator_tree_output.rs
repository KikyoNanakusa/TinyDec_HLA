use petgraph::graph::NodeIndex;
use petgraph::algo::dominators::Dominators;
use std::{fs::File, io::Write, process::Command};

use crate::{block::BlockStore, util::graph_utils::Cfg};

#[allow(dead_code)]
pub fn write_dominator_tree_dot(
    cfg: &Cfg,
	block_store: &BlockStore,
    dom: &Dominators<NodeIndex>,
    filename: &str,
) {
    // image directory
    let image_dir = "images";
    std::fs::create_dir_all(image_dir).expect("Failed to create image directory");

    // create file
    let file_path = format!("{}/{}.dot", image_dir, filename);
    let mut file = File::create(&file_path)
        .unwrap_or_else(|e| panic!("Failed to create {}: {}", file_path, e));

    // header
    writeln!(file, "digraph dominator_tree {{").unwrap();
    writeln!(file, "  rankdir=TB;").unwrap();
    writeln!(file, "  node [shape=box, style=filled, fillcolor=lightgray];").unwrap();

    // output node labels
    for node in cfg.node_indices() {
        let label_str = {
            let blk = block_store.get(*cfg.node_weight(node).unwrap()).unwrap();
            if let Some(name) = &blk.label {
                format!("\"{}\"", name)
            } else {
                format!("\"0x{:x}\"", blk.start)
            }
        };
        writeln!(file, "  {} [label={}];", node.index(), label_str).unwrap();
    }

    // output edges: each node's immediate dominator → node
    for node in cfg.node_indices() {
        if let Some(idom) = dom.immediate_dominator(node) {
            writeln!(
                file,
                "  {} -> {} [color=blue];",
                idom.index(),
                node.index()
            )
            .unwrap();
        }
    }

    writeln!(file, "}}").unwrap();

    let png = format!("{}.png", file_path);
    let _ = Command::new("dot")
        .arg("-Tpng")
        .arg(file_path)
        .arg("-o")
        .arg(&png)
        .output()
        .expect("Failed to run dot");
}