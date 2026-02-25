#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    fret_examples::chart_stress_demo::run()
}

#[cfg(target_arch = "wasm32")]
fn main() {}
