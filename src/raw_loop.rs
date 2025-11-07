use std::collections::HashSet;
use petgraph::{Direction::{Incoming, Outgoing}, graph::{EdgeIndex, NodeIndex}, visit::EdgeRef};
use crate::util::graph_utils::Cfg;

#[derive(Debug, Clone)]
pub struct RawLoop {
	pub loop_index: usize,
	pub nodes: HashSet<NodeIndex>,
	pub head: NodeIndex,
	pub entries: HashSet<NodeIndex>,
	pub exit_edges: HashSet<EdgeIndex>,
}

impl RawLoop {
	pub fn new(cfg: &Cfg, loop_index: usize, nodes: HashSet<NodeIndex>, head: NodeIndex) -> Self {
		let entries = extract_entry_nodes(&nodes, cfg);
		let exit_edges = extract_exit_edges(&nodes, cfg);
		Self { loop_index, nodes, head, entries, exit_edges }
	}
}

fn extract_entry_nodes(nodes: &HashSet<NodeIndex>, cfg: &Cfg) -> HashSet<NodeIndex> {
        let mut entrance_nodes = HashSet::new();
        for v in nodes.iter() {
            for e in cfg.edges_directed(*v, Incoming) {
                if !nodes.contains(&e.source()) {
                    entrance_nodes.insert(*v);
                }
            }
        }
    entrance_nodes
}

fn extract_exit_edges(nodes: &HashSet<NodeIndex>, cfg: &Cfg) -> HashSet<EdgeIndex> {
    let mut exit_edges = HashSet::new();
    for v in nodes.iter() {
        for e in cfg.edges_directed(*v, Outgoing) {
            if !nodes.contains(&e.target()) {
                exit_edges.insert(e.id());
            }
        }
    }
    exit_edges
}