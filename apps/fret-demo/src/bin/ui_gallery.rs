#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    fret_ui_gallery::run()
}

#[cfg(target_arch = "wasm32")]
fn main() {}
