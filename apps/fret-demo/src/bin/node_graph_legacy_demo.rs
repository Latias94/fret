#[cfg(all(not(target_arch = "wasm32"), feature = "node-graph-demos-legacy"))]
fn main() -> anyhow::Result<()> {
    fret_examples::node_graph_legacy_demo::run()
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "node-graph-demos-legacy")))]
fn main() {
    eprintln!("node_graph_legacy_demo is disabled (enable `fret-demo/node-graph-demos-legacy`).");
}

#[cfg(target_arch = "wasm32")]
fn main() {}
