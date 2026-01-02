#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    fret_demo::components_gallery::run()
}

#[cfg(target_arch = "wasm32")]
fn main() {}
