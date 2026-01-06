#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    fret_examples::node_graph_demo::run()
}

#[cfg(target_arch = "wasm32")]
fn main() {}

