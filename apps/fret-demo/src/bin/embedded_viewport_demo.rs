#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    fret_examples::embedded_viewport_demo::run()
}

#[cfg(target_arch = "wasm32")]
fn main() {}
