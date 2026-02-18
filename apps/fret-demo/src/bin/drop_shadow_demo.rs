#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    fret_examples::drop_shadow_demo::run()
}

#[cfg(target_arch = "wasm32")]
fn main() {}
