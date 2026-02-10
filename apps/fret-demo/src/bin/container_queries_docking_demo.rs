#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    fret_examples::container_queries_docking_demo::run()
}

#[cfg(target_arch = "wasm32")]
fn main() {}
