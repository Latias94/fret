#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    fret_demo::ime_smoke_demo::run()
}

#[cfg(target_arch = "wasm32")]
fn main() {}
