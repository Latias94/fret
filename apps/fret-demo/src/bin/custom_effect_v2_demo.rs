#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    fret_examples::custom_effect_v2_demo::run()
}

#[cfg(target_arch = "wasm32")]
fn main() {}
