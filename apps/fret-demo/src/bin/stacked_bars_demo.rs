#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    fret_examples::stacked_bars_demo::run()
}

#[cfg(target_arch = "wasm32")]
fn main() {}
