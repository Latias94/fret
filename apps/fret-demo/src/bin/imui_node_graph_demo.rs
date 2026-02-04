#[cfg(all(not(target_arch = "wasm32"), feature = "node-graph-demos"))]
fn main() -> anyhow::Result<()> {
    fret_examples::imui_node_graph_demo::run()
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "node-graph-demos")))]
fn main() {
    eprintln!("imui_node_graph_demo is disabled (enable `fret-demo/node-graph-demos`).");
}

#[cfg(target_arch = "wasm32")]
fn main() {}
