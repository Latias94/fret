#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    fret_demo::docking_arbitration_demo::run()
}

#[cfg(target_arch = "wasm32")]
fn main() {}
