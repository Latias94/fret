#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    fret_examples::markdown_demo::run()
}

#[cfg(target_arch = "wasm32")]
fn main() {}

