#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    fret_examples::date_picker_demo::run()
}

#[cfg(target_arch = "wasm32")]
fn main() {}
