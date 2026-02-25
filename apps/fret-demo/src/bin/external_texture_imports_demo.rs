#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    fret_examples::external_texture_imports_demo::run()
}

#[cfg(target_arch = "wasm32")]
fn main() {}
