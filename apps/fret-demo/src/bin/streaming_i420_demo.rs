#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    fret_examples::streaming_i420_demo::run()
}

#[cfg(target_arch = "wasm32")]
fn main() {}
