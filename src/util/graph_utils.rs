use std::collections::{HashMap, HashSet, VecDeque};
use petgraph::algo::dominators::{simple_fast, Dominators};
use petgraph::visit::{DfsPostOrder, Reversed};
use petgraph::Direction::{Incoming, Outgoing};
use petgraph::{graph::NodeIndex, visit::EdgeRef};
use petgraph::prelude::StableGraph;
use crate::{block::{BlockId, BlockStore}, edge_label::EdgeLabel};

pub type Cfg = StableGraph<BlockId, EdgeLabel>;
pub type Tree = StableGraph<BlockId, EdgeLabel>;

/// Build a forward-reachable subgraph (Tree) from `cfg`, starting at `entry_node`.
/// Returns the constructed tree and the corresponding entry node in that tree.
pub fn extract_tree(cfg: &Cfg, entry_node: NodeIndex, block_store: &mut BlockStore) -> (Tree, NodeIndex) {
	let mut tree: Tree = StableGraph::new();
	let mut cfg2tree = HashMap::new(); // map from cfg node index to tree node index

	let mut visited = HashSet::new(); 
	let entry = block_store.add_block_to_graph(&mut tree, *cfg.node_weight(entry_node).unwrap()).unwrap();
	cfg2tree.insert(entry_node, entry);
	
	let mut worklist = VecDeque::new(); 
	worklist.push_back(entry_node);
	visited.insert(entry_node);
	
	while let Some(node) = worklist.pop_front() {
		for edge in cfg.edges(node) {
			let succ = edge.target();
			let succ_tree_node = *cfg2tree.entry(succ).or_insert_with(|| {
				block_store.add_block_to_graph(&mut tree, *cfg.node_weight(succ).unwrap()).unwrap()
			});

			let tree_node = cfg2tree[&node];
			tree.add_edge(tree_node, succ_tree_node, edge.weight().clone());
			if visited.insert(succ) {
				worklist.push_back(succ);
			}
		}
	}
	(tree, entry)
}

/// Returns true if `dominator` dominates `target` in the given dominator tree.
pub fn dominates(
    dominator: NodeIndex, 
    target: NodeIndex, 
    dominator_tree: &Dominators<NodeIndex>
) -> bool {
	dominator_tree
        .dominators(target)
        .map_or(false, |mut it| it.any(|d| d == dominator))
}

/// Compute post-dominators for `tree` using `virtual_exit_node` as the sink.
/// If multiple exits exist, temporarily add edges to the virtual exit to form a single sink.
pub fn calc_post_dominator(tree: &Tree, virtual_exit_node: NodeIndex) -> Dominators<NodeIndex> {
	let exit_nodes = tree.
        node_indices()
        .filter(|&n| tree.edges_directed(n, Outgoing).count() == 0)
        .filter(|&n| n != virtual_exit_node)
        .collect::<Vec<_>>();

	if exit_nodes.is_empty() {
		return simple_fast(Reversed(&tree), virtual_exit_node);
	}

	let mut tmp_tree = tree.clone();
	for exit_node in exit_nodes {
		tmp_tree.add_edge(exit_node, virtual_exit_node, EdgeLabel::Unconditional);
	}
	let post_dom = simple_fast(
        Reversed(&tmp_tree), 
        virtual_exit_node
    );

	post_dom
}

/// Quick cycle predicate: true if `target` has an outgoing edge to a node that dominates it.
/// Skips the designated `exit` node.
pub fn is_cyclic(
    tree: &Tree, 
    target: NodeIndex, 
    exit: NodeIndex, 
    dom: &Dominators<NodeIndex>
) -> bool {
	if target == exit { return false }
	for edge in tree.edges_directed(target, Outgoing) {
		let head = edge.target();
		if dominates(head, target, &dom) { 
            return true; 
        }
	}
	false
}

/// Find a node with zero incoming edges that has at least one neighbor.
/// Returns `None` if no such entry is found.
pub fn find_entry_node(tree: &Tree) -> Option<NodeIndex> {
    let mut entries = tree
        .node_indices()
        .filter(|&n| tree.edges_directed(n, Incoming).count() == 0);
    entries.find(|&n| tree.neighbors(n).count() > 0)
}

/// Compute DFS post-order from `entry_node`.
pub fn get_dfs_post_order(tree: &Tree, entry_node: NodeIndex) -> Vec<NodeIndex> {
    let mut dfs = DfsPostOrder::new(&tree, entry_node);
    let mut order = Vec::new();
    while let Some(n) = dfs.next(&tree) { order.push(n); }
    order
}

/// Remove a set of nodes from the graph in descending `NodeIndex` order.
/// If `except` is provided, skip removing that node even if present.
pub fn remove_nodes_descending<I>(
	tree: &mut Tree, 
	block_store: &mut BlockStore, 
	nodes: I, 
	except: Option<NodeIndex>
) where I: IntoIterator<Item = NodeIndex> {
    let mut worklist: Vec<NodeIndex> = nodes
        .into_iter()
        .filter(|&n| Some(n) != except)
        .collect();
    worklist.sort_by_key(|&n| std::cmp::Reverse(n.index()));
    for n in worklist {
        if let Some(&bid) = tree.node_weight(n) {
            let _ = block_store.remove_block_from_graph(tree, bid);
        }
    }
}

/// Redirect all incoming edges of `target` to `new_target`.
/// If `avoid_duplicates` is true, skip adding an edge if one already exists.
pub fn redirect_incoming(
    tree: &mut Tree,
    target: NodeIndex,
    new_target: NodeIndex,
    avoid_duplicates: bool,
) {
    let mut edges = Vec::new();
    for e in tree.edges_directed(target, Incoming) {
        edges.push((e.id(), e.source()));
    }
    for (eid, s) in edges {
        let w = tree.edge_weight(eid).unwrap().clone();
        tree.remove_edge(eid);
        if avoid_duplicates && tree.find_edge(s, new_target).is_some() { continue; }
        tree.add_edge(s, new_target, w);
    }
}

/// Redirect incoming edges of `target` to `new_target`, skipping sources in `exclude_sources`.
/// If `avoid_duplicates` is true, skip adding an edge if one already exists.
pub fn redirect_incoming_except(
    tree: &mut Tree,
    target: NodeIndex,
    new_target: NodeIndex,
    exclude_sources: &HashSet<NodeIndex>,
    avoid_duplicates: bool,
) {
    let mut edges = Vec::new();
    for e in tree.edges_directed(target, Incoming) {
        let s = e.source();
        if exclude_sources.contains(&s) { continue; }
        edges.push((e.id(), s));
    }
    for (eid, s) in edges {
        let w = tree.edge_weight(eid).unwrap().clone();
        tree.remove_edge(eid);
        if avoid_duplicates && tree.find_edge(s, new_target).is_some() { continue; }
        tree.add_edge(s, new_target, w);
    }
}

/// Redirect all outgoing edges of `source` to originate from `new_source`.
/// If `avoid_duplicates` is true, skip adding an edge if one already exists.
pub fn redirect_outgoing(
    tree: &mut Tree,
    source: NodeIndex,
    new_source: NodeIndex,
    avoid_duplicates: bool,
) {
    let mut edges = Vec::new();
    for e in tree.edges_directed(source, Outgoing) {
        edges.push((e.id(), e.target()));
    }
    for (eid, t) in edges {
        let w = tree.edge_weight(eid).unwrap().clone();
        tree.remove_edge(eid);
        if avoid_duplicates && tree.find_edge(new_source, t).is_some() { continue; }
        tree.add_edge(new_source, t, w);
    }
}
