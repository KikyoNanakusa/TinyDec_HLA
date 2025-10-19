use crate::{block_store::BlockStore, util::graph_utils::Cfg, edge_label::EdgeLabel};
use anyhow::{anyhow, Result};
use petgraph::graph::NodeIndex;


pub fn add_virtual_exit(
    mut graph: Cfg,
    block_store: &mut BlockStore,
) -> Result<(Cfg, NodeIndex)>{
    let mut exit_nodes = Vec::new();

    for node in graph.node_indices() {
        if graph.edges(node).next().is_none() { exit_nodes.push(node); }
    }
    if exit_nodes.is_empty() { return Err(anyhow!("Cyclic control flow detected")) }

    let virtual_exit_block_id = block_store.new_block(0, 0, vec![], Some("virtual_exit".to_string()));
    let virtual_exit_node = block_store.add_block_to_graph(&mut graph, virtual_exit_block_id)?;
    for exit_node in exit_nodes {
        graph.add_edge(exit_node, virtual_exit_node, EdgeLabel::Unconditional);
    }

    Ok((graph, virtual_exit_node))
}

